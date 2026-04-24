import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import { App } from "./app/App";
import { loadBoardFonts } from "./board/sequence_font";
import { initWasm } from "./core/wasm_bridge";
import "./app/global.css";

function routerBasename(baseUrl: string): string | undefined {
  if (baseUrl === "/") {
    return undefined;
  }

  return baseUrl.endsWith("/") ? baseUrl.slice(0, -1) : baseUrl;
}

await initWasm();
await loadBoardFonts();

const root = document.getElementById("root");

if (!root) {
  throw new Error("missing #root mount");
}

ReactDOM.createRoot(root).render(
  <BrowserRouter basename={routerBasename(import.meta.env.BASE_URL)}>
    <App />
  </BrowserRouter>,
);
