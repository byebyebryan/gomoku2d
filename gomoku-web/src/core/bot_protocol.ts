export type GameVariant = "freestyle" | "renju";

export type BotSpec =
  | { kind: "human" }
  | { kind: "baseline"; depth: number };

export interface BotMove {
  row: number;
  col: number;
}

export type BotWorkerRequest = {
  type: "choose_move";
  requestId: number;
  spec: BotSpec;
  variant: GameVariant;
  fen: string;
};

export type BotWorkerResponse =
  | {
      type: "move";
      requestId: number;
      move: BotMove | null;
    }
  | {
      type: "error";
      requestId?: number;
      message: string;
    };
