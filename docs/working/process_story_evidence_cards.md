# Process Story Evidence Cards

Purpose: curate a small proof packet for the external Gomoku2D story.
Each card connects one public claim to private conversation arcs, current
product/lab artifacts, and a concrete screenshot target.

Status: selected evidence targets, not final public copy. Screenshot capture is
a follow-up pass.

## Editorial Defaults

- Lead public product surfaces with the game and replay analyzer. Use the
  process-first angle only for making-of/devlog copy, and prove it with shipped
  product and lab surfaces.
- Do not publish raw transcript screenshots or long quote dumps.
- Prefer product/lab receipts over chat excerpts.
- Keep the recurring thesis explicit: agents widened the work surface; human
  judgment kept the project coherent.
- Keep model caveats visible: replay analysis is bounded explanation, not a
  full Gomoku solver.

## Card 1: The Playable Game Still Comes First

Public claim:

The project is not only a lab demo. It is a playable local-first Gomoku/Renju
game with a board-first product loop.

Why it matters:

The process story needs a product anchor. Without this, the story turns into
generic AI-workflow commentary.

Private arc evidence:

- `arc_0179`: README framing was corrected so a new reader understands the game
  before repo mechanics.
- `arc_0180`: the product hook moved toward a retro Gomoku game with a modern
  stack.
- `arc_0002`: settings UI is framed around the board because the board is the
  main stage.

Product/lab proof:

- Root README and home page present "An old favorite, built properly."
- Web app supports immediate guest play, bot settings, Renju/Freestyle, replay
  history, and optional cloud continuity.

Screenshot target:

- Hero capture: home page plus an active board.
- Optional mobile board capture only if the public post needs a mobile proof.

Claim guard:

Do not lead with architecture before the reader sees the game loop.

## Card 2: Agents Widened The Work Surface; Judgment Kept It Coherent

Public claim:

One developer used agents like a small production team, but the important
decisions still came from taste, scope control, and domain judgment.

Why it matters:

This is the making-of hook. It keeps the story away from both "AI did it" and
anti-AI framing, but it should sit on top of visible product proof.

Private arc evidence:

- `arc_0001`: agent asked to fix a half-finished mobile/Phaser sizing problem
  by tracing the root layout path instead of patching symptoms.
- `arc_0057`: user rejects purely functional UI work and redirects toward a
  visual/design guide.
- `arc_0061`: design is stress-tested before implementation.
- `arc_1004`: v0.5 reconciliation splits report/artifact cleanup into reviewed
  commits.

Product/lab proof:

- `/visuals/` proves visual language became inspectable.
- `/rules/`, `/guide/`, and `/lab/` prove agent-assisted breadth was turned
  into coherent public surfaces, not just more code.
- Current repo cleanup and generated-artifact guards show the process includes
  reconciliation, not only generation.

Screenshot target:

- Contact sheet: `/rules/`, `/guide/`, `/lab/`, `/visuals/`.
- Optional: a compact release/commit timeline instead of any chat screenshot.

Claim guard:

Do not say agents owned the product. Say they expanded throughput and surface
area while the human kept deciding what mattered.

## Card 3: The Lab Made Decisions Inspectable

Public claim:

The Rust bot lab made bot and analyzer decisions measurable instead of
vibes-driven.

Why it matters:

This explains why the project has a public Lab page and why reports are part of
the product identity.

Private arc evidence:

- `arc_0350`: bot-config UI was deferred until the lab proved which knobs
  mattered.
- `arc_0362`: report data/rendering were separated so expensive tournaments did
  not need reruns for presentation tweaks.
- `arc_0403`: failed experiments were treated as evidence that the target was
  wrong, not only that an implementation needed tuning.

Product/lab proof:

- Current `reports/lab/bot-report.json`: `1,792` matches, `8` bot
  configurations, seed `63`, pooled budget tournament.
- `/lab/` ranking/search tabs show standings, search timing, and report
  provenance.
- v0.5 report rewrite moved presentation out of Rust-generated HTML and into
  the web viewer.

Screenshot target:

- `/lab/` ranking tab with top entrants.
- `/lab/?tab=search` with search-time split bars.

Claim guard:

Do not pitch strongest-bot claims. The point is inspectability and disciplined
iteration.

## Card 4: The Failed Bot Trick Became Replay Analysis

Public claim:

Corridor search did not promote as a broad live-search shortcut, but it became
the analyzer's vocabulary for explaining finished games.

Why it matters:

This is the strongest narrative beat: conflict, failed direction, salvage, and
visible product payoff.

Private arc evidence:

- `arc_0504`: forced-line/corridor work is pivoted to analysis first.
- `arc_0516`: analyzer goal becomes feeding a replay and walking backward
  through bounded forced play.
- `arc_0532`: escape proof is narrowed to whether the losing side had a legal
  alternative that stops the line without immediately losing elsewhere.
- `arc_0621`: replay-analysis policy is simplified around corridor replies.

Product/lab proof:

- Current `reports/lab/analysis-report.json`: preset-triangle analysis with
  `192` analyzed games and `0` analysis-generation failures.
- Selected proof target:
  `match_0065__search-d1__vs__search-d3_pattern-eval`, a short 18-move game
  with lethal onset at ply `16`, displayed failure step at ply `15`, report
  `last_chance_ply` at `14`, and setup corridor `15-16`.
- Replay UI and the Lab report use setup corridor, lethal onset, and last
  escape; `/guide/` teaches the same flow as combo onset, setup corridor, and
  last escape.

