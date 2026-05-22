/// <reference lib="webworker" />

import type { BotSpec, BotWorkerRequest, BotWorkerResponse } from "./bot_protocol";
import {
  chooseWasmBotMove,
  createWasmBotFromSpec,
  wasmBoardFromFenWithVariant,
  type WasmBoard,
  type WasmBot,
} from "./wasm_bridge";

const workerScope = self as DedicatedWorkerGlobalScope;
let bots: [WasmBot | null, WasmBot | null] = [null, null];

function postMessage(message: BotWorkerResponse): void {
  workerScope.postMessage(message);
}

function buildBot(spec: BotSpec): WasmBot | null {
  return createWasmBotFromSpec(spec);
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
    board = wasmBoardFromFenWithVariant(message.fen, message.variant);
    const move = chooseWasmBotMove(bot, board);

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
