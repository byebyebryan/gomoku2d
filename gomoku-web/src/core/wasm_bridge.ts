import init, { WasmBoard, WasmBot } from "gomoku-wasm";

let initialized = false;
let initPromise: Promise<void> | null = null;

export async function initWasm(): Promise<void> {
  if (initialized) return;
  if (initPromise) return initPromise;

  initPromise = (async () => {
    await init("/assets/gomoku_wasm_bg.wasm");
    initialized = true;
  })();

  return initPromise;
}

export { WasmBoard, WasmBot };
