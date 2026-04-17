import { WasmBoard } from "gomoku-wasm";

export async function initWasm(): Promise<void> {
  // WASM auto-initializes via ES module import with wasm-pack bundler target
}

export { WasmBoard };
