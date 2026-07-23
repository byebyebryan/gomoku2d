# Process Story Evidence Map

Status: archived evidence map; not an active publication task.

Purpose: map the current process-story thesis to concrete private evidence so
future public writing can pull from the right conversation arcs, commits,
reports, and screenshots without publishing raw transcript dumps.

The smaller selected proof packet is
[`Process Story Evidence Cards`](process_story_evidence_cards.md). Use this
file as the broader routing map.

This is private working material. Use it to choose examples and verify claims.
Do not copy long transcript excerpts into public docs.

## Source Material

- `docs/archive/process_story/process_story_narrative_context.md`: current thesis and
  narrative arc.
- `docs/archive/process_story/process_story_leads.md`: editorial shortlist.
- `docs/archive/process_story/process_story_evidence_cards.md`: selected proof packet for
  the external story.
- `gomoku-bot-lab/outputs/process-story/conversation_arcs.md`: private arc
  index.
- `gomoku-bot-lab/outputs/process-story/conversation_arcs/*.md`: split arc
  review chunks.
- `gomoku-bot-lab/outputs/process-story/git_chronology.json`: commit/release
  trail.
- Current public surfaces: `/rules`, `/guide`, `/lab`, `/visuals`, replay
  analysis, and README copy.

## How To Use This Map

For each story beat:

- Use the claim as the public-facing point.
- Use the arc IDs to recover private conversation context.
- Use the commit/report/page artifacts as public evidence.
- Re-check current repo state before publishing anything specific.
- Prefer paraphrase. Quote only short phrases where the exact wording carries
  the point.

## Strongest Evidence Beats

### 1. Personal Revival Before AI

Claim: the project started as a personal revival, not as a generic AI-made game.
That gives the product a reason to exist before the process story enters.

Conversation evidence:

- `arc_0179`: README framing rejected "the game itself is the main thing" as
  too inward-facing; the root copy needed to explain what the game is and why
  the Rust lab matters.
- `arc_0180`: the framing moved toward a retro Gomoku game with a modern stack,
  and toward separating emotional/product hook from repo mechanics.
- `arc_0001`: the first recovered work already treats the repo as a Rust-first
  Gomoku sandbox with core rules, Renju, Wasm, bot, and web layers.

Commit/product evidence:

- `3b36490` restructured the Rust crates under `gomoku-bot-lab/` and reframed
  the web game as the product.
- `v0.1` and early `v0.2` commits establish the playable browser game, local
  replay, rules selection, and mobile layout.
- Current README and home copy carry the "retro Gomoku / built properly" idea
  without over-explaining the old project.

Best public shape:

- Opening paragraph of a devlog or README refresh.
- Pair with current home screen and one early architecture diagram or commit
  list.

Risk:

- Do not make this sentimental. Keep the proof concrete: playable web game,
  board-first interaction, replay, and real rules core.

### 2. Agents Bridged Unfamiliar Stacks, But Did Not Own Taste

Claim: agents made frontend, Phaser, Rust, Wasm, static pages, and reports
practical to work on, but the user still supplied taste, priorities, and
review pressure.

Conversation evidence:

- `arc_0001`: the user enters with a half-finished mobile/Phaser sizing problem
  and asks for a root-cause fix, not a surface patch.
- `arc_0002`: the settings UI is reframed around board replacement because it
  fits the game's visual language better than a generic overlay.
- `arc_0057`: the user rejects purely functional UI work and asks for a visual
  design/styling guide instead.
- `arc_0061`: design is stress-tested before implementation, showing producer
  mode rather than blind execution.

Commit/product evidence:

- `1d6d3c6`, `366f4fa`, and later visual commits show screenshot/design review
  becoming part of the workflow.
- `/visuals` now acts as a compact visual guide rather than just an asset dump.
- `/rules`, `/guide`, and `/lab` share a tighter public style after repeated
  copy and visual passes.

Best public shape:

- "Where agents helped" section: they widened the practical stack surface.
- "Where human judgment mattered" section: mobile layout, visual tone, copy,
  and what not to build.

Risk:

- Do not imply the human understood every implementation detail equally. The
  more accurate point is ownership of the product layer and critical review.

### 3. The Bot Lab Looked Oversized, Then Became Load-Bearing

Claim: a simple Gomoku game did not strictly need a Rust bot lab, tournament
reports, or generated artifacts. That foundation became valuable once the
project needed measurable bot behavior and replay analysis.

