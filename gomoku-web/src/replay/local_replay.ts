import { BOARD_SIZE } from "../board/constants";
import type { GameVariant } from "../core/bot_protocol";
import type { CellPosition, CellStone, MatchMove, MatchPlayer, MatchStatus } from "../game/types";
import type { GuestSavedMatch } from "../profile/guest_profile_store";

export interface LocalReplayFrame {
  cells: CellStone[][];
  currentPlayer: 1 | 2;
  lastMove: MatchMove | null;
  moveIndex: number;
  moves: MatchMove[];
  status: MatchStatus;
  winningCells: CellPosition[];
}

export function shouldShowReplaySequenceNumbers(frame: Pick<LocalReplayFrame, "status">): boolean {
  return frame.status !== "playing";
}

function emptyCells(): CellStone[][] {
  return Array.from({ length: BOARD_SIZE }, () =>
    Array.from({ length: BOARD_SIZE }, () => null),
  );
}

function cloneMoves(moves: MatchMove[]): MatchMove[] {
  return moves.map((move) => ({ ...move }));
}

function cloneWinningCells(cells: CellPosition[]): CellPosition[] {
  return cells.map((cell) => ({ ...cell }));
}

function clampMoveIndex(moveIndex: number, max: number): number {
  return Math.max(0, Math.min(moveIndex, max));
}

export function replayPlayerName(player: MatchPlayer, guestDisplayName: string): string {
  return player.kind === "human" ? guestDisplayName : player.name;
}

export function variantLabel(variant: GameVariant): string {
  return variant === "renju" ? "Renju" : "Freestyle";
}

export function replayPlayerLabel(match: GuestSavedMatch, guestDisplayName: string): string {
  return match.players
    .map((player) => `${replayPlayerName(player, guestDisplayName)} (${player.stone})`)
    .join(" vs ");
}

export function replayWinnerLabel(match: GuestSavedMatch, guestDisplayName: string): string {
  if (match.status === "draw") {
    return "Draw";
  }

  const winningStone = match.status === "black_won" ? "black" : "white";
  const winner = match.players.find((player) => player.stone === winningStone);
  const winnerName = winner ? replayPlayerName(winner, guestDisplayName) : winningStone;

  return `${winnerName} wins`;
}

export function buildLocalReplayFrame(match: GuestSavedMatch, moveIndex: number): LocalReplayFrame {
  const clampedMoveIndex = clampMoveIndex(moveIndex, match.moves.length);
  const moves = cloneMoves(match.moves.slice(0, clampedMoveIndex));
  const cells = emptyCells();

  for (const move of moves) {
    cells[move.row][move.col] = move.player === 1 ? 0 : 1;
  }

  const lastMove = moves.length > 0 ? moves[moves.length - 1] : null;
  const atEnd = clampedMoveIndex === match.moves.length;

  return {
    cells,
    currentPlayer: clampedMoveIndex % 2 === 0 ? 1 : 2,
    lastMove,
    moveIndex: clampedMoveIndex,
    moves,
    status: atEnd ? match.status : "playing",
    winningCells: atEnd ? cloneWinningCells(match.winningCells) : [],
  };
}
