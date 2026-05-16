import { createStore, type StoreApi } from "zustand/vanilla";

import { BOARD_SIZE } from "../board/constants";
import { BotRunner } from "../core/bot_runner";
import type { BotSpec, GameVariant } from "../core/bot_protocol";
import {
  DEFAULT_BOT_CONFIG,
  botPlayerName,
  resolveBotConfig,
  sanitizeBotConfig,
  type BotConfig,
} from "../core/bot_config";
import { WasmBoard } from "../core/wasm_bridge";

import type { CellPosition, CellStone, MatchMove, MatchPlayer, MatchStatus } from "./types";

type MatchBotRunner = Pick<BotRunner, "chooseMove" | "configure" | "dispose"> &
  Partial<Pick<BotRunner, "cancelPending">>;

interface FinishedLocalMatch {
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  botConfig: BotConfig;
  status: Exclude<MatchStatus, "playing">;
  undoFloor: number;
  variant: GameVariant;
  winningCells: CellPosition[];
}

export interface LocalMatchState {
  cells: CellStone[][];
  counterThreatMoves: CellPosition[];
  currentPlayer: 1 | 2;
  currentBotConfig: BotConfig;
  currentVariant: GameVariant;
  forbiddenMoves: CellPosition[];
  imminentThreatMoves: CellPosition[];
  lastMove: CellPosition | null;
  moves: MatchMove[];
  pendingBotMove: boolean;
  players: [MatchPlayer, MatchPlayer];
  playerClockMs: [number, number];
  selectedBotConfig: BotConfig;
  selectedVariant: GameVariant;
  selectBotConfig: (botConfig: BotConfig) => void;
  selectVariant: (variant: GameVariant) => void;
  startNewMatch: () => void;
  startNextRound: () => void;
  status: MatchStatus;
  threatMoves: CellPosition[];
  turnStartedAtMs: number;
  undoFloor: number;
  undoLastTurn: () => boolean;
  placeHumanMove: (row: number, col: number) => boolean;
  dispose: () => void;
  winningMoves: CellPosition[];
  winningCells: CellPosition[];
}

interface WasmThreatSnapshot {
  counterThreatMoves: Array<{ row: number; col: number }>;
  forbiddenMoves: Array<{ row: number; col: number }>;
  immediateThreatMoves: Array<{ row: number; col: number }>;
  imminentThreatMoves: Array<{ row: number; col: number }>;
  winningMoves: Array<{ row: number; col: number }>;
}

export interface LocalMatchStoreOptions {
  botDepth?: number;
  botRunner?: MatchBotRunner;
  boardFactory?: (variant: GameVariant) => WasmBoard;
  humanDisplayName?: string;
  nowMs?: () => number;
  onMatchFinished?: (match: FinishedLocalMatch) => void;
  botConfig?: BotConfig;
  resumeState?: LocalMatchResumeSeed;
  variant?: GameVariant;
}

export interface LocalMatchResumeSeed {
  currentPlayer: 1 | 2;
  moves: MatchMove[];
  undoFloor?: number;
  variant: GameVariant;
}

function withBotPlayerNames(
  players: [MatchPlayer, MatchPlayer],
  botConfig: BotConfig,
): [MatchPlayer, MatchPlayer] {
  return players.map((player) =>
    player.kind === "bot"
      ? { ...player, name: botPlayerName(botConfig) }
      : { ...player },
  ) as [MatchPlayer, MatchPlayer];
}

function defaultPlayers(
  humanDisplayName = "You",
  botConfig: BotConfig = DEFAULT_BOT_CONFIG,
): [MatchPlayer, MatchPlayer] {
  return [
    { kind: "human", name: humanDisplayName, stone: "black" },
    { kind: "bot", name: botPlayerName(botConfig), stone: "white" },
  ];
}

function clonePlayers(players: [MatchPlayer, MatchPlayer]): [MatchPlayer, MatchPlayer] {
  return [{ ...players[0] }, { ...players[1] }];
}

function cloneResumeMoves(moves: MatchMove[]): MatchMove[] {
  return moves.map((move) => ({ ...move }));
}

function normalizeUndoFloor(undoFloor: number | undefined, moveCount: number): number {
  const fallback = undoFloor ?? moveCount;
  if (!Number.isFinite(fallback)) {
    return moveCount;
  }

  return Math.max(0, Math.min(moveCount, Math.floor(fallback)));
}

function swapPlayers(players: [MatchPlayer, MatchPlayer]): [MatchPlayer, MatchPlayer] {
  return [
    { ...players[1], stone: "black" },
    { ...players[0], stone: "white" },
  ];
}

