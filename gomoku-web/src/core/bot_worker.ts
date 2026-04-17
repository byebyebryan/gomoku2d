/// <reference lib="webworker" />

import { WasmBoard, WasmBot } from "gomoku-wasm";

import type { BotWorkerRequest, BotWorkerResponse } from "./bot_protocol";

const workerScope = self as DedicatedWorkerGlobalScope;

function postMessage(message: BotWorkerResponse): void {
  workerScope.postMessage(message);
}

workerScope.addEventListener("message", (event: MessageEvent<BotWorkerRequest>) => {
  const message = event.data;
  const { requestId, spec, variant, fen } = message;

  if (spec.kind === "human") {
    postMessage({ type: "move", requestId, move: null });
    return;
  }

  let bot: WasmBot | null = null;
  let board: WasmBoard | null = null;

  try {
    bot = WasmBot.createBaseline(spec.depth);
    board = WasmBoard.fromFenWithVariant(fen, variant);
    const move = bot.chooseMove(board) as { row: number; col: number } | null;

    postMessage({
      type: "move",
      requestId,
      move: move ? { row: move.row, col: move.col } : null,
    });
  } catch (error) {
    postMessage({
      type: "error",
      requestId,
      message: error instanceof Error ? error.message : String(error),
    });
  } finally {
    bot?.free();
    board?.free();
  }
});
