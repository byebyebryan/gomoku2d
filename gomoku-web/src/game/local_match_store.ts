import { createStore, type StoreApi } from "zustand/vanilla";

import { BOARD_SIZE, WIN_LENGTH } from "../board/constants";
import { BotRunner } from "../core/bot_runner";
import type { BotSpec, GameVariant } from "../core/bot_protocol";
import { WasmBoard } from "../core/wasm_bridge";

import type { CellPosition, CellStone, MatchMove, MatchPlayer, MatchStatus } from "./types";

type MatchBotRunner = Pick<BotRunner, "chooseMove" | "configure" | "dispose"> &
  Partial<Pick<BotRunner, "cancelPending">>;

interface FinishedLocalMatch {
  mode: "bot";
  moves: MatchMove[];
  players: [MatchPlayer, MatchPlayer];
  status: Exclude<MatchStatus, "playing">;
  variant: GameVariant;
  winningCells: CellPosition[];
}

export interface LocalMatchState {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  currentVariant: GameVariant;
  forbiddenMoves: CellPosition[];
  lastMove: CellPosition | null;
  moves: MatchMove[];
  pendingBotMove: boolean;
  players: [MatchPlayer, MatchPlayer];
  selectedVariant: GameVariant;
  selectVariant: (variant: GameVariant) => void;
  startNewMatch: () => void;
  startNextRound: () => void;
  status: MatchStatus;
  threatMoves: CellPosition[];
  undoLastTurn: () => boolean;
  placeHumanMove: (row: number, col: number) => boolean;
  dispose: () => void;
  winningMoves: CellPosition[];
  winningCells: CellPosition[];
}

export interface LocalMatchStoreOptions {
  botDepth?: number;
  botRunner?: MatchBotRunner;
  boardFactory?: (variant: GameVariant) => WasmBoard;
  humanDisplayName?: string;
  onMatchFinished?: (match: FinishedLocalMatch) => void;
  variant?: GameVariant;
}

function defaultPlayers(humanDisplayName = "You"): [MatchPlayer, MatchPlayer] {
  return [
    { kind: "human", name: humanDisplayName, stone: "black" },
    { kind: "bot", name: "Classic Bot", stone: "white" },
  ];
}

