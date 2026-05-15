/// <reference lib="webworker" />

import { WasmBoard, WasmBot } from "gomoku-wasm";

import type { BotSpec, BotWorkerRequest, BotWorkerResponse } from "./bot_protocol";

const workerScope = self as DedicatedWorkerGlobalScope;
let bots: [WasmBot | null, WasmBot | null] = [null, null];

function postMessage(message: BotWorkerResponse): void {
  workerScope.postMessage(message);
}

function buildBot(spec: BotSpec): WasmBot | null {
  switch (spec.kind) {
    case "human":
      return null;
    case "baseline":
      return WasmBot.createBaseline(spec.depth);
    case "search":
      return WasmBot.createSearch(
        spec.depth,
        spec.childLimit ?? 0,
        spec.patternEval,
        spec.corridorProof?.depth ?? 0,
        spec.corridorProof?.width ?? 0,
        spec.corridorProof?.candidateLimit ?? 0,
      );
  }
}

function configure(specs: [BotSpec, BotSpec]): void {
  bots[0]?.free();
  bots[1]?.free();
  bots = [buildBot(specs[0]), buildBot(specs[1])];
}

function handleChooseMove(message: Extract<BotWorkerRequest, { type: "choose_move" }>): void {
  const bot = bots[message.slot];
  if (!bot) {
    postMessage({ type: "move", requestId: message.requestId, move: null });
    return;
  }

  let board: WasmBoard | null = null;

  try {
    board = WasmBoard.fromFenWithVariant(message.fen, message.variant);
    const move = bot.chooseMove(board) as { row: number; col: number } | null;

    postMessage({
      type: "move",
      requestId: message.requestId,
      move: move ? { row: move.row, col: move.col } : null,
    });
  } catch (error) {
    postMessage({
      type: "error",
      requestId: message.requestId,
      message: error instanceof Error ? error.message : String(error),
    });
  } finally {
    board?.free();
  }
}

workerScope.addEventListener("message", (event: MessageEvent<BotWorkerRequest>) => {
  const message = event.data;

  switch (message.type) {
    case "configure":
      configure(message.specs);
      break;
    case "choose_move":
      handleChooseMove(message);
      break;
  }
});

postMessage({ type: "ready" });