Conversation evidence:

- `arc_0350`: the 0.4 plan pivots away from exposing bot knobs before proving
  which knobs matter in the lab.
- `arc_0362`: structured report output is requested early, with raw data split
  from HTML so expensive tournaments do not need reruns for presentation
  tweaks.
- `arc_0386`: after early experiments, the next steps are realigned around
  measured lab evidence instead of more random knobs.
- `arc_0403`: failed shape-eval experiments are treated as evidence that the
  planning target is wrong, not just that one implementation needs more tuning.

Commit/product evidence:

- `63a381e` adds configurable search bot plumbing.
- `65df1c6`, `a19620c`, and `bbfc196` establish eval harness and tournament
  reporting.
- `v0.5.1` later replaces Rust-generated HTML with web-rendered report viewers
  and compact data.

Best public shape:

- "The lab was not a side project" section.
- Pair with current `/lab` plus one old/new report screenshot.

Risk:

- Keep bot-strength claims modest. The lab is strongest as a way to inspect and
  explain behavior.

### 4. Corridor Search Failed As A Bot Shortcut, Then Became Replay Analysis

Claim: corridor search is the cleanest process story: it failed as a broad live
search shortcut, but became the vocabulary and engine for explaining finished
games.

Conversation evidence:

- `arc_0504`: forced-line work is explicitly pivoted toward analysis first
  because it is needed regardless of Elo impact.
- `arc_0516`: the desired analyzer shape is defined as feeding a replay and
  walking back through a forced sequence under explicit limits.
- `arc_0532`: the bounded "escape" model emerges: find an alternative legal
  move that stops the winning sequence and does not immediately lose another
  way.
- `arc_0618`: report language settles on terms like immediate win, immediate
  threat, imminent threat, counter threat, and corridor entry.
- `arc_0621`: old replay-analysis policy branches are removed in favor of a
  simplified corridor reply model.
- `arc_0750`: later bot-side corridor proof work is made proof-only and budgeted
  after normal search, showing the bot-search version remained experimental.

Commit/product evidence:

- `9455fe6`, `bc49e6e`, `d4f36f6`, and `50025cd` build the first replay
  analyzer, fixture reports, batch report, and proof details.
- `9bb4c39` simplifies the replay-analysis policy surface.
- `d35a59b` removes the retired corridor portal path.
- `ed7090e` and `9221100` surface setup corridor/replay-analysis reporting.
- Replay UI and the Lab report use setup corridor, lethal onset, and last
  escape; `/guide` teaches the same flow as combo onset, setup corridor, and
  last escape.

Best public shape:

- Standalone devlog: "The failed bot trick that became replay analysis."
- Use a concrete report/replay frame showing last escape -> setup corridor ->
  lethal onset.

Risk:

- The analyzer is bounded explanation, not a full solver. Say that directly.

### 5. Domain Counterexamples Beat Generic Execution

Claim: Gomoku looks simple, but the project repeatedly needed domain-specific
corrections from the user. Generic "implement the tactic" work was not enough.

Conversation evidence:

- `arc_0592` and nearby arcs: broken/open three response variants are corrected
  and carried into both bot and corridor search.
- `arc_0878` and `arc_0879`: combo threat handling is tightened so all same-tier
  threats become candidates before filtering actual/illegal moves.
- `arc_0906`: lethal threat depth compression is explicitly framed as useful
  for analysis first, and only maybe useful for search.
- `arc_0908`: Renju complicates lethal threats because an apparent open four
  can be illegal for Black, and a single local threat can become lethal if the
  block is forbidden.

Commit/product evidence:

- `a2d187e` formalizes tactical shape vocabulary.
- `c360e8a`, `8b6b99b`, and related commits centralize and unify tactical
  threat policies.
- `aab37a9` and `39e10d9` refine corridor failure analysis and report details.
- `/guide` now teaches immediate threat, imminent threat, counter threat,
  combo, setup corridor, and last escape.

Best public shape:

- Technical sidebar: "The hard part was not five in a row. It was deciding what
  counts as a real threat."

Risk:

- Avoid dumping every shape rule. One or two visual examples are enough.

### 6. Renju Was Not A Regex

Claim: the old Renju forbidden check looked plausible until concrete examples
showed that legality depends on whether the branches are real lethal threats.

Conversation evidence:

- `arc_0921`: a specific report move starts the question: why is this forbidden?
- `arc_0922`: the response is not to patch the case, but to design and validate
  Renju rules against external references.
