export type CellStone = 0 | 1 | null;

export interface CellPosition {
  row: number;
  col: number;
}

export interface MatchMove extends CellPosition {
  moveNumber: number;
  player: 1 | 2;
}

export interface MatchPlayer {
  kind: "human" | "bot";
  name: string;
  stone: "black" | "white";
}

export type MatchStatus = "playing" | "black_won" | "white_won" | "draw";