Screenshot target:

- `/lab/?tab=analysis&match=match_0065__search-d1__vs__search-d3_pattern-eval`.
- Replay Analysis timeline from a seeded finished game if a live replay example
  is easier to read than the report.

Claim guard:

Say corridor search failed in the live bot-search role under browser-scale
budgets. Do not say the concept failed.

## Card 5: Renju Correctness Forced Real Domain Modeling

Public claim:

Renju forbidden moves were not shape matching; they required proof of real
threat branches.

Why it matters:

This shows the domain was deep enough that generic execution could produce a
clean but wrong model.

Private arc evidence:

- `arc_0921`: a report move raises the question of why a candidate is forbidden.
- `arc_0923`: proper forbidden checks are connected to real double-three /
  double-four threat proof.
- `arc_0926` to `arc_0933`: Renju.net examples are extracted, corrected,
  promoted to a corpus, and validated.
- `arc_0935`: the new checker is clarified as a replacement, not a one-off
  patch.

Product/lab proof:

- `docs/reference/corpora/renju_corpus.md` documents the curated corpus.
- `/rules/` explains blocked/neutral double-three cases.
- Renju legality now feeds rules, hints, reports, and bot search.

Screenshot target:

- `/rules/` complex Renju section, preferably a blocked or forbidden-continuation
  double-three example.
- Optional: small crop of corpus board if it is visually legible.

Claim guard:

Do not bury the reader in RIF details. The public insight is "real threat
branches, not rough shapes."

## Card 6: Reports Became Receipts, Then Product Surfaces

Public claim:

Reports started as developer diagnostics, then became the way the project made
decisions and showed its work.

Why it matters:

This connects the lab to the product and explains why public reports are not
throwaway artifacts.

Private arc evidence:

- `arc_0362`: structured report output is requested to decouple data from
  rendering.
- `arc_0612`: report metrics are refined when "branch probes" was too ambiguous.
- `arc_1050`: analysis sampling is reconsidered around useful preset matchups.
- `arc_1004`: v0.5 hardens curated report artifacts and hides generated noise.

Product/lab proof:

- `/lab/` now merges tournament and replay-analysis reports into a web-rendered
  viewer.
- Published report JSON is compact and tracked under `reports/lab/`.
- Report routes share the public app visual language.

Screenshot target:

- Current `/lab/` report header plus one ranking/search/analysis detail crop.
- Optional before/after: old generated report artifact vs current Lab, only if
  the contrast is readable.

Claim guard:

Reports support the game. Do not make the product sound like a reporting tool.

## Card 7: Public Pages Turned Lab Vocabulary Into Player Language

Public claim:

The lab's tactical vocabulary only became valuable once it could be explained
through concise public pages and board diagrams.

Why it matters:

This proves the project did not stop at internal correctness. It translated the
model into player-facing understanding.

Private arc evidence:

- `arc_1087`: "real threes/fours" copy is corrected to focus on actual lethal
  result, not vague continuation.
- `arc_1100`: Guide flow is reorganized around avoid mistakes, make a combo,
  force a combo, and replay analysis.
- `arc_1118`: replay-analysis entry points are renamed so the feature is not
  hidden as ordinary replay playback.

Product/lab proof:

- `/rules/` teaches Freestyle/Renju and real forbidden threats.
- `/guide/` teaches immediate threats, imminent threats, combos, forced
  sequence, and replay analysis.
- Home/Profile/Replay labels now surface `Analyze`, `Inspect`, and `Replay
  Analysis`.

Screenshot target:

- `/guide/` forced-sequence frame.
- `/rules/` forbidden-move examples.

Claim guard:

Keep examples concise. The public pages work because they show board states
instead of becoming long strategy essays.

## Card 8: Cleanup Made Agent-Assisted Work Sustainable

Public claim:

AI-assisted breadth creates surface area; cleanup and reconciliation are what
keep it trustworthy.

Why it matters:

This is the process lesson that prevents the story from sounding like pure
velocity.

Private arc evidence:

- `arc_0900`: cleanup pass targets code, tests, docs, and duplicated logic after
  analyzer/search changes.
- `arc_1004`: v0.5 reconciliation creates generated-artifact guards and report
  plumbing cleanup.
- `arc_1124`: 0.5.3 pivots toward repo housekeeping before more public
  packaging.

Product/lab proof:

- `.gitattributes` and report publishing checks keep curated report artifacts
  explicit.
- `docs/README.md` now separates reference, working, and archive docs.
- `reports/lab/` centralizes curated report JSON.

Screenshot target:

- Release/commit timeline card.
- Optional: concise diff/stat or docs index crop, not raw terminal output.

Claim guard:

Do not frame cleanup as bureaucracy. Frame it as preserving trust after a wide
agent-assisted work loop.

## Capture Queue

Capture these after the evidence cards are reviewed:

1. Home plus active board.
2. `/lab/` ranking tab.
3. `/lab/?tab=search` with timing bars.
4. `/lab/?tab=analysis&match=match_0065__search-d1__vs__search-d3_pattern-eval`.
5. `/rules/` complex Renju example.
6. `/guide/` forced-sequence example.
7. `/visuals/` style guide overview.
8. Compact release/commit timeline graphic.

## Do Not Use

- Raw transcript screenshots.
- `quote_candidates.md` excerpts without manual review.
- Dense full-page tournament tables.
- Solver-strength claims.
- Full old generated HTML reports unless cropped enough to be readable.