- `arc_0923`: the user connects proper forbidden checks to the same idea as
  lethal combo proof: real double-threes/fours, not rough shapes.
- `arc_0926` to `arc_0933`: Renju.net advanced examples are extracted,
  corrected, promoted to a corpus, and validated through external reference
  code.
- `arc_0935`: the new checker direction is clarified as a full replacement of
  the old checker, not an additive patch.
- `arc_0937` to `arc_0945`: correctness creates a performance regression, then
  metrics and fast paths are added to recover enough speed.

Commit/product evidence:

- `949c378` adds the Renju rule corpus.
- `626c1ae` removes a superseded one-off Renju regression after the corpus
  exists.
- `e4c1150`, `d188a30`, and `de45359` add legality metrics and fast paths.
- `/rules` now explains real double-three/double-four cases with board examples.

Best public shape:

- Compact technical post: "Renju rules were not a regex."
- Show one "looks forbidden, but is legal" example and one true forbidden
  example.

Risk:

- The implementation details get deep fast. Lead with the player-visible rule
  insight, not the checker recursion.

### 7. Rolling Frontier Was The Boring Infrastructure That Paid

Claim: corridor portals were exciting but did not promote. Rolling frontier was
less flashy, but it made threat-heavy search, hints, and analysis more
practical.

Conversation evidence:

- `arc_0667`: the user identifies full-board threat scans as the likely next
  bottleneck and asks for a design/tradeoff pass before implementation.
- `arc_0680`: metrics are planned first because the existing report/trace path
  can carry frontier timing and counters.
- `arc_0700`: the safety gate model is corrected from "probe then filter" to
  first-order "what threat must I respond to now?"
- `arc_0750`: later corridor-proof experiments still need metrics to answer
  candidate count and budget/headroom questions.

Commit/product evidence:

- `e65b767`, `1ea37f9`, `8149094`, and `2403876` introduce rebuild/shadow/lab
  rolling frontier modes and timing.
- `d6b69bc`, `5e9d120`, `b948224`, and `b0e2c2c` progressively optimize
  rolling frontier, immediate-win indexes, and pattern eval.
- `967aeed` promotes rolling frontier default after enough correctness/perf
  work.

Best public shape:

- Supporting section in a technical post. It explains why the analyzer and
  richer hints are viable in-browser.

Risk:

- This is a support story, not the headline. The user-visible headline remains
  "the game can explain where the finish came from."

### 8. Reports Became Receipts, Then Product Surfaces

Claim: reports started as internal diagnostics, then became proof of how the
project thinks and eventually public product surfaces.

Conversation evidence:

- `arc_0362`: report data/rendering are split so analysis can be iterated on
  without rerunning expensive tournaments.
- `arc_0612`: report metrics are refined when "branch probes" means something
  different than expected.
- `arc_1050`: report sampling is reconsidered around presets and useful match
  groups instead of dumping everything.
- `arc_1004`: the 0.5 cleanup hardens curated report artifacts and hides
  generated noise from diffs.
- `arc_1115`: public route/static-report reviews catch stale links and route
  mismatches.

Commit/product evidence:

- `f3fd48d`, `4a29544`, and `7aedeae` move published reports toward compact
  data plus web-rendered viewers.
- `9129a28` unifies the lab report viewer.
- `8f9e896` centralizes lab report artifacts.
- Current `/lab` merges bot and analysis reports into a public viewer.

Best public shape:

- "Reports are not screenshots of work; they are how the work happened."
- Pair old Rust-generated HTML artifact size/noise with current `/lab` route.

Risk:

- Do not make the product sound like a reporting tool. Reports should support
  replay analysis and bot transparency.

### 9. Public Pages Turned Strategy Vocabulary Into Product Copy

Claim: the lab vocabulary only became valuable after being translated into
concise public pages and replay UI.

Conversation evidence:

- `arc_1087` to `arc_1099`: the Rules page gets repeatedly tightened around
  real Renju threats, blocked branches, forbidden continuation, and 4+3.
- `arc_1100` to `arc_1111`: the Guide page is reorganized into avoid mistakes,
  make a combo, force a combo, and learn from replay analysis.
- `arc_1118` and `arc_1119`: replay analysis is surfaced as "Analyze" /
  "Inspect" so it is not hidden as ordinary replay playback.
- `arc_1120`: Visuals becomes a style guide, not just an asset inventory.

Commit/product evidence:

- `645e270` adds the in-app Rules page.
- `a4973ee` renders Guide boards with the game board.
- `3444c0f` polishes Rules.
- `55ba1a6` polishes Guide threat examples.
- `d1357d1` polishes Visuals.

Best public shape:

- Use current `/rules` and `/guide` as evidence that the lab terms became
  player-facing explanation, not just internal jargon.

Risk:

- Keep public copy concise. The pages work when they teach by showing board
  frames, not by becoming long lessons.

### 10. Housekeeping Was Part Of The Product Process

Claim: keeping the repo clean was not only aesthetic. It was how the project
kept AI-assisted breadth from turning into accumulated unreconciled debt.

Conversation evidence:

- `arc_0900`: a cleanup pass explicitly targets code/tests/docs and duplicated
  logic across analyzer/search use cases.
- `arc_1004` to `arc_1008`: v0.5 starts with reconciliation, generated-artifact
  guards, subagent reviews, and behavior-neutral cleanup.
- `arc_1030`: docs are reviewed for public-facing clarity after structure
  changes.
- `arc_1124` to `arc_1129`: 0.5.3 pivots toward docs/repo housekeeping before
  more feature work.

Commit/product evidence:

- `90cd355` trims slow analysis fixtures.
- `6ef8d22` guards curated report artifacts.
- `2e1ae81` cleans bot report plumbing.
- `299a851`, `ecdd339`, and related commits keep the corridor-proof/search code
  modular after experimentation.

Best public shape:

- Process lesson: when agents make it cheap to add, make it routine to review,
  cut, and reconcile.

Risk:

- This is a supporting process point. Do not lead with cleanup unless writing
  specifically about the AI-assisted workflow.

## Most Transferable Lessons

### Amplification Requires Ownership

Evidence beats: 2, 5, 6, 10.

Agents widened the practical stack surface and implementation throughput. The
distinctive project shape came from human review: rejecting weak UI framing,
catching Gomoku/Renju semantic errors, deciding which experiments were dead
ends, and keeping the repo/report/docs reconciled.

### Failed Experiments Are Only Useful If You Preserve The Insight

Evidence beats: 3, 4, 7.

Corridor portals and several bot-tuning paths did not promote cleanly. The
useful result was not the discarded implementation; it was the vocabulary,
metrics, and analyzer model that survived.

### Make Intermediate Artifacts When They Change Decisions

Evidence beats: 3, 8, 9.

Reports, screenshots, corpus fixtures, visual guides, and static pages were not
busywork because they changed what could be reviewed. The artifact is worth it
when it makes taste, correctness, or tradeoffs visible.

### Domain Knowledge Still Sets The Bar

Evidence beats: 5, 6, 9.

For a domain-heavy project, "just implement a bot" is too vague. The important
work is naming the model, finding counterexamples, and checking whether the
implementation means the same thing as the strategy term.

## Suggested Public Story Cuts

### Short README / Website Blurb

- Personal revival and modern stack.
- Replay analysis as the unique hook.
- Public lab as transparency, not raw benchmark bragging.

### Devlog 1: The Game After The Game

- Start from replay analysis.
- Explain last escape, setup corridor, lethal onset with one visual example.
- End by noting the bot-lab/report machinery that made it possible.

### Devlog 2: The Failed Bot Trick

- Corridor search as intended bot shortcut.
- Why it was too expensive/awkward in live search.
- How it became perfect for finished-game analysis.

### Devlog 3: Renju Was Not A Regex

- A concrete false-forbidden example.
- External corpus/reference validation.
- The payoff: rules page, analyzer, and bot legality now share a real model.

### Devlog 4: One Developer, Agents As A Small Team

- Handoff mode for clear implementation.
- Research mode for tradeoffs.
- Pair-logic mode for domain semantics.
- Producer mode for UI/copy/visual taste.
- The boundary: AI amplifies, but it should not replace critical thinking.

## Gaps To Fill Before Public Drafting

- Review the selected proof targets in `process_story_evidence_cards.md`.
- Capture the selected replay-analysis, Renju, lab, guide, visuals, and product
  screenshots from the evidence-card queue.
- Use before/after report screenshots only if the contrast is visually legible.
- Lead public packaging with product value: playable Gomoku/Renju, replay
  analysis, and the visible lab. Use process-first framing only for the
  making-of/devlog layer, proved through shipped product and lab surfaces.
- Re-run any dated benchmark numbers before publishing exact performance
  claims.
