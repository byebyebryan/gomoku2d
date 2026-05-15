export type GameVariant = "freestyle" | "renju";

export interface CorridorProofSpec {
  candidateLimit: number;
  depth: number;
  width: number;
}

export type BotSpec =
  | { kind: "human" }
  | { kind: "baseline"; depth: number }
  | {
    childLimit: number | null;
    corridorProof: CorridorProofSpec | null;
    depth: number;
    kind: "search";
    patternEval: boolean;
  };

export interface BotMove {
  row: number;
  col: number;
}

export type BotWorkerRequest =
  | { type: "configure"; specs: [BotSpec, BotSpec] }
  | {
      type: "choose_move";
      requestId: number;
      slot: 0 | 1;
      variant: GameVariant;
      fen: string;
    };

export type BotWorkerResponse =
  | { type: "ready" }
  | { type: "move"; requestId: number; move: BotMove | null }
  | { type: "error"; requestId?: number; message: string };