function emptyCells(): CellStone[][] {
  return Array.from({ length: BOARD_SIZE }, () =>
    Array.from({ length: BOARD_SIZE }, () => null),
  );
}

function resumedPlayers(
  currentPlayer: 1 | 2,
  humanDisplayName = "You",
  botConfig: BotConfig = DEFAULT_BOT_CONFIG,
): [MatchPlayer, MatchPlayer] {
  const base = defaultPlayers(humanDisplayName, botConfig);
  return currentPlayer === 1 ? base : swapPlayers(base);
}

function seedBoard(board: WasmBoard, moves: MatchMove[]): boolean {
  for (const move of moves) {
    const result = board.applyMove(move.row, move.col) as { error?: unknown };
    if (result?.error) {
      return false;
    }
  }

  return true;
}

function cellsFromBoard(board: WasmBoard): CellStone[][] {
  const cells = emptyCells();

  for (let row = 0; row < BOARD_SIZE; row += 1) {
    for (let col = 0; col < BOARD_SIZE; col += 1) {
      const cell = board.cell(row, col);
      if (cell === 1) {
        cells[row][col] = 0;
      } else if (cell === 2) {
        cells[row][col] = 1;
      }
    }
  }

  return cells;
}

function statusFromResult(result: string): MatchStatus {
  switch (result) {
    case "black":
      return "black_won";
    case "white":
      return "white_won";
    case "draw":
      return "draw";
    default:
      return "playing";
  }
}

function normalizeMoves(moves: Array<{ row: number; col: number }>): CellPosition[] {
  return moves.map((move) => ({ row: move.row, col: move.col }));
}

function deriveHumanHints(
  board: WasmBoard,
  pendingBotMove: boolean,
  players: [MatchPlayer, MatchPlayer],
  status: MatchStatus,
): Pick<
  LocalMatchState,
  "counterThreatMoves" | "forbiddenMoves" | "imminentThreatMoves" | "threatMoves" | "winningMoves"
> {
  if (status !== "playing" || pendingBotMove) {
    return {
      counterThreatMoves: [],
      forbiddenMoves: [],
      imminentThreatMoves: [],
      threatMoves: [],
      winningMoves: [],
    };
  }

  const currentPlayer = board.currentPlayer() as 1 | 2;
  const currentIndex = (currentPlayer - 1) as 0 | 1;

  if (players[currentIndex].kind !== "human") {
    return {
      counterThreatMoves: [],
      forbiddenMoves: [],
      imminentThreatMoves: [],
      threatMoves: [],
      winningMoves: [],
    };
  }

  const threatSnapshot = board.threatSnapshot() as WasmThreatSnapshot;

  return {
    counterThreatMoves: normalizeMoves(threatSnapshot.counterThreatMoves),
    forbiddenMoves: normalizeMoves(threatSnapshot.forbiddenMoves),
    imminentThreatMoves: normalizeMoves(threatSnapshot.imminentThreatMoves),
    threatMoves: normalizeMoves(threatSnapshot.immediateThreatMoves),
    winningMoves: normalizeMoves(threatSnapshot.winningMoves),
  };
}

function snapshotState(
  board: WasmBoard,
  moves: MatchMove[],
  pendingBotMove: boolean,
  players: [MatchPlayer, MatchPlayer],
  currentVariant: GameVariant,
  selectedVariant: GameVariant,
  currentBotConfig: BotConfig,
  selectedBotConfig: BotConfig,
  playerClockMs: [number, number],
  turnStartedAtMs: number,
  undoFloor: number,
): Omit<
  LocalMatchState,
  | "dispose"
  | "placeHumanMove"
  | "selectBotConfig"
  | "selectVariant"
  | "startNewMatch"
  | "startNextRound"
  | "undoLastTurn"
> {
  const lastMove = moves.length > 0 ? moves[moves.length - 1] : null;
  const status = statusFromResult(board.result());
  const hints = deriveHumanHints(board, pendingBotMove, players, status);

  return {
    cells: cellsFromBoard(board),
    counterThreatMoves: hints.counterThreatMoves,
    currentPlayer: board.currentPlayer() as 1 | 2,
    currentBotConfig,
    currentVariant,
    forbiddenMoves: hints.forbiddenMoves,
    imminentThreatMoves: hints.imminentThreatMoves,
    lastMove,
    moves,
    pendingBotMove,
    players,
    playerClockMs: [...playerClockMs],
    selectedBotConfig,
    selectedVariant,
    status,
    threatMoves: hints.threatMoves,
    turnStartedAtMs,
    undoFloor,
    winningMoves: hints.winningMoves,
    winningCells: normalizeMoves(board.winningCells() as Array<{ row: number; col: number }>),
  };
}

