"use strict";
const D = window.HISTORY_STUDY;
const milestones = D.milestones;
const screens = ["Home", "Match", "Replay", "Profile", "Settings", "Public player", "Rules & Guide", "Lab", "Visuals"];
const layouts = {
  journal: "A · Read changes in order. Each milestone opens its comparison in the flow of the page.",
  screens: "B · Follow one screen through the versions that changed it. Compare neighbouring steps when a detail catches your eye.",
  browser: "C · Keep the milestone list beside the comparison. Faster to move around, with less room for each capture.",
};
const chapters = {"0.2": "A local game", "0.3": "Cloud continuity", "0.4": "Explainable play", "0.5": "A public product", "0.6": "Games and people"};
let state;
let pendingFocus = null;
let restoreControl = null;
const escape = value => String(value).replace(/[&<>"']/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
const id = value => value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
function parse() {
  const p = new URLSearchParams(location.hash.slice(1));
  return {
    layout: Object.hasOwn(layouts, p.get("layout")) ? p.get("layout") : "journal",
    version: milestones.some(m => m.version === p.get("version")) ? p.get("version") : "0.6.2",
    screen: screens.includes(p.get("screen")) ? p.get("screen") : "Public player",
    viewport: p.get("viewport") === "desktop" ? "desktop" : "portrait",
  };
}
function url(change = {}) {
  return "#" + new URLSearchParams({...state, ...change}).toString();
}
function stateLink(text, change, extra = "") {
  return `<a href="${escape(url(change))}" data-state ${extra}>${escape(text)}</a>`;
}
function asset(version, screen) {
  return D.assets[`${version}|${screen}|${state.viewport}`];
}
function capture(version, screen, before = false) {
  const a = asset(version, screen);
  const introduced = before && !milestones.slice(0, milestones.findIndex(m => m.version === state.version)).some(m => m.screens.includes(screen));
  const caption = a ? a.label : introduced ? "Not present yet" : version ? `v${version}` : "Before this screen";
  const metadata = a ? `${a.width} × ${a.height} · ${a.kind}` : introduced ? "This milestone introduces the screen" : "Historical capture to add";
  return `<figure data-version="${escape(version || "none")}" data-availability="${a ? "capture" : introduced ? "introduced" : "missing"}">
    <figcaption><strong>${escape(caption)}</strong><span>${escape(metadata)}</span></figcaption>
    ${a ? `<img src="${escape(a.file)}" width="${a.width}" height="${a.height}" loading="lazy" alt="${escape(a.alt)}">` : `<div class="empty-capture"><span class="empty-marker">${introduced ? "INTRODUCED HERE" : "STUDY PLACEHOLDER"}</span><strong>${escape(screen)}</strong><p>${introduced ? "The earlier version has no equivalent screen to compare." : `Capture the ${state.viewport} ${screen.toLowerCase()} view from this source. No other version's screenshot is substituted.`}</p></div>`}
  </figure>`;
}
function viewportNav() {
  return `<nav class="viewport-nav" aria-label="Capture layout">${["portrait", "desktop"].map(v => stateLink(v === "portrait" ? "Portrait" : "Desktop", {viewport:v}, `data-control="viewport-${v}" aria-current="${state.viewport === v ? "page" : "false"}"`)).join("")}</nav>`;
}
function sourceNote(screen, versions) {
  if (screen === "Public player" && versions.includes("0.6.1")) return "Layout references: a 0.6.1-era development checkpoint and the 0.6.2 release. Synthetic names and game lists differ. A standardized release recapture is still needed.";
  return "Available images retain their original source identity. Empty frames show missing coverage in this design study; they do not imply that a screen was absent.";
}
function comparison(screen, before, after, namespace = "") {
  const assets = [asset(before, screen), asset(after, screen)];
  return `<section class="comparison" id="${namespace}compare-${id(screen)}" aria-label="${escape(screen)} comparison">
    <div class="section-intro"><h3>${escape(screen)}</h3><p>${before ? `v${escape(before)} → ` : "Introduced in "}v${escape(after)}</p></div>
    <div class="capture-grid" data-viewport="${state.viewport}">${capture(before, screen, true)}${capture(after, screen)}</div>
    <p class="capture-note">${escape(sourceNote(screen, [before, after]))}</p>
    ${assets.some(Boolean) ? `<div class="comparison-links">${assets.map((a,i) => a ? `<a href="${escape(a.file)}" target="_blank" rel="noopener">Open ${i ? "later" : "earlier"} image</a>` : "").join("")}</div>` : ""}
  </section>`;
}
function neighbour(m, filtered = milestones) {
  const i = filtered.findIndex(item => item.version === m.version);
  return {before:filtered[i-1], after:filtered[i+1]};
}
function neighbourNav(m, filtered = milestones) {
  const n = neighbour(m,filtered);
  return `<nav class="neighbour-nav" aria-label="Neighbouring milestones">${n.before ? stateLink(`← v${n.before.version}`, {version:n.before.version}, 'class="dev-tool" data-focus="milestone"') : '<span class="disabled-step"></span>'}${n.after ? stateLink(`v${n.after.version} →`, {version:n.after.version}, 'class="dev-tool next-step" data-focus="milestone"') : '<span class="disabled-step"></span>'}</nav>`;
}
function screenJumps(m, prefix = "") {
  return `<nav class="screen-jumps" aria-label="Changed screens">${m.screens.map(s => `<a href="#${prefix}compare-${id(s)}" data-jump="${prefix}compare-${id(s)}">${escape(s)}</a>`).join("")}</nav>`;
}
function detail(m, prefix = "") {
  const before = neighbour(m).before;
  return `<header class="detail-header"><h2 tabindex="-1" class="focus-heading" id="${prefix}comparison-heading">${before ? `v${before.version} → ` : ""}v${m.version}</h2>${viewportNav()}</header>
    ${screenJumps(m,prefix)}
    ${m.screens.map(s => comparison(s, before?.version, m.version,prefix)).join("")}
    <details class="dev-disclosure"><summary>Design notes and source context</summary><div class="dev-readable"><p>${escape(m.summary)}</p><p>The proposed milestone is based on the release notes. Source review and matching capture states will establish the final comparison. This study does not mark the milestone as captured or reviewed.</p><p><a href="${escape(D.changelogUrl)}">Read the release notes</a></p></div></details>`;
}
function journal() {
  return `<div class="section-intro"><h2>Changes, in order</h2><p>0.2 → 0.6 · proposed milestone sequence</p></div>
    <nav class="chapter-nav" aria-label="History chapters">${Object.entries(chapters).map(([v,t]) => `<a href="#chapter-${v}" data-jump="chapter-${v}">${v}</a>`).join("")}</nav>
    <p class="mode-caption"><a class="dev-tool" href="#journal-${state.version}" data-jump="journal-${state.version}">Jump to open comparison · v${state.version}</a></p>
    ${Object.entries(chapters).map(([v,t]) => `<section class="chapter" id="chapter-${v}"><h2>${v}<span>${escape(t)}</span></h2>
      ${milestones.filter(m => m.version.startsWith(v+".")).map(m => `<article id="journal-${m.version}">
        <div class="journal-row" ${m.version===state.version ? 'aria-current="true"' : ""}><div class="journal-version">${m.version}<small>${m.date.slice(5).replace("-"," / ")}</small></div>
        <div><h3>${escape(m.title)}</h3><p>${escape(m.summary)}</p><div class="affected">${m.screens.map(s => `<span>${escape(s)}</span>`).join("")}</div></div>
        ${stateLink(m.version===state.version ? "Comparison open" : "Compare previous", {version:m.version}, `class="dev-tool" data-focus="milestone" aria-expanded="${m.version===state.version}" aria-controls="journal-detail"`)}
        </div>${m.version===state.version ? `<div class="journal-detail" id="journal-detail">${detail(m,"journal-")}</div>` : ""}</article>`).join("")}
    </section>`).join("")}`;
}
function screenHistory() {
  const rows = milestones.filter(m => m.screens.includes(state.screen));
  const m = rows.find(x => x.version === state.version) || rows.at(-1);
  const before = neighbour(m,rows).before;
  return `<nav class="screen-nav" aria-label="Screen histories">${screens.map(s => stateLink(s, {screen:s}, `data-control="screen-${id(s)}" aria-current="${state.screen===s ? "page" : "false"}"`)).join("")}</nav>
    <div class="section-intro" style="margin-top:24px"><h2>${escape(state.screen)}, over time</h2>${viewportNav()}</div>
    <p class="screen-history-note">Only steps that changed this screen. Read left to right, then down. Each capture appears once in the trail.</p>
    <div class="screen-trail" data-count="${rows.length}" data-viewport="${state.viewport}">${rows.map(row => `<article class="trail-item"><h3>v${row.version}</h3><p>${escape(row.title)}</p>
      <div class="capture-grid">${capture(row.version,state.screen)}</div>
      <p class="capture-note">${escape(row.summary)}</p>
      ${stateLink(row.version === rows[0].version ? "Inspect introduction" : "Compare previous step", {version:row.version}, 'class="dev-tool compare-step" data-focus="screen-comparison"')}
    </article>`).join("")}</div>
    <section class="screen-focus" id="screen-comparison"><h2 tabindex="-1" class="focus-heading" id="screen-heading">Inspect a change</h2><p class="dev-muted">${escape(m.title)}</p>${neighbourNav(m,rows)}${comparison(state.screen,before?.version,m.version,"screen-")}</section>`;
}
function browser() {
  const m = milestones.find(x => x.version === state.version);
  return `<div class="history-workbench"><details class="milestone-rail" id="milestone-rail"><summary>Milestones · v${m.version} ▾</summary><nav class="rail-content" aria-label="Milestone list">
    ${Object.entries(chapters).map(([v,t]) => `<section class="rail-group"><h2>${v} / ${escape(t)}</h2>${milestones.filter(row => row.version.startsWith(v+".")).map(row => stateLink(`v${row.version}`, {version:row.version}, `data-title="${escape(row.title)}" data-version-link="${row.version}" data-focus="milestone" aria-current="${row.version===m.version ? "page" : "false"}"`)).join("")}</section>`).join("")}
    </nav></details><article class="milestone-panel"><p class="dev-eyebrow">v${m.version} · ${m.date}</p><h2 tabindex="-1" id="browser-milestone-heading" class="focus-heading">${escape(m.title)}</h2><p class="dev-muted">${escape(m.summary)}</p>${neighbourNav(m)}${detail(m,"browser-")}</article></div>`;
}
function render() {
  state = parse();
  for(const node of document.querySelectorAll("[data-icon]")) node.innerHTML=D.icons[node.dataset.icon] || "";
  document.title = `UI history · ${state.layout} · Gomoku2D study`;
  document.getElementById("option-note").textContent = layouts[state.layout];
  for (const link of document.querySelectorAll("[data-layout]")) {
    link.href = url({layout:link.dataset.layout});
    link.setAttribute("aria-current",link.dataset.layout===state.layout ? "page" : "false");
  }
  document.getElementById("history").innerHTML = state.layout === "journal" ? journal() : state.layout === "screens" ? screenHistory() : browser();
  for (const a of document.querySelectorAll("[data-version-link]")) {
    const strong = document.createElement("strong"); strong.textContent=a.textContent;
    const subtitle = document.createElement("span");subtitle.textContent=a.dataset.title;
    a.replaceChildren(strong,subtitle);
  }
  if (state.layout === "browser") {
    document.getElementById("milestone-rail").open = matchMedia("(min-width:701px)").matches;
    const selected = document.querySelector('.rail-content [aria-current="page"]');
    const rail = document.querySelector(".rail-content");
    if (selected && rail && matchMedia("(min-width:701px)").matches) rail.scrollTop = selected.offsetTop - 20;
  }
  if (pendingFocus) {
    const target = pendingFocus === "screen-comparison" ? document.getElementById("screen-heading") : state.layout === "journal" ? document.getElementById(`journal-${state.version}`) : document.getElementById("browser-milestone-heading");
    if(target) {target.setAttribute("tabindex","-1");target.focus({preventScroll:true});target.scrollIntoView({block:"start"});}
    pendingFocus=null;
  } else if (restoreControl) {
    document.querySelector(`[data-control="${restoreControl}"]`)?.focus({preventScroll:true});
  }
  restoreControl=null;
}
document.addEventListener("click",event => {
  const a = event.target.closest("a");
  if(!a || event.button!==0 || event.ctrlKey || event.metaKey || event.shiftKey || event.altKey) return;
  if(a.hasAttribute("data-jump")) {
    event.preventDefault();
    const target=document.getElementById(a.dataset.jump);
    if(target){target.setAttribute("tabindex","-1");target.focus({preventScroll:true});target.scrollIntoView({block:"start"});}
  } else if(a.hasAttribute("data-state")) {
    pendingFocus=a.dataset.focus || null;
    restoreControl=a.dataset.control || null;
    if(a.hash===location.hash){event.preventDefault();render();}
  }
});
addEventListener("hashchange",render);
matchMedia("(min-width:701px)").addEventListener("change", event => {
  const rail=document.getElementById("milestone-rail");
  if(rail) rail.open=event.matches;
});
render();
