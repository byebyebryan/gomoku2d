const params = new URLSearchParams(location.search);
const state = { layout: ["split","a","b","c","current"].includes(params.get("layout")) ? params.get("layout") : "split", surface: "visitor", sidebar: "card", scenario: "full", stats: "on", labels: "normal", viewport: "desktop", text: "100", thumbs: "off" };
const controls = ["surface", "sidebar", "scenario", "stats", "labels", "viewport", "text", "thumbs"];
const descriptions = {
  split: "Refined · Two columns — a public player card on the left; A's shared record above the game list on the right. Compare the player card with the earlier minimal left panel.",
  a: "A · Wide record — the player's name and familiar app navigation lead; a restrained record strip sits above a full-width game list.",
  b: "B · Profile columns — the private Profile screen's identity/record rail becomes a public reader view, with published games beside it.",
  c: "C · Compact record — a single contained panel brings name, record and games together; navigation stays visible above it.",
  current: "Current structure — source-based 0.6.1 reconstruction with the same fixtures. Narrow card, separate stat boxes and disclosure before the games."
};
const frame = document.querySelector("#prototype");
const stage = document.querySelector("#stage");
function fit() {
  const width = state.viewport === "desktop" ? 1280 : state.viewport === "mobile" ? 390 : 320;
  const height = state.viewport === "desktop" ? 900 : 1000;
  const scale = Math.min(1, stage.parentElement.clientWidth / width - (state.viewport === "desktop" ? 0.02 : 0));
  const safe = Math.max(0.15, scale);
  frame.style.width = width + "px"; frame.style.height = height + "px"; frame.style.transform = "scale(" + safe + ")";
  stage.style.width = (width * safe) + "px"; stage.style.height = (height * safe) + "px";
}
function syncControls() {
  for (const key of controls) document.getElementById(key).value = state[key];
  document.getElementById("sidebar").disabled = state.layout !== "split" || state.surface !== "visitor";
  for (const button of document.querySelectorAll("[data-layout]")) button.setAttribute("aria-pressed", String(button.dataset.layout === state.layout));
  document.getElementById("option-description").textContent = state.surface === "owner"
    ? "Shared owner companion — General Settings-style rows inside the existing Profile shell. Visitor layouts remain alternative public views. Actions are in-memory demonstrations."
    : descriptions[state.layout];
}
function update() {
  syncControls();
  const query = new URLSearchParams(state);
  document.getElementById("full-size").href = "prototype.html?" + query.toString();
  query.set("embed", "1");
  frame.src = "prototype.html?" + query.toString();
  fit();
}
for (const key of controls) document.getElementById(key).addEventListener("change", (event) => { state[key] = event.target.value; update(); });
for (const button of document.querySelectorAll("[data-layout]")) button.addEventListener("click", () => { state.layout = button.dataset.layout; update(); });
window.addEventListener("message", (event) => {
  if (event.origin !== location.origin || event.source !== frame.contentWindow || event.data?.type !== "gomoku-study-surface") return;
  if (["owner", "visitor"].includes(event.data.surface)) { state.surface = event.data.surface; syncControls(); }
});
let lastWidth = -1;
let fitFrame;
new ResizeObserver((entries) => {
  const width = entries[0].contentRect.width;
  if (Math.abs(width - lastWidth) < 0.5) return;
  lastWidth = width;
  cancelAnimationFrame(fitFrame);
  fitFrame = requestAnimationFrame(fit);
}).observe(stage.parentElement);
update();