function snapshotFinishedMatch(
  board: WasmBoard,
  moves: MatchMove[],
  players: [MatchPlayer, MatchPlayer],
  botConfig: BotConfig,
  variant: GameVariant,
  undoFloor: number,
): FinishedLocalMatch | null {
  const snapshot = snapshotState(
    board,
    moves,
    false,
    players,
    variant,
    variant,
    botConfig,
    botConfig,
    [0, 0],
    0,
    undoFloor,
  );
  if (snapshot.status === "playing") {
    return null;
  }

  return {
    mode: "bot",
    moves,
    players,
    botConfig,
    status: snapshot.status,
    undoFloor,
    variant,
    winningCells: snapshot.winningCells,
  };
}

export function createLocalMatchStore(
  options: LocalMatchStoreOptions = {},
): StoreApi<LocalMatchState> {
  const initialResumeState = options.resumeState
    ? {
        ...options.resumeState,
        moves: cloneResumeMoves(options.resumeState.moves),
      }
    : null;
  let currentVariant = initialResumeState?.variant ?? options.variant ?? "freestyle";
  let selectedVariant = currentVariant;
  let currentBotConfig = sanitizeBotConfig(options.botConfig);
  let selectedBotConfig = currentBotConfig;
  const boardFactory = options.boardFactory ?? WasmBoard.createWithVariant;
  const botRunner = options.botRunner ?? new BotRunner();
  const nowMs = options.nowMs ?? (() => Date.now());
  let players = initialResumeState
    ? resumedPlayers(initialResumeState.currentPlayer, options.humanDisplayName, currentBotConfig)
    : defaultPlayers(options.humanDisplayName, currentBotConfig);

  let board = boardFactory(currentVariant);
  const seededMoves = initialResumeState && seedBoard(board, initialResumeState.moves) ? initialResumeState.moves : [];
  let undoFloor = initialResumeState ? normalizeUndoFloor(initialResumeState.undoFloor, seededMoves.length) : 0;
  if (initialResumeState && seededMoves.length !== initialResumeState.moves.length) {
    board.free();
    board = boardFactory(currentVariant);
    players = defaultPlayers(options.humanDisplayName, currentBotConfig);
    undoFloor = 0;
  }
  let playerClockMs: [number, number] = [0, 0];
  let moveClockMs: number[] = seededMoves.map(() => 0);
  let turnStartedAtMs = nowMs();
  let requestToken = 0;

  const store = createStore<LocalMatchState>((set, get) => {
    const interruptBotRequests = (): void => {
      requestToken += 1;
      botRunner.cancelPending?.();
    };

    const configureBots = (): void => {
      const botSpec = options.botDepth === undefined
        ? resolveBotConfig(currentBotConfig)
        : { kind: "baseline", depth: options.botDepth } satisfies BotSpec;

      botRunner.configure(
        players.map((player) =>
          player.kind === "human"
            ? { kind: "human" }
            : botSpec,
        ) as [BotSpec, BotSpec],
      );
    };

    const updateState = (nextMoves: MatchMove[], pendingBotMove: boolean): void => {
      set(snapshotState(
        board,
        nextMoves,
        pendingBotMove,
        players,
        currentVariant,
        selectedVariant,
        currentBotConfig,
        selectedBotConfig,
        playerClockMs,
        turnStartedAtMs,
        undoFloor,
      ));
    };

    const currentPlayerSlot = (): 0 | 1 => ((board.currentPlayer() as 1 | 2) - 1) as 0 | 1;

    const resetClocks = (): void => {
      playerClockMs = [0, 0];
      moveClockMs = [];
      turnStartedAtMs = nowMs();
    };

    const settleMoveClock = (player: 1 | 2): void => {
      const nextNow = nowMs();
      const elapsedMs = Math.max(0, Math.floor(nextNow - turnStartedAtMs));
      const slot = (player - 1) as 0 | 1;
      playerClockMs = playerClockMs.map((value, index) =>
        index === slot ? value + elapsedMs : value,
      ) as [number, number];
      moveClockMs = [...moveClockMs, elapsedMs];
      turnStartedAtMs = nextNow;
    };

    const undoMoveClock = (move: MatchMove | undefined): void => {
      if (!move) {
        return;
      }

      const elapsedMs = moveClockMs[moveClockMs.length - 1] ?? 0;
      const slot = (move.player - 1) as 0 | 1;
      moveClockMs = moveClockMs.slice(0, -1);
      playerClockMs = playerClockMs.map((value, index) =>
        index === slot ? Math.max(0, value - elapsedMs) : value,
      ) as [number, number];
    };

    const applyMove = (row: number, col: number, player: 1 | 2): boolean => {
      const result = board.applyMove(row, col) as { error?: unknown };

      if (result?.error) {
        return false;
      }

      settleMoveClock(player);

      const nextMoves = [
        ...get().moves,
        {
          col,
          moveNumber: get().moves.length + 1,
          player,
          row,
        },
      ];

      updateState(nextMoves, false);

      const finishedMatch = snapshotFinishedMatch(
        board,
        nextMoves,
        players,
        currentBotConfig,
        currentVariant,
        undoFloor,
      );
      if (finishedMatch) {
        options.onMatchFinished?.(finishedMatch);
      }

      return true;
    };

    const queueBotMove = async (slot: 0 | 1): Promise<void> => {
      const activeToken = ++requestToken;

      updateState(get().moves, true);

      try {
        const move = await botRunner.chooseMove(slot, currentVariant, board.toFen());

        if (activeToken !== requestToken) {
          return;
        }

        if (!move || !board.isLegal(move.row, move.col)) {
          updateState(get().moves, false);
          return;
        }

        applyMove(move.row, move.col, (slot + 1) as 1 | 2);
      } catch (error) {
        if (activeToken === requestToken) {
          console.error("[local_match_store] bot move failed", error);
          updateState(get().moves, false);
        }
      }
    };

    const maybeQueueBotMove = (): void => {
      if (board.result() !== "ongoing") {
        return;
      }

      const slot = currentPlayerSlot();
      if (players[slot].kind !== "bot") {
        return;
      }

      void queueBotMove(slot);
    };

    const minimumRetainedMoveCount = (): number =>
      Math.max(undoFloor, players[0].kind === "bot" ? 1 : 0);

    const resetMatch = (
      nextPlayers: [MatchPlayer, MatchPlayer],
      nextVariant = selectedVariant,
      nextBot = selectedBotConfig,
    ): void => {
      interruptBotRequests();
      board.free();
      currentVariant = nextVariant;
      currentBotConfig = nextBot;
      board = boardFactory(currentVariant);
      players = withBotPlayerNames(clonePlayers(nextPlayers), currentBotConfig);
      undoFloor = 0;
      resetClocks();
      configureBots();
      updateState([], false);
      maybeQueueBotMove();
    };

    configureBots();

    return {
      ...snapshotState(
        board,
        seededMoves,
        false,
        players,
        currentVariant,
        selectedVariant,
        currentBotConfig,
        selectedBotConfig,
        playerClockMs,
        turnStartedAtMs,
        undoFloor,
      ),
      dispose: () => {
        interruptBotRequests();
        botRunner.dispose();
        board.free();
      },
      placeHumanMove: (row: number, col: number) => {
        const state = get();
        const player = board.currentPlayer() as 1 | 2;
        const slot = (player - 1) as 0 | 1;

        if (state.pendingBotMove || state.status !== "playing" || players[slot].kind !== "human") {
          return false;
        }
        if (!board.isLegal(row, col)) {
          return false;
        }

        const moved = applyMove(row, col, player);

        if (moved) {
          maybeQueueBotMove();
        }

        return moved;
      },
      selectVariant: (variant) => {
        selectedVariant = variant;

        if (get().moves.length === 0) {
          resetMatch(players, variant);
          return;
        }

        updateState(get().moves, get().pendingBotMove);
      },
      selectBotConfig: (botConfig) => {
        selectedBotConfig = sanitizeBotConfig(botConfig);

        if (get().moves.length === 0) {
          resetMatch(players, selectedVariant, selectedBotConfig);
          return;
        }

        updateState(get().moves, get().pendingBotMove);
      },
      startNewMatch: () => {
        resetMatch(players);
      },
      startNextRound: () => {
        resetMatch(swapPlayers(players));
      },
      undoLastTurn: () => {
        const state = get();
        const minimumMoves = minimumRetainedMoveCount();
        if (state.moves.length <= minimumMoves) {
          return false;
        }

        interruptBotRequests();

        const nextMoves = [...state.moves];
        do {
          board.undoLastMove();
          undoMoveClock(nextMoves.pop());
        } while (
          nextMoves.length > minimumMoves &&
          players[((board.currentPlayer() as 1 | 2) - 1) as 0 | 1]?.kind === "bot"
        );

        turnStartedAtMs = nowMs();
        updateState(nextMoves, false);
        return true;
      },
    };
  });

  return store;
}
