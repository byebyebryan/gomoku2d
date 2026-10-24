const query = new URLSearchParams(location.search);
const pick = (key, choices, fallback) => choices.includes(query.get(key)) ? query.get(key) : fallback;
const state = {
  layout: pick("layout", ["split", "a", "b", "c", "current"], "split"),
  surface: pick("surface", ["visitor", "owner"], "visitor"),
  scenario: pick("scenario", ["full", "sparse", "empty", "loading", "error", "unavailable"], "full"),
  stats: query.get("stats") !== "off", long: query.get("labels") === "long",
  sidebar: query.get("sidebar") === "minimal" ? "minimal" : "card",
  thumbs: query.get("thumbs") === "on", text: pick("text", ["100", "150", "200"], "100"),
  page: 0, enabled: true, confirmDisable: false, confirmEnable: false, removed: new Set(), name: "Juniper"
};
const titles = ["The long way around", "One move too late", "A quiet reply", "Double threat", "Hold the line", "A second look", "No room to escape", "A different ending"];
const app = document.getElementById("app");
const dialog = document.getElementById("replay-dialog");
let icons = {};
let toastTimer;
let replayTrigger;
document.documentElement.style.fontSize = (16 * Number(state.text) / 100) + "px";
document.body.classList.toggle("large-text", state.text !== "100");
document.getElementById("study-chrome").hidden = query.has("embed");
const esc = (value) => String(value).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
const icon = (name) => '<span class="uiIcon" aria-hidden="true">' + (icons[name] || "") + "</span>";
const button = (label, name, action, tone = "Neutral", extra = "") => '<button type="button" class="uiAction uiAction' + tone + '" data-action="' + action + '" ' + extra + ">" + icon(name) + '<span class="uiActionLabel">' + esc(label) + "</span></button>";
function playerName() { return state.long ? "棋盤の旅人_" + "W".repeat(64) : state.name; }
function games() {
  const count = state.scenario === "empty" || !state.enabled ? 0 : state.scenario === "sparse" ? 1 : 32;
  return Array.from({length: count}, (_, i) => ({
    id: i, title: state.long ? "終盤の分岐_" + "W".repeat(60) : titles[i % titles.length] + (i >= 8 ? " · " + (i + 1) : ""),
    opponent: state.long ? "対戦相手_" + "W".repeat(64) : ["Normal Bot", "Hard Bot", "Easy Bot", "Rin"][i % 4],
    kind: i % 4 === 3 ? "human" : "bot", result: i === 3 ? "Draw" : ["Win", "Loss", "Win", "Win", "Win", "Loss", "Win", "Win"][i % 8],
    rule: i % 3 === 0 ? "Renju" : "Freestyle", moves: i === 3 ? 225 : [41, 58, 33, 49, 47, 62, 39, 52][i % 8]
  })).filter((entry) => !state.removed.has(entry.id));
}
function nav() {
  return '<nav class="app-nav" aria-label="App navigation">' +
    button("Play", "play", "nav-play", "Primary", 'aria-label="Play"') +
    button("My profile", "profile", "nav-profile", "Secondary", 'aria-label="My profile"') +
    button("Home", "home", "nav-home", "Neutral", 'aria-label="Home"') + "</nav>";
}
function header(title, eyebrow = "Gomoku2D · Public player", name = true) {
  return '<header class="page-header"><div class="header-copy"><p class="uiPageEyebrow">' + esc(eyebrow) +
    '</p><h1 class="page-title ' + (name ? "player-name" : "") + '" title="' + esc(title) + '">' + esc(title) + "</h1></div>" + nav() + "</header>";
}
function stats() {
  return '<dl class="stats" aria-label="Shared player statistics">' +
    [["Matches","38","matches"],["Wins","24","wins"],["Losses","13","losses"],["Draws","1","draws"],["Win rate","63.2%","rate"]].map(([label,value,cls]) =>
      '<div class="stat ' + cls + '"><dt>' + label + "</dt><dd>" + value + "</dd></div>").join("") + "</dl>";
}
function record() {
  if (!state.stats) return "";
  return '<section class="record" aria-labelledby="record-title"><div class="record-top"><h2 class="section-label" id="record-title">Shared record</h2><p class="date">Updated October 23, 2026</p></div>' + stats() + "</section>";
}
function note() {
  return '<details class="record-note"><summary>' + icon("help") + '<span>About these records</span></summary><p>Player names and game details are supplied by the publisher. Statistics are a player-shared snapshot; the games below are selected casual records. Neither is independently verified.</p></details>';
}
function boardSvg(size = 96) {
  let lines = "";
  for (let i = 0; i < 15; i++) {
    const n = 8 + i * 6;
    lines += '<path d="M8 ' + n + "H92 M" + n + ' 8V92"/>';
  }
  const prefix = [[7,7],[8,7],[7,6],[8,6],[7,5],[7,8],[8,5],[6,9]];
  let stones = prefix.map(([x,y],i) => '<rect x="' + (6 + x*6) + '" y="' + (6 + y*6) + '" width="5" height="5" fill="' + (i%2 ? "#fffdf4" : "#41413f") + '"/>').join("");
  return '<svg class="' + (size === 96 ? "thumbnail" : "preview-board") + '" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="' + size + '" height="' + size + '" aria-hidden="true" shape-rendering="crispEdges"><rect width="100" height="100" fill="#ffd069"/><g stroke="#282315" stroke-width=".5">' + lines + "</g>" + stones + "</svg>";
}
function row(entry) {
  if (state.layout === "current") return '<li class="game-row" data-game-id="' + entry.id + '"><span class="game-title">' + esc(entry.title) +
    '</span><span class="opponent">vs ' + esc(entry.opponent) + '</span><span class="result ' + entry.result.toLowerCase() + '">' + entry.result +
    '</span><span class="rule">' + entry.rule + '</span><span class="moves">' + entry.moves + ' moves</span><button type="button" class="uiAction uiActionSecondary replay-action" data-action="replay" data-entry="' + entry.id + '" aria-label="Replay ' + esc(entry.title) + '">Replay</button></li>';
  const copy = '<div class="copy-text"><p class="game-title" title="' + esc(entry.title) + '">' + esc(entry.title) +
    '</p><p class="opponent">' + icon(entry.kind) + '<span class="opponent-text" title="' + esc(entry.opponent) + '">vs ' + esc(entry.opponent) + "</span></p></div>";
  return '<li class="game-row" data-game-id="' + entry.id + '"><span class="result ' + entry.result.toLowerCase() + '"><span class="uiSrOnly">Result: </span>' + entry.result +
    '</span><div class="game-copy">' + (state.thumbs ? boardSvg() : "") + copy + '</div><span class="rule"><span class="uiSrOnly">Rule: </span>' + entry.rule +
    '</span><span class="moves">' + entry.moves + ' moves</span>' + button("Replay", "replay", "replay", "Secondary", 'aria-label="Replay ' + esc(entry.title) + '" data-entry="' + entry.id + '" style="min-width:0"').replace('class="uiAction ', 'class="replay-action uiAction ') + "</li>";
}
function paging(entries) {
  const count = Math.ceil(entries.length / 8);
  if (count <= 1) return "";
  return '<nav class="paging" aria-label="Published game pages">' +
    button("Prev", "prev", "previous", "Neutral", 'aria-label="Previous page" ' + (state.page === 0 ? 'class-placeholder="" tabindex="-1" aria-hidden="true"' : "")).replace('class="uiAction ', 'class="previous ' + (state.page === 0 ? "absent " : "") + 'uiAction ') +
    '<span class="page-indicator" aria-live="polite">' + (state.page + 1) + " / " + count + "</span>" +
    button("Next", "next", "next", "Neutral", 'aria-label="Next page" ' + (state.page === count - 1 ? 'tabindex="-1" aria-hidden="true"' : "")).replace('class="uiAction ', 'class="next ' + (state.page === count - 1 ? "absent " : "") + 'uiAction ') + "</nav>";
}
function catalog() {
  const entries = games();
  state.page = Math.max(0, Math.min(state.page, Math.max(0, Math.ceil(entries.length / 8) - 1)));
  return '<section class="catalog ' + (state.thumbs ? "with-thumbs" : "") + '" aria-labelledby="games-title"><div class="catalog-head"><div><h2 class="section-title" id="games-title">Published games</h2><p class="record-source">Player-shared · Unverified</p></div><span class="game-count">' + entries.length + " " + (entries.length === 1 ? "game" : "games") + "</span></div>" +
    (state.thumbs ? '<p class="thumbs-caption">Thumbnail composition sketch · 0.6.3</p>' : "") +
    (entries.length ? '<div class="game-head" aria-hidden="true"><span>Result</span><span>Game / opponent</span><span>Rule</span><span>Moves</span><span></span></div><ol class="games" aria-label="Published games, page ' + (state.page+1) + '">' + entries.slice(state.page*8,state.page*8+8).map(row).join("") + "</ol>" + paging(entries)
      : '<div class="empty"><h3>No games published yet</h3><p>This player has not shared any games.</p>' + button("Play a game", "play", "nav-play", "Primary") + "</div>") + "</section>";
}
function copyFooter() { return '<div class="public-footer">' + button("Copy profile link", "copy", "copy", "Neutral").replace('class="uiAction ', 'class="copy-link uiAction ') + "</div>"; }
function utilities() { return '<footer class="public-utilities">' + note() + copyFooter() + "</footer>"; }
function identityPanel(name) {
  const copy = button("Copy link","copy","copy","Neutral",'aria-label="Copy profile link"').replace('class="uiAction ', 'class="copy-link uiAction ');
  const identity = '<div class="identity-name"><p class="uiPageEyebrow">Player</p><h2 class="rail-name player-name" title="' + esc(name) + '">' + esc(name) + "</h2></div>";
  if (state.sidebar === "minimal") return '<aside class="identity-rail simple-identity" aria-label="Public player identity">' + identity + copy + "</aside>";
  const count = games().length;
  const open = innerWidth > 720 ? " open" : "";
  return '<aside class="identity-rail simple-identity player-card" aria-label="Public player identity"><div class="identity-core">' + identity +
    '<dl class="shared-count"><div><dt>' + (count === 1 ? "Shared game" : "Shared games") + "</dt><dd>" + count + "</dd></div></dl>" + copy +
    '</div><details class="identity-extra"' + open + '><summary>' + icon("profile") + '<span>Profile details</span></summary><div class="identity-extra-content"><section class="profile-address-section"><h3 class="section-label">Profile link</h3><p class="profile-address">gomoku2d.byebyebryan.com/player/?id=demo</p></section>' +
    note() + "</div></details></aside>";
}
function visitor() {
  const name = playerName();
  if (["loading","error","unavailable"].includes(state.scenario) || !state.enabled) return statusView();
  if (state.layout === "current") return '<div class="baseline"><section class="baseline-card"><p class="uiPageEyebrow">Gomoku 2D · Public profile</p><h1>' + esc(name) + "</h1>" + record() +
    '<p class="date">Games selected for public sharing</p><p class="baseline-disclosure">Player names and game details are supplied by the publisher. These games are casual records and are not independently verified.</p>' + catalog() + '<button type="button" class="uiAction uiActionNeutral" data-action="nav-home">Home</button></section></div>';
  if (state.layout === "split") return '<div class="app layout-split">' + header("Public profile", "Gomoku2D · Player record", false) +
    '<div class="columns">' + identityPanel(name) + '<div class="split-record">' + record() + catalog() + (state.sidebar === "minimal" ? note() : "") + "</div></div></div>";
  if (state.layout === "b") return '<div class="app layout-b">' + header("Public profile", "Gomoku2D · Player record", false) +
    '<div class="columns"><aside class="identity-rail"><div><p class="uiPageEyebrow">Player</p><h2 class="rail-name player-name" title="' + esc(name) + '">' + esc(name) + '</h2></div>' +
    (state.stats ? '<div class="rail-section">' + record() + "</div>" : "") + button("Copy link","copy","copy","Neutral").replace('class="uiAction ', 'class="copy-link uiAction ') +
    "</aside><div>" + catalog() + note() + "</div></div></div>";
  if (state.layout === "c") return '<div class="app layout-c"><header class="page-header"><p class="brand">GOMOKU2D</p>' + nav() + '</header><section class="compact-panel"><div class="compact-masthead"><div><p class="uiPageEyebrow">Public player</p><h1 class="page-title player-name" title="' + esc(name) + '">' + esc(name) +
    "</h1></div>" + record() + "</div>" + catalog() + utilities() + "</section></div>";
  return '<div class="app layout-a">' + header(name) + record() + catalog() + utilities() + "</div>";
}
function statusView() {
  const loading = state.scenario === "loading";
  const unavailable = state.scenario === "unavailable" || !state.enabled;
  const title = loading ? "Loading profile" : unavailable ? "Profile unavailable" : "Could not load profile";
  return '<div class="app layout-a">' + header("Public profile", "Gomoku2D · Player record", false) +
    '<section class="error-panel" aria-labelledby="status-title"' + (loading ? ' aria-busy="true"' : "") + '><h2 class="section-title" id="status-title">' + title +
    '</h2><p' + (!loading && !unavailable ? ' role="alert"' : ' role="status"') + ">" +
    (loading ? "Loading the shared record and published games." : unavailable ? "This public profile is no longer available." : "Check your connection and try again.") + "</p>" +
    (loading ? '<div aria-hidden="true"><div class="skeleton"></div><div class="skeleton"></div></div>' : !unavailable ? button("Try again","reset","retry","Secondary") : button("Play a game","play","nav-play","Primary")) + "</section></div>";
}
function setting(label, help, control, id) {
  return '<div class="setting-row"><div class="setting-copy">' + (id ? '<label for="' + id + '">' : "<strong>") + label + (id ? "</label>" : "</strong>") + '<p>' + help + '</p></div><div class="setting-control">' + control + "</div></div>";
}
function owner() {
  if (["loading","error","unavailable"].includes(state.scenario)) {
    const loading = state.scenario === "loading";
    return '<div class="app">' + header("Profile","Player record",false) + '<section class="error-panel"><h2 class="section-title">Public profile</h2><p role="' + (state.scenario === "error" ? "alert" : "status") + '">' +
      (loading ? "Loading your public profile." : state.scenario === "error" ? "Could not load your public profile. Check your connection and try again." : "Public sharing is currently unavailable.") +
      "</p>" + (loading ? '<div class="skeleton" aria-hidden="true"></div>' : button("Reload public profile","reset","retry","Secondary")) + "</section></div>";
  }
  const entries = games();
  const link = "https://gomoku2d.byebyebryan.com/player/?id=demo";
  let manager = '<section class="manager"><header class="manager-header"><div><p class="uiPageEyebrow">Optional sharing · ' + (state.enabled ? "Enabled" : "Off") +
    '</p><h2 class="section-title">Public profile</h2></div>' + button("Reload","reset","reload","Neutral",'aria-label="Reload public profile"') + '</header><section class="setting-section"><h3>Identity</h3>' +
    setting("Public name", "Shown to anyone with your profile link.", '<input id="public-name" class="uiInput" maxlength="160" value="' + esc(playerName()) + '">' +
      button("Save name","save","save-name","Primary"), "public-name");
  if (state.enabled) manager += setting("Profile link","Your public page and selected games.",'<input id="profile-link" class="uiInput owner-record-link" aria-label="Public profile link" readonly value="' + link + '"><div class="setting-actions">' +
    button("Copy link","copy","copy","Neutral") + button("View public profile","profile","view-public","Secondary") + "</div>");
  else manager += setting("Visibility","Starts empty. Statistics remain off until you opt in.",
    state.confirmEnable ? '<p class="hint">Enable a public profile with the name shown above?</p><div class="setting-actions">' + button("Enable profile","publish","confirm-enable","Primary") + button("Cancel","back","cancel-enable","Neutral") + "</div>" : button("Enable public profile","publish","enable","Primary"));
  manager += "</section>";
  if (state.enabled) manager += '<section class="setting-section"><h3>Public statistics</h3>' +
    setting("My games summary","An optional snapshot. Update it when you choose.",'<div class="segment-pair" role="group" aria-label="Share My games statistics publicly">' +
      '<button type="button" class="uiSegment ' + (!state.stats ? "uiSegmentActive" : "") + '" aria-pressed="' + !state.stats + '" data-action="stats-off">Off</button>' +
      '<button type="button" class="uiSegment ' + (state.stats ? "uiSegmentActive" : "") + '" aria-pressed="' + state.stats + '" data-action="stats-on">On</button></div>' +
      (state.stats ? '<div class="owner-stats">' + stats() + '</div><p class="hint">Published October 23, 2026</p>' + button("Update public stats","reset","refresh-stats","Secondary") : "")) + '</section>' +
    '<section class="setting-section"><h3>Published games</h3>' + setting(entries.length + " / 32 games","Publish from a replay. Unpublishing removes the listing; shared copies remain.",
      entries.length ? entries.slice(0,3).map(entry => '<div class="publication-row"><span>' + esc(entry.title) + "</span>" + button("Unpublish","minus","unpublish","Neutral",'data-entry="' + entry.id + '"') + "</div>").join("") +
        (entries.length > 3 ? '<p class="hint">' + (entries.length - 3) + " more publications · management layout sample</p>" : "") : '<p class="hint">No games published yet.</p>' + button("My games","replay","nav-games","Secondary")) + '</section>' +
    '<section class="setting-section"><h3>Visibility</h3>' + setting("Disable public profile","Removes the public page, statistics and published games.",state.confirmDisable
      ? '<div class="danger-confirm"><p>Disable this public profile? Re-enabling starts empty with statistics off. Shared replay copies remain.</p><div class="setting-actions">' + button("Confirm disable","minus","confirm-disable","Danger") + button("Keep profile","back","cancel-disable","Neutral") + "</div></div>"
      : button("Disable public profile","minus","disable","Danger")) + "</section>";
  manager += "</section>";
  return '<div class="app owner-app">' + header("Profile", "Player record", false) + '<div class="owner-shell"><aside class="owner-private"><p class="section-label">Identity · Account</p><label class="section-label" for="private-name">Name</label><input id="private-name" class="uiInput" readonly value="Juniper"><div class="uiDivider"></div><p class="quiet">Signed in. Private settings and game history stay private.</p>' + button("Sign out","sign_out","nav-signout","Neutral") +
    '<div class="uiDivider"></div>' + button("Reset profile","reset","nav-reset","Neutral").replace('class="uiAction ', 'class="reset-private uiAction ') +
    '</aside><div style="min-width:0"><nav class="owner-tabs" aria-label="Profile sections">' + button("My games","replay","nav-games","Neutral") + button("Saved replays","save","nav-saved","Neutral") +
    '<button type="button" class="uiAction uiActionSecondary" aria-current="page" data-action="owner-tab">Public profile</button></nav>' + manager + "</div></div></div>";
}
function render() {
  app.innerHTML = state.surface === "owner" ? owner() : visitor();
}
function toast(message) {
  const element = document.getElementById("toast");
  clearTimeout(toastTimer); element.textContent = message; element.hidden = false;
  toastTimer = setTimeout(() => { element.hidden = true; }, 4500);
}
function switchSurface(surface) {
  state.surface = surface; render(); window.scrollTo(0,0);
  if (parent !== window) parent.postMessage({type:"gomoku-study-surface",surface}, location.origin);
}
function openReplay(entry, trigger) {
  replayTrigger = trigger;
  dialog.innerHTML = '<div class="replay-header"><div><p class="uiPageEyebrow">Replay · navigation mock</p><h2 id="replay-title">' + esc(entry.title) + "</h2></div>" + button("Back to " + state.name + "'s games","back","close-replay","Neutral") +
    '</div><div class="replay-body">' + boardSvg(500) + '<div class="replay-meta"><dl><dt>Players</dt><dd>' + esc(state.name) + " vs " + esc(entry.opponent) +
    '</dd></dl><dl><dt>Rule</dt><dd>' + entry.rule + '</dd></dl><dl><dt>Opening</dt><dd>Move 8</dd></dl><p class="replay-note">This sketch demonstrates returning to the same profile page. The board is an illustration; gameplay and analysis use the existing app.</p></div></div>';
  dialog.showModal();
}
dialog.addEventListener("click", (event) => { if (event.target.closest('[data-action="close-replay"]')) dialog.close(); });
dialog.addEventListener("close", () => { replayTrigger?.focus({preventScroll:true}); });
app.addEventListener("click", async (event) => {
  const trigger = event.target.closest("[data-action]");
  if (!trigger) return;
  const action = trigger.dataset.action;
  if (action === "next" || action === "previous") {
    state.page += action === "next" ? 1 : -1; render();
    app.querySelector(action === "next" ? ".next:not(.absent)" : ".previous:not(.absent)")?.focus({preventScroll:true});
  } else if (action === "replay") {
    const entry = games().find(item => item.id === Number(trigger.dataset.entry)); if (entry) openReplay(entry, trigger);
  } else if (action === "copy") {
    const link = new URL(location.href);
    link.searchParams.delete("embed");
    link.searchParams.set("surface", "visitor");
    const value = link.toString();
    try { if (!navigator.clipboard) throw new Error("manual"); await navigator.clipboard.writeText(value); toast("Prototype link copied."); }
    catch {
      const input = document.createElement("input"); input.value = value; input.setAttribute("aria-label","Prototype link to copy"); input.style.cssText = "position:fixed;top:12px;left:12px;width:calc(100% - 24px);z-index:50"; document.body.append(input); input.focus(); input.select();
      let copied = false; try { copied = document.execCommand("copy"); } catch {}
      if (copied) { input.remove(); trigger.focus(); toast("Prototype link copied."); }
      else { toast("Link selected. Copy it, then press Escape."); input.addEventListener("keydown", (key) => { if (key.key === "Escape") { input.remove(); trigger.focus(); } }); }
    }
  } else if (action === "view-public") switchSurface("visitor");
  else if (action === "save-name") { state.name = Array.from(document.getElementById("public-name").value.trim()).slice(0,80).join("") || "Juniper"; state.long = false; render(); toast("Public name updated in this prototype."); }
  else if (action === "stats-on" || action === "stats-off") { const next = action === "stats-on"; if (state.stats !== next) { state.stats = next; render(); toast(next ? "Snapshot shared in this prototype." : "Snapshot hidden in this prototype."); } }
  else if (action === "refresh-stats") toast("Snapshot refreshed in this prototype.");
  else if (action === "unpublish") { state.removed.add(Number(trigger.dataset.entry)); render(); toast("Publication removed from this prototype."); }
  else if (action === "disable") { state.confirmDisable = true; render(); }
  else if (action === "cancel-disable") { state.confirmDisable = false; render(); }
  else if (action === "confirm-disable") { state.enabled = false; state.stats = false; state.scenario = "empty"; state.confirmDisable = false; render(); toast("Public profile disabled in this prototype."); }
  else if (action === "enable") { state.confirmEnable = true; render(); }
  else if (action === "cancel-enable") { state.confirmEnable = false; render(); }
  else if (action === "confirm-enable") { state.enabled = true; state.confirmEnable = false; render(); toast("Public profile enabled. Statistics remain off."); }
  else if (action === "retry" || action === "reload") { state.scenario = "full"; state.removed.clear(); render(); toast("Prototype fixtures reloaded."); }
  else if (action.startsWith("nav-")) { toast("Prototype destination: " + ({"nav-play":"local match","nav-profile":"your own Profile","nav-home":"Home","nav-games":"My games","nav-saved":"Saved replays","nav-signout":"account sign-out","nav-reset":"private reset confirmation"}[action] || action) + "."); }
});
fetch("assets/icons.json").then(response => { if (!response.ok) throw new Error("Icons unavailable"); return response.json(); }).then(value => { icons = value; render(); }).catch(error => { render(); toast(error.message); });
