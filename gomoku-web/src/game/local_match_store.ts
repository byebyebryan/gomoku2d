import { createStore, type StoreApi } from "zustand/vanilla";

import { BOARD_SIZE, WIN_LENGTH } from "../board/constants";
import { BotRunner } from "../core/bot_runner";
import type { BotMove, BotSpec, GameVariant } from "../core/bot_protocol";
import { WasmBoard } from "../core/wasm_bridge";

import type { CellPosition, CellStone, MatchMove, MatchPlayer, MatchStatus } from "./types";

type MatchBotRunner = Pick<BotRunner, "chooseMove" | "configure" | "dispose">;

export interface LocalMatchState {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  lastMove: CellPosition | null;
  moves: MatchMove[];
  pendingBotMove: boolean;
  players: [MatchPlayer, MatchPlayer];
  startNewMatch: () => void;
  status: MatchStatus;
  placeHumanMove: (row: number, col: number) => boolean;
  dispose: () => void;
  winningCells: CellPosition[];
}

export interface LocalMatchStoreOptions {
  botDepth?: number;
  botRunner?: MatchBotRunner;
  boardFactory?: (variant: GameVariant) => WasmBoard;
  variant?: GameVariant;
}

const DEFAULT_PLAYERS: [MatchPlayer, MatchPlayer] = [
  { kind: "human", name: "You", stone: "black" },
  { kind: "bot", name: "Search Bot", stone: "white" },
];

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

function snapshotState(
  board: WasmBoard,
  moves: MatchMove[],
  pendingBotMove: boolean,
  players: [MatchPlayer, MatchPlayer],
): Omit<LocalMatchState, "dispose" | "placeHumanMove" | "startNewMatch"> {
  const lastMove = moves.length > 0 ? moves[moves.length - 1] : null;
  const status = statusFromResult(board.result());
  const winner =
    status === "black_won" ? 1 : status === "white_won" ? 2 : null;

  return {
    cells: cellsFromBoard(board),
    currentPlayer: board.currentPlayer() as 1 | 2,
    lastMove,
    moves,
    pendingBotMove,
    players,
    status,
    winningCells: winner ? findWinningCells(board, lastMove, winner) : [],
  };
}

export function createLocalMatchStore(
  options: LocalMatchStoreOptions = {},
): StoreApi<LocalMatchState> {
  const variant = options.variant ?? "freestyle";
  const botDepth = options.botDepth ?? 3;
  const boardFactory = options.boardFactory ?? WasmBoard.createWithVariant;
  const botRunner = options.botRunner ?? new BotRunner();
  const players = DEFAULT_PLAYERS;

  let board = boardFactory(variant);
  let requestToken = 0;

  botRunner.configure([
    { kind: "human" },
    { kind: "baseline", depth: botDepth },
  ] satisfies [BotSpec, BotSpec]);

  const store = createStore<LocalMatchState>((set, get) => {
    const updateState = (nextMoves: MatchMove[], pendingBotMove: boolean): void => {
      set(snapshotState(board, nextMoves, pendingBotMove, players));
    };

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
      return true;
    };

    const queueBotMove = async (): Promise<void> => {
      const activeToken = ++requestToken;

      updateState(get().moves, true);

      try {
        const move = await botRunner.chooseMove(1, variant, board.toFen());

        if (activeToken !== requestToken) {
          return;
        }

        if (!move || !board.isLegal(move.row, move.col)) {
          updateState(get().moves, false);
          return;
        }

        applyMove(move.row, move.col, 2);
      } catch (error) {
        console.error("[local_match_store] bot move failed", error);
        if (activeToken === requestToken) {
          updateState(get().moves, false);
        }
      }
    };

    return {
      ...snapshotState(board, [], false, players),
      dispose: () => {
        requestToken += 1;
        botRunner.dispose();
        board.free();
      },
      placeHumanMove: (row: number, col: number) => {
        const state = get();

        if (state.pendingBotMove || state.status !== "playing" || state.currentPlayer !== 1) {
          return false;
        }
        if (!board.isLegal(row, col)) {
          return false;
        }

        const moved = applyMove(row, col, 1);

        if (moved && board.result() === "ongoing" && board.currentPlayer() === 2) {
          void queueBotMove();
        }

        return moved;
      },
      startNewMatch: () => {
        requestToken += 1;
        board.free();
        board = boardFactory(variant);
        updateState([], false);
      },
    };
  });

  return store;
}
