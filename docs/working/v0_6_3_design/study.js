"use strict";
const D = window.STUDY_DATA;
const main = document.querySelector("main");
const byName = new Map(D.icons.map(x => [x.name, x]));
const validSections = new Set(["overview", "icons", "lab", "reviews", "process"]);
const escapeHTML = value => String(value).replace(/[&<>"']/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
const e = escapeHTML;
const glyph = name => byName.get(name).svg.replace(/<title>.*?<\/title>/s, "").replace("<svg ", '<svg aria-hidden="true" focusable="false" ');
const sourceLink = path => `https://github.com/byebyebryan/gomoku2d/blob/${D.publicSource}/${path}`;
let timer = null, spritePlaying = false, spriteFrame = 0, spriteImage = null, spriteKey = "";
let lastHash = null, lastSection = null;
const motion = matchMedia("(prefers-reduced-motion: reduce)");
function announce(message) { document.getElementById("announce").textContent = message; }
function state() {
  const [raw, query = ""] = location.hash.slice(1).split("?");
  const q = new URLSearchParams(query);
  const section = validSections.has(raw) ? raw : "overview";
  let selected = [...new Set((q.get("compare") || q.get("icon") || "share").split(","))].filter(x => byName.has(x)).slice(0,4);
  if (!selected.length) selected = ["share"];
  return {section, view: ["style","icons","sprites"].includes(q.get("view")) ? q.get("view") : "icons",
    selected, light:q.get("surface")==="light", grid:q.get("grid")!=="off", search:q.get("q")||"",
    tab:["ranking","search","analysis"].includes(q.get("tab")) ? q.get("tab") : "ranking",
    frame:[15,25,28].includes(Number(q.get("frame"))) ? Number(q.get("frame")) : 15,
    review:q.get("case")==="public" ? "public" : "profile", full:q.get("full")==="1", sync:q.get("sync")!=="off",
    animation:D.sprites.animations.some(a=>a.name===q.get("animation")) ? q.get("animation") : "stone-idle-1",
    tint:["white","black","red"].includes(q.get("tint")) ? q.get("tint") : "white"};
}
function hash(s) {
  const q = new URLSearchParams();
  if (s.section === "icons") {
    if (s.view !== "icons") q.set("view", s.view);
    if (s.view === "icons") { if(s.selected.length>1) q.set("compare",s.selected.join(",")); else if(s.selected[0]!=="share") q.set("icon",s.selected[0]); if(s.light) q.set("surface","light"); if(!s.grid)q.set("grid","off"); if(s.search)q.set("q",s.search); }
    if (s.view === "sprites") { q.set("animation",s.animation); if(s.tint!=="white")q.set("tint",s.tint); }
  }
  if (s.section === "lab") {if(s.tab!=="ranking")q.set("tab",s.tab);if(s.tab==="analysis"&&s.frame!==15)q.set("frame",s.frame);}
  if (s.section === "reviews") {if(s.review!=="profile")q.set("case",s.review);if(s.full)q.set("full","1");if(!s.sync)q.set("sync","off");}
  return "#"+s.section+(q.size?"?"+q.toString():"");
}
function go(patch, key) {
  const next = {...state(), ...patch};
  history.pushState(null,"",hash(next)); render();
  if(key) document.querySelector(`[data-focus-key="${key}"]`)?.focus();
}
function head(eyebrow,title,description,context="") {return `<header class="page-head"><p class="eyebrow">${eyebrow}</p><h1>${title}</h1><p class="lead">${description}</p>${context?`<div class="context-line">${context}</div>`:""}</header>`;}
function localNav(items,current,field) {return `<nav class="view-nav" aria-label="${field==="view"?"Asset":"Lab"} views">${items.map(([id,label])=>`<a href="${hash({...state(),[field]:id})}" ${id===current?'aria-current="page"':""}>${label}</a>`).join("")}</nav>`;}
function overview() {
  const rows=[["icons","grid","Design & assets","The visual language, actual controls and the pixels behind them."],["lab","bot","Inside the bot lab","Inspect recorded experiments, search costs and where games turned."],["reviews","replay","Review the work","Compare what changed, the findings and the evidence behind a fix."],["process","settings","How development works","Research, implement, inspect, validate—and improve the next iteration."]];
  return head("The work behind the game","Inside Gomoku2D","Design, experiments and the checks that shape the game.") +
    `<div class="feature-links">${rows.map(([id,icon,title,copy])=>`<a href="#${id}" class="feature-row"><span class="glyph">${glyph(icon)}</span><span><h2>${title}</h2><p>${copy}</p></span><span class="feature-arrow" aria-hidden="true">→</span></a>`).join("")}</div>
    <div class="overview-bottom"><section><h2>AI-built. Human-directed.</h2><p class="muted">Agents research, implement, inspect and test. Human judgment sets scope, domain meaning, visual taste and acceptance.</p><p class="muted">The useful record is the decision, the result and the evidence that changed the next step.</p><a class="text-link" href="#process">Follow the development process</a></section><section><h2>Start with a concrete example</h2><ul class="record-list"><li><a href="#icons?compare=share%2Csave%2Cpublish">Compare the sharing icons</a><span class="meta">Canonical SVGs · native size and pixel grid</span></li><li><a href="#reviews">Mobile Profile, before and after</a><span class="meta">Dated 0.6.2 checkpoint · follow-up identified</span></li><li><a href="#lab?tab=analysis">A possible last escape</a><span class="meta">Recorded analysis · bounded model</span></li></ul></section></div>`;
}
function inspection(icon,count) {
  return `<article class="icon-inspection" data-inspect="${icon.name}"><h3>${e(icon.label)}${count>1?`<button class="remove" data-action="remove" data-name="${icon.name}" aria-label="Remove ${e(icon.label)} from comparison">×</button>`:""}</h3><div class="inspection-surface"><div class="native-samples"><div><small>1×</small><span class="native-one" data-native="${icon.name}">${glyph(icon.name)}</span></div><div><small>2×</small><span class="native-two">${glyph(icon.name)}</span></div></div><div class="pixel-grid" role="img" aria-label="${e(icon.label)}, 24 by 24 pixel grid">${glyph(icon.name)}</div></div><div class="icon-metrics">Margins L / T / R / B: ${icon.margins.join(" / ")}<br>${icon.pixels} filled pixels · <a href="icons/${icon.name}.svg">Source SVG</a></div></article>`;
}
function catalog(s) {
  const found=D.icons.filter(x=>(x.name+" "+x.label+" "+x.category).toLowerCase().includes(s.search.toLowerCase()));
  return found.map(icon=>`<article class="catalog-card ${s.selected.includes(icon.name)?"selected":""}"><button class="catalog-open" data-action="inspect" data-name="${icon.name}" data-focus-key="inspect-${icon.name}" aria-label="Inspect ${e(icon.label)}"><span class="catalog-glyph">${glyph(icon.name)}</span><span class="catalog-name">${e(icon.name)}</span></button><button class="compare-add" data-action="compare" data-name="${icon.name}" data-focus-key="compare-${icon.name}" aria-pressed="${s.selected.includes(icon.name)}" aria-label="Compare ${e(icon.label)}">${s.selected.includes(icon.name)?"✓ In comparison":"+ Compare"}</button></article>`).join("") || '<p class="muted">No matching icons.</p>';
}
function iconPage(s) {
  return `<div class="tool-row"><label><span class="search-label">Find an icon</span><input id="icon-search" type="search" value="${e(s.search)}" placeholder="Name or category" autocomplete="off"></label><span class="count quiet">41 source icons · 24 × 24</span></div>
    <section class="inspector ${s.light?"light-surface":""} ${s.grid?"":"grid-off"}" aria-label="Icon inspector"><div class="inspector-head"><div><h2>Pixel inspection</h2><p class="meta">Native samples beside the enlarged source grid.</p></div><div class="inspector-controls"><button class="small-button" data-action="grid" data-focus-key="grid" aria-pressed="${s.grid}">Grid ${s.grid?"on":"off"}</button><button class="small-button" data-action="light" data-focus-key="light" aria-pressed="${s.light}">${s.light?"Light":"Dark"} surface</button><button class="small-button" data-action="browse">Browse all</button></div></div><div class="compare-grid" data-count="${s.selected.length}">${s.selected.map(name=>inspection(byName.get(name),s.selected.length)).join("")}${s.selected.length===1?`<aside class="single-inspector-note"><h3>One grid. A consistent family.</h3><p>Inspect the actual source at 1× and 2×. The larger view makes stepped contours, line weight and alignment easy to compare.</p><p>Gold axes mark the centre; the pixel grid can be switched off. Existing margins are shown as facts, not corrected automatically.</p><button class="small-button" data-action="sharing">Compare sharing actions</button><details class="disclosure"><summary>Source identity</summary><p class="meta">${e(byName.get(s.selected[0]).note)}</p><p class="meta">SVG SHA-256<br><code>${byName.get(s.selected[0]).sha256}</code></p></details></aside>`:""}</div></section>
    <div class="section-heading"><h2 id="catalog-title" tabindex="-1">Icon catalog</h2><span class="quiet">Inspect one · compare up to four</span></div><div class="icon-catalog" id="icon-catalog">${catalog(s)}</div>`;
}
function stylePage() {
  const names=[["--bg","Background"],["--surface","Panel"],["--text","Text"],["--text-muted","Muted text"],["--accent","Emphasis"],["--info","Secondary action"],["--success","Primary action"],["--danger","Danger"]];
  return `<section><h2>Colours with a job</h2><p class="muted readable">Current values from the application CSS. Reference pages should follow these tokens as the product evolves.</p><div class="palette">${names.map(([key,label])=>`<div class="swatch"><span class="swatch-color" style="background:var(${key})"></span><span>${label}<code>${key} · ${D.tokens[key]}</code></span></div>`).join("")}</div></section><section class="divider-section"><h2>Actual app controls</h2><p class="muted readable">The same classes, not a redrawn button set. Try pointer, pressed and keyboard-focus states.</p><div class="specimen-controls"><button class="uiAction uiActionPrimary">Primary</button><button class="uiAction uiActionSecondary">Secondary</button><button class="uiAction uiActionNeutral">Neutral</button><button class="uiAction uiActionDanger">Danger</button><button class="uiAction uiActionNeutral" disabled>Disabled</button></div></section><section class="divider-section readable"><h2>Pixel type, with room to read</h2><p>PixelOperator carries explanations and metadata. PixelOperatorBold carries headings, labels and actions. A quieter measure and line spacing make longer developer pages easier to read without changing the product's identity.</p><p class="quiet">Font sources and license are bundled with this study.</p></section>`;
}
function spritePage(s) {
  const anim=D.sprites.animations.find(a=>a.name===s.animation);
  return `<div class="sprite-workbench"><div class="sprite-stage"><canvas id="sprite-canvas" width="16" height="16" aria-label="Selected sprite frame"></canvas></div><div class="sprite-controls"><label><span class="search-label">Animation</span><select id="animation-select">${D.sprites.animations.map(a=>`<option value="${a.name}" ${a.name===s.animation?"selected":""}>${e(a.name)}</option>`).join("")}</select></label><div class="button-group" aria-label="Sprite colour">${["white","black","red"].map(t=>`<button class="small-button" data-action="tint" data-tint="${t}" aria-pressed="${s.tint===t}">${t}</button>`).join("")}</div><div class="button-group"><button class="small-button primary" id="sprite-play" data-action="sprite-play" aria-pressed="false">Play animation</button><button class="small-button" data-action="sprite-step">Next frame</button><button class="small-button" data-action="sprite-reset">Reset frame</button></div><p class="meta" id="frame-meta">Frame ${anim.start} · ${anim.start}–${anim.end} · ${anim.fps} fps</p><p class="muted">${e(anim.role)}. Native 16 × 16 pixels, shown at 10×.</p><p class="quiet">Original sheet with source RGB multiplied by the chosen colour; alpha retained. Reduced motion starts paused. This preview is not a Phaser renderer test.</p></div></div><details class="disclosure"><summary>Source sheet and frame range</summary><img class="source-sheet" src="${anim.file}" alt="${e(anim.name)} source sheet"><p class="meta">Seven sheets · 22 source-defined animations. No engine or account is loaded.</p></details>`;
}
function assets(s) {return head("Design & assets","Pixels with a purpose","The current visual language, from controls to the smallest source pixel.")+localNav([["style","Style"],["icons","Icons"],["sprites","Sprites"]],s.view,"view")+(s.view==="style"?stylePage():s.view==="sprites"?spritePage(s):iconPage(s));}
function botLabel(spec) {
  const depth=/search-d(\d+)/.exec(spec)?.[1];
  return {title:`Depth ${depth}${spec.includes("pattern-eval")?" · Pattern eval":" · Basic eval"}`,
    note:[/tactical-cap-(\d+)/.exec(spec)?.[1] ? `Tactical cap ${/tactical-cap-(\d+)/.exec(spec)[1]}` : "Full tactical width",spec.includes("corridor-proof")?"Corridor proof":""].filter(Boolean).join(" · ")};
}
function ranking() {
  return `<div class="ledger-head" aria-hidden="true"><span>#</span><span>Bot configuration</span><span>Score</span><span>W / D / L</span><span>Avg search</span></div>${D.lab.standings.map((b,i)=>{
    const label=botLabel(b.bot), score=(b.wins+b.draws*.5)/b.match_count*100;
    return `<details class="bot-row"><summary class="bot-summary"><span class="bot-rank">${i+1}</span><span class="bot-name"><strong>${label.title}</strong><small>${label.note}</small></span><span class="bot-score"><span class="mobile-label">Score</span>${score.toFixed(1)}%</span><span class="bot-record"><span class="mobile-label">W / D / L</span>${b.wins} / ${b.draws} / ${b.losses}</span><span class="bot-time"><span class="mobile-label">Avg search</span>${Math.round(b.avg_search_time_ms).toLocaleString()} ms</span></summary><div class="bot-details"><p class="meta">${b.match_count} games · score = wins + half the draws, divided by games</p><dl><div><dt>Shuffled Elo</dt><dd>${b.shuffled_elo_avg.toFixed(0)} ± ${b.shuffled_elo_stddev.toFixed(0)}</dd></div><div><dt>Average depth</dt><dd>${b.avg_depth.toFixed(2)}</dd></div><div><dt>Budget hit</dt><dd>${(b.budget_exhausted_rate*100).toFixed(1)}%</dd></div></dl><code>${e(b.bot)}</code></div></details>`;
  }).join("")}`;
}
const stages=[["stage_move_gen_ns","Move generation","#5f8b82"],["stage_ordering_ns","Ordering","#706c66"],["stage_eval_ns","Scoring","#fccb57"],["stage_threat_ns","Threat detection","#30a860"],["stage_proof_ns","Proof","#b877ff"]];
function search() {return `<p class="muted readable">Where time went inside the instrumented stages. This is a share of those stages, not total wall time.</p>${D.lab.standings.map(b=>{const total=stages.reduce((sum,[key])=>sum+b[key],0);return `<article class="search-row"><div class="search-row-head"><strong>${botLabel(b.bot).title}</strong><span class="meta">${botLabel(b.bot).note} · ${Math.round(b.avg_nodes).toLocaleString()} average nodes</span></div><div class="stage-bar" aria-hidden="true">${stages.map(([key,,color])=>`<span style="width:${b[key]/total*100}%;background:${color}"></span>`).join("")}</div><div class="stage-labels">${stages.filter(([key])=>b[key]>0).map(([key,label,color])=>`<span><span class="stage-key" style="background:${color}"></span>${label} ${(b[key]/total*100).toFixed(0)}%</span>`).join("")}</div></article>`;}).join("")}<p class="quiet">The full report retains detailed totals and other/unattributed time. No benchmark was rerun for this study.</p>`;}
function board(frame) {
  const moves=D.lab.entry.match_report.move_cells, prefix=frame.ply-1;
  const notation=n=>({x:(n.charCodeAt(0)-65)*22+26,y:(Number(n.slice(1))-1)*22+26});
  const grid=Array.from({length:15},(_,i)=>`<path d="M26 ${26+i*22} H334 M${26+i*22} 26 V334" stroke="#121212" stroke-width="1"/>`).join("");
  const evidence=frame.markers.filter(m=>m.kinds.some(k=>k.endsWith("evidence"))).map(m=>{const p=notation(m.notation);return `<rect x="${p.x-8}" y="${p.y-8}" width="16" height="16" fill="#ff7ab6" opacity=".6"/>`;}).join("");
  const stones=moves.slice(0,prefix).map((cell,i)=>`<rect x="${(cell%15)*22+18}" y="${Math.floor(cell/15)*22+18}" width="16" height="16" fill="${i%2?"#ffffff":"#404040"}" stroke="${i%2?"#ffffff":"#121212"}" stroke-width="1"/>`).join("");
  const markers=frame.markers.filter(m=>m.kinds.includes("actual")||m.kinds.includes("possible_escape")).map(m=>{const p=notation(m.notation);if(m.kinds.includes("actual"))return `<rect x="${p.x-6}" y="${p.y-6}" width="12" height="12" fill="none" stroke="${frame.side_to_move==="Black"?"#404040":"#ffffff"}" stroke-width="3"/>`;return `<rect x="${p.x-9}" y="${p.y-9}" width="18" height="18" fill="#5f8b82"/><text x="${p.x}" y="${p.y+5}" text-anchor="middle" font-size="16" fill="#121212">P</text>`;}).join("");
  const labels=Array.from({length:15},(_,i)=>`<text x="${26+i*22}" y="14" text-anchor="middle" font-size="13" fill="#121212">${String.fromCharCode(65+i)}</text><text x="12" y="${30+i*22}" text-anchor="middle" font-size="13" fill="#121212">${i+1}</text>`).join("");
  return `<svg class="proof-board" role="img" aria-label="Recorded board before move ${frame.ply}, ${frame.side_to_move} to play; ${prefix} earlier stones" viewBox="0 0 360 360" xmlns="http://www.w3.org/2000/svg"><rect width="360" height="360" fill="#ffcc66"/>${grid}${evidence}${stones}${markers}${labels}</svg>`;
}
function analysis(s) {
  const entry=D.lab.entry, frame=entry.proof_details.proof_frames.find(f=>f.ply===s.frame);
  return `<div class="analysis-layout"><div>${board(frame)}<p class="proof-legend">Filled: earlier moves · outline: actual move${frame.ply===15?" · P: possible escape":""}</p></div><section><p class="eyebrow">Easy / Normal source configurations</p><h2>Before move ${frame.ply}</h2><p class="muted">${frame.side_to_move} to play. ${frame.ply===15?"The recorded model found a possible escape at I11. It is not a guaranteed winning line.":"The recorded model marks a forced line for White, the eventual winner."}</p><div class="view-nav" aria-label="Recorded proof frames">${[15,25,28].map(n=>`<a href="${hash({...s,frame:n})}" ${s.frame===n?'aria-current="page"':""}>Before ${n}</a>`).join("")}</div><dl class="analysis-facts"><div><dt>Last chance</dt><dd>After move ${entry.last_chance_ply}</dd></div><div><dt>Setup corridor</dt><dd>${entry.setup_corridor.start_ply}–${entry.setup_corridor.end_ply}</dd></div><div><dt>Lethal onset</dt><dd>After move ${entry.lethal_onset.prefix_ply}</dd></div><div><dt>Recorded result</dt><dd>White · ${entry.match_report.move_count} moves</dd></div></dl><a class="text-link" href="https://gomoku2d.byebyebryan.com/lab/?tab=analysis&amp;match=${entry.path}">Open the published analysis entry</a><p class="quiet" style="margin-top:16px">Report analysis revision ${D.lab.analysisProvenance.git_commit}. This view renders recorded frames; it runs no analyzer.</p></section></div>`;
}
function lab(s) {
  return head("Experiments & analysis","Inside the bot lab","Compare behavior under a recorded budget, then inspect the decisions.",`<span><strong>Renju</strong> · 15 × 15</span><span>Recorded June 1, 2026</span><span>8 configurations · 64 games per pair</span>`) +
    localNav([["ranking","Ranking"],["search","Search"],["analysis","Analysis"]],s.tab,"tab") + `<div class="run-note"><strong>Reference run</strong> · pooled CPU budget, 2 seconds base per search move. These figures describe the recorded engine revisions—not the current browser or a live ranking.</div>` +
    (s.tab==="ranking"?ranking():s.tab==="search"?search():analysis(s)) +
    `<details class="disclosure"><summary>How to read this report · run & source details</summary><div class="readable"><p>Ranking scores are local to this tournament. Search figures describe the recorded host and settings. Analysis is bounded explanation, not a full-game solution.</p><p>Bot run <code>${D.lab.provenance.git_commit}</code> · ${D.lab.provenance.generated_at_utc}. Analysis <code>${D.lab.analysisProvenance.git_commit}</code> · ${D.lab.analysisProvenance.generated_at_utc}.</p><p><a class="text-link" href="${sourceLink('reports/lab/bot-report.json')}">Recorded bot data</a> · <a class="text-link" href="${sourceLink('reports/lab/analysis-report.json')}">Recorded analysis data</a> · <a class="text-link" href="https://gomoku2d.byebyebryan.com/lab/">Full published report</a></p></div></details>`;
}
function reviews(s) {
  const profile=s.review==="profile", title=profile?"Keep the record within reach":"More games, less repeated chrome";
  const description=profile?"A mobile Profile pass that brought the record tabs and useful controls closer to the player.":"A compact public-game list, using the established mobile replay action.";
  return head("Review / mobile records",title,description,`<span>October 23–24, 2026</span><span>390 × 844 · application captures</span>`)+
    `<div class="status-line"><span class="technical">Implementation checked</span><span class="pending">Physical-phone review unperformed</span><span>Historical checkpoint · follow-up recorded</span></div><nav class="view-nav" aria-label="Review cases">${[["profile","Profile"],["public","Public player"]].map(([id,label])=>`<a href="${hash({...s,review:id})}" ${id===s.review?'aria-current="page"':""}>${label}</a>`).join("")}</nav><div class="review-toolbar"><button class="small-button" data-action="full" data-focus-key="full" aria-pressed="${s.full}">${s.full?"Fit comparison panes":"Show full captures"}</button><button class="small-button" data-action="sync" data-focus-key="sync" aria-pressed="${s.sync}">Scroll together ${s.sync?"on":"off"}</button></div><div class="review-pair ${s.full?"full-captures":""}">${[["before","Before",D.review.baseline],["after","After this pass",D.review.runtime]].map(([id,label,commit])=>`<figure class="review-image"><figcaption><strong>${label}</strong><span>${e(commit)} · same ${profile?"guest Profile":"18-game public fixture"}</span></figcaption><div class="capture-pane" tabindex="0" aria-label="${label} capture, scroll to inspect"><img src="review/${s.review}-${id}.png" width="390" height="844" alt="${profile?"Mobile Profile":"Public player"} ${id} the recorded refinement"></div></figure>`).join("")}</div><section class="divider-section"><h2>What changed</h2><ul class="findings">${(profile?[["FIXED","Essential account controls stay visible","Secondary profile/reset controls move into a disclosure."],["FIXED","One stable record-tab row","My games, Saved replays and Public profile fit together on a narrow screen."]]:[["FIXED","A compact replay action","Existing 48px mobile icon control, with a full accessible name."],["FIXED","A clearer row hierarchy","Result, game and opponent lead; rules and move count stay secondary."]]).map(([status,heading,copy])=>`<li><span class="finding-state">${status}</span><span><strong>${heading}</strong><p class="muted">${copy}</p></span></li>`).join("")}</ul></section><details class="disclosure" open><summary>Checkpoint, follow-up and acceptance</summary><div class="readable"><p>These are the original captures for the ${D.review.runtime} mobile pass. Later landscape/enlarged-text fixes landed at <code>${D.review.followUp}</code>. This comparison is not the final 0.6.2 baseline.</p><p>${D.review.human} Browser/emulator validation is separate from that acceptance.</p><p>Profile and public-player pairs use matching scenarios. Their image bytes are unchanged; this study reorganizes the presentation.</p><p><a class="text-link" href="${sourceLink('docs/working/v0_6_2_mobile/README.md')}">Original review and follow-up evidence</a> · <a class="text-link" href="https://github.com/byebyebryan/gomoku2d/commit/9c4c3bb">Later fixes</a></p></div></details>`;
}
function process() {
  const steps=[["Research","Settle the question","Trace the current behavior, domain rules and constraints before choosing a change."],["Design","Make the choice reviewable","Use real content, concrete alternatives and source-backed visual references."],["Implement","Bound the change","Give the work clear behavior, ownership and invariants. Keep unrelated work safe."],["Review & fix","Find what is still wrong","Inspect source, actual browser behavior and captures. Preserve findings and follow-ups."],["Validate","Say what the evidence establishes","Choose checks for the changed contract. Keep automation, human acceptance and deployment distinct."],["Release","Verify what was published","Require the source-qualified gate, then compare deployed bytes and exercise the live build."]];
  return head("Development process","Build. Inspect. Improve.","How the work moves forward—and how the workflow itself gets better.")+`<section class="readable"><p>Human direction owns scope, domain meaning, taste and acceptance. Agents research, implement, inspect and validate. The next step follows the evidence, including failures and corrections.</p></section><dl class="process-grid">${steps.map(([label,title,copy])=>`<dt>${label}</dt><dd><h3>${title}</h3><p>${copy}</p></dd>`).join("")}</dl><section class="divider-section"><h2>One product, clear responsibilities</h2><div class="architecture" aria-label="Architecture responsibilities"><span>React · shell & state</span><span class="connector" aria-hidden="true">↔</span><span>Phaser · board</span><span class="connector" aria-hidden="true">↔</span><span>Rust / Wasm · rules & analysis</span></div><p class="muted readable">The native lab runs experiments around the same core. Optional Firebase continuity serves private account data. Developer references and recorded reports need neither an account nor private history.</p></section><section class="divider-section"><h2>Evidence has different jobs</h2><div class="evidence-rows"><div class="evidence-row"><strong>Automated checks</strong><p>Rules, state transitions, browser behavior and renderer pixels on a stated source. <a class="text-link" href="https://github.com/byebyebryan/gomoku2d/actions/runs/${D.release.ci.run_id}">0.6.2 CI</a> is a dated example.</p></div><div class="evidence-row"><strong>Visual review</strong><p>Layout, density, icon consistency and whether a feature communicates clearly. The comparison and actual decision belong together.</p></div><div class="evidence-row"><strong>Human/device use</strong><p>Physical touch, browser keyboards, assistive technology and real account round trips. Record what happened; an emulator pass does not close these checks.</p></div></div></section><section class="divider-section readable"><h2>A worked release, not a promise</h2><p>0.6.2 passed its exact-source CI and publication gates. The deployed browser checks recorded ${D.release.production.passed} passes and six intentional renderer skips. The real signed-in sharing handoff remains pending.</p><a class="text-link" href="${sourceLink('docs/archive/v0_6_2_release.md')}">Read the dated release record</a></section>`;
}
function render() {
  const s=state(), changed=s.section!==lastSection;
  clearInterval(timer); timer=null; spritePlaying=false;
  main.innerHTML = s.section==="overview"?overview():s.section==="icons"?assets(s):s.section==="lab"?lab(s):s.section==="reviews"?reviews(s):process();
  for(const link of document.querySelectorAll("[data-section]")){if(link.dataset.section===s.section)link.setAttribute("aria-current","page");else link.removeAttribute("aria-current");}
  document.title=`Gomoku2D · ${s.section==="icons"?"Design & assets":s.section[0].toUpperCase()+s.section.slice(1)} · design study`;
  if(changed&&lastSection!==null){window.scrollTo(0,0);main.querySelector("h1").tabIndex=-1;main.querySelector("h1").focus({preventScroll:true});}
  lastSection=s.section; lastHash=location.hash;
  if(s.section==="icons"&&s.view==="sprites")loadSprite(s);
  if(s.section==="reviews")wireScroll(s);
}
function onLocation() {if(location.hash!==lastHash)render();}
addEventListener("hashchange",onLocation);addEventListener("popstate",onLocation);
document.addEventListener("click",event=>{
  const anchor=event.target.closest('a[href="#content"],a[href="#study-notes"]');
  if(anchor){event.preventDefault();const target=document.querySelector(anchor.getAttribute("href"));target.scrollIntoView({block:"start"});if(target.id==="content")target.focus();else target.open=true;}
});
main.addEventListener("input",event=>{
  if(event.target.id!=="icon-search")return;
  const s={...state(),search:event.target.value};history.replaceState(null,"",hash(s));lastHash=location.hash;
  document.getElementById("icon-catalog").innerHTML=catalog(s);
});
main.addEventListener("change",event=>{if(event.target.id==="animation-select"){spriteKey="";go({animation:event.target.value});document.getElementById("animation-select")?.focus();}});
main.addEventListener("click",event=>{
  const button=event.target.closest("button[data-action]");if(!button)return;
  const s=state(),name=button.dataset.name,action=button.dataset.action;
  if(action==="grid")go({grid:!s.grid},"grid");
  if(action==="light")go({light:!s.light},"light");
  if(action==="sharing")go({selected:["share","save","publish"]});
  if(action==="browse"){const target=document.getElementById("catalog-title");target.focus();target.scrollIntoView({block:"start"});}
  if(action==="inspect"){go({selected:[name]});main.querySelector(".inspector h2").tabIndex=-1;main.querySelector(".inspector h2").focus();}
  if(action==="compare"){
    if(!s.selected.includes(name)&&s.selected.length===4){announce("Compare up to four icons. Remove one before adding another.");return;}
    const selected=s.selected.includes(name)?s.selected.filter(x=>x!==name):[...s.selected,name];go({selected:selected.length?selected:["share"]},`compare-${name}`);
  }
  if(action==="remove")go({selected:s.selected.filter(x=>x!==name)});
  if(action==="full")go({full:!s.full},"full");
  if(action==="sync")go({sync:!s.sync},"sync");
  if(action==="tint")go({tint:button.dataset.tint});
  if(action==="sprite-play"){
    if(spritePlaying){stopSprite();return;}
    if(motion.matches){announce("Reduced motion is on. Use Next frame to inspect the animation.");return;}
    spritePlaying=true;button.textContent="Pause animation";button.setAttribute("aria-pressed","true");
    const anim=D.sprites.animations.find(a=>a.name===s.animation);
    timer=setInterval(()=>{spriteFrame=spriteFrame>=anim.end?anim.start:spriteFrame+1;drawSprite(state());},1000/anim.fps);
  }
  if(action==="sprite-step"){stopSprite();const a=D.sprites.animations.find(a=>a.name===s.animation);spriteFrame=spriteFrame>=a.end?a.start:spriteFrame+1;drawSprite(s);}
  if(action==="sprite-reset"){stopSprite();spriteFrame=D.sprites.animations.find(a=>a.name===s.animation).start;drawSprite(s);}
});
function wireScroll(s){const panes=[...main.querySelectorAll(".capture-pane")];let busy=false;for(const pane of panes)pane.addEventListener("scroll",()=>{if(busy||!s.sync||s.full)return;busy=true;const other=panes.find(p=>p!==pane),range=pane.scrollHeight-pane.clientHeight;other.scrollTop=range>0?pane.scrollTop/range*(other.scrollHeight-other.clientHeight):0;requestAnimationFrame(()=>{busy=false;});});}
function stopSprite(){clearInterval(timer);timer=null;spritePlaying=false;const b=document.getElementById("sprite-play");if(b){b.textContent="Play animation";b.setAttribute("aria-pressed","false");}}
function loadSprite(s){const a=D.sprites.animations.find(a=>a.name===s.animation);if(spriteKey!==a.name){spriteFrame=a.start;spriteKey=a.name;}const image=new Image();image.onload=()=>{if(state().section==="icons"&&state().view==="sprites"&&state().animation===a.name){spriteImage=image;drawSprite(state());}};image.src=a.file;}
function drawSprite(s){const canvas=document.getElementById("sprite-canvas");if(!canvas||!spriteImage)return;const a=D.sprites.animations.find(a=>a.name===s.animation),context=canvas.getContext("2d",{willReadFrequently:true});context.clearRect(0,0,16,16);context.drawImage(spriteImage,(spriteFrame%a.cols)*16,Math.floor(spriteFrame/a.cols)*16,16,16,0,0,16,16);const image=context.getImageData(0,0,16,16),rgb=s.tint==="black"?[64,64,64]:s.tint==="red"?[255,68,68]:[255,255,255];for(let i=0;i<image.data.length;i+=4)for(let c=0;c<3;c++)image.data[i+c]=Math.round(image.data[i+c]*rgb[c]/255);context.putImageData(image,0,0);canvas.dataset.frame=String(spriteFrame);canvas.dataset.tint=s.tint;document.getElementById("frame-meta").textContent=`Frame ${spriteFrame} · ${a.start}–${a.end} · ${a.fps} fps`;}
motion.addEventListener("change",()=>{if(motion.matches)stopSprite();});
for(const el of document.querySelectorAll("[data-icon]"))el.innerHTML=glyph(el.dataset.icon);
render();
