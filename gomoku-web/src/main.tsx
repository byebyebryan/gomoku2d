import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import { App } from "./app/App";
import { initWasm } from "./core/wasm_bridge";
import "./app/global.css";

await initWasm();

const root = document.getElementById("root");

if (!root) {
  throw new Error("missing #root mount");
}

ReactDOM.createRoot(root).render(
  <BrowserRouter>
    <App />
  </BrowserRouter>,
);