function clonePlayers(players: [MatchPlayer, MatchPlayer]): [MatchPlayer, MatchPlayer] {
  return [{ ...players[0] }, { ...players[1] }];
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

function findWinningCells(
  board: WasmBoard,
  lastMove: CellPosition | null,
  winner: 1 | 2,
): CellPosition[] {
  if (!lastMove) {
    return [];
  }

  const directions = [
    { dr: 0, dc: 1 },
    { dr: 1, dc: 0 },
    { dr: 1, dc: 1 },
    { dr: 1, dc: -1 },
  ];

  for (const { dr, dc } of directions) {
    const cells: CellPosition[] = [{ row: lastMove.row, col: lastMove.col }];

    for (let step = 1; step < WIN_LENGTH; step += 1) {
      const row = lastMove.row + dr * step;
      const col = lastMove.col + dc * step;

      if (row < 0 || row >= BOARD_SIZE || col < 0 || col >= BOARD_SIZE) {
        break;
      }
      if (board.cell(row, col) !== winner) {
        break;
      }

      cells.push({ row, col });
    }

    for (let step = 1; step < WIN_LENGTH; step += 1) {
      const row = lastMove.row - dr * step;
      const col = lastMove.col - dc * step;

      if (row < 0 || row >= BOARD_SIZE || col < 0 || col >= BOARD_SIZE) {
        break;
      }
      if (board.cell(row, col) !== winner) {
        break;
      }

      cells.push({ row, col });
    }

    if (cells.length >= WIN_LENGTH) {
      return cells;
    }
  }

  return [];
}

function normalizeMoves(moves: Array<{ row: number; col: number }>): CellPosition[] {
  return moves.map((move) => ({ row: move.row, col: move.col }));
}

function deriveHumanHints(
  board: WasmBoard,
  pendingBotMove: boolean,
  players: [MatchPlayer, MatchPlayer],
  status: MatchStatus,
): Pick<LocalMatchState, "forbiddenMoves" | "threatMoves" | "winningMoves"> {
  if (status !== "playing" || pendingBotMove) {
    return {
      forbiddenMoves: [],
      threatMoves: [],
      winningMoves: [],
    };
  }

  const currentPlayer = board.currentPlayer() as 1 | 2;
  const currentIndex = (currentPlayer - 1) as 0 | 1;

  if (players[currentIndex].kind !== "human") {
    return {
      forbiddenMoves: [],
      threatMoves: [],
      winningMoves: [],
    };
  }

  const winningMoves = normalizeMoves(
    board.immediateWinningMovesFor(currentPlayer) as Array<{ row: number; col: number }>,
  );
  const winningKeys = new Set(winningMoves.map((move) => `${move.row},${move.col}`));
  const opponent = currentPlayer === 1 ? 2 : 1;
  const threatMoves = normalizeMoves(
    board.immediateWinningMovesFor(opponent) as Array<{ row: number; col: number }>,
  ).filter((move) => !winningKeys.has(`${move.row},${move.col}`));

  return {
    forbiddenMoves: normalizeMoves(
      board.forbiddenMovesForCurrentPlayer() as Array<{ row: number; col: number }>,
    ),
    threatMoves,
    winningMoves,
  };
}

function snapshotState(
  board: WasmBoard,
  moves: MatchMove[],
  pendingBotMove: boolean,
  players: [MatchPlayer, MatchPlayer],
  currentVariant: GameVariant,
  selectedVariant: GameVariant,
): Omit<
  LocalMatchState,
  "dispose" | "placeHumanMove" | "selectVariant" | "startNewMatch" | "startNextRound" | "undoLastTurn"
> {
  const lastMove = moves.length > 0 ? moves[moves.length - 1] : null;
  const status = statusFromResult(board.result());
  const winner =
    status === "black_won" ? 1 : status === "white_won" ? 2 : null;
  const hints = deriveHumanHints(board, pendingBotMove, players, status);

  return {
    cells: cellsFromBoard(board),
    currentPlayer: board.currentPlayer() as 1 | 2,
    currentVariant,
    forbiddenMoves: hints.forbiddenMoves,
    lastMove,
    moves,
    pendingBotMove,
    players,
    selectedVariant,
    status,
    threatMoves: hints.threatMoves,
    winningMoves: hints.winningMoves,
    winningCells: winner ? findWinningCells(board, lastMove, winner) : [],
  };
}

function snapshotFinishedMatch(
  board: WasmBoard,
  moves: MatchMove[],
  players: [MatchPlayer, MatchPlayer],
  variant: GameVariant,
): FinishedLocalMatch | null {
  const snapshot = snapshotState(board, moves, false, players, variant, variant);
  if (snapshot.status === "playing") {
    return null;
  }

  return {
    mode: "bot",
    moves,
    players,
    status: snapshot.status,
    variant,
    winningCells: snapshot.winningCells,
  };
}

export function createLocalMatchStore(
  options: LocalMatchStoreOptions = {},
): StoreApi<LocalMatchState> {
  let currentVariant = options.variant ?? "freestyle";
  let selectedVariant = currentVariant;
  const botDepth = options.botDepth ?? 3;
  const boardFactory = options.boardFactory ?? WasmBoard.createWithVariant;
  const botRunner = options.botRunner ?? new BotRunner();
  let players = defaultPlayers(options.humanDisplayName);

  let board = boardFactory(currentVariant);
  let requestToken = 0;

  const store = createStore<LocalMatchState>((set, get) => {
    const interruptBotRequests = (): void => {
      requestToken += 1;
      botRunner.cancelPending?.();
    };

    const configureBots = (): void => {
      botRunner.configure(
        players.map((player) =>
          player.kind === "human"
            ? { kind: "human" }
            : { kind: "baseline", depth: botDepth },
        ) as [BotSpec, BotSpec],
      );
    };

    const updateState = (nextMoves: MatchMove[], pendingBotMove: boolean): void => {
      set(snapshotState(board, nextMoves, pendingBotMove, players, currentVariant, selectedVariant));
    };

    const currentPlayerSlot = (): 0 | 1 => ((board.currentPlayer() as 1 | 2) - 1) as 0 | 1;

    const applyMove = (row: number, col: number, player: 1 | 2): boolean => {
      const result = board.applyMove(row, col) as { error?: unknown };

      if (result?.error) {
        return false;
      }

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

      const finishedMatch = snapshotFinishedMatch(board, nextMoves, players, currentVariant);
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

    const minimumRetainedMoveCount = (): number => (players[0].kind === "bot" ? 1 : 0);

    const resetMatch = (nextPlayers: [MatchPlayer, MatchPlayer], nextVariant = selectedVariant): void => {
      interruptBotRequests();
      board.free();
      currentVariant = nextVariant;
      board = boardFactory(currentVariant);
      players = clonePlayers(nextPlayers);
      configureBots();
      updateState([], false);
      maybeQueueBotMove();
    };

    configureBots();

    return {
      ...snapshotState(board, [], false, players, currentVariant, selectedVariant),
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
          nextMoves.pop();
        } while (
          nextMoves.length > minimumMoves &&
          players[((board.currentPlayer() as 1 | 2) - 1) as 0 | 1]?.kind === "bot"
        );

        updateState(nextMoves, false);
        return true;
      },
    };
  });

  return store;
}
