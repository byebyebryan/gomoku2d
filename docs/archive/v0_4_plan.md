# `v0.4` Working Plan

Status: active planning note. This doc captures the `v0.4` pivot after the
first bot-lab experiments. `docs/roadmap.md` remains the canonical phase-level
roadmap.

## Frame

`v0.4` should make the bot lab visible in normal play, but the first step is not
to expose knobs just because they sound useful. The lab needs to prove which
knobs are real before the product turns them into settings, analysis, or
practice modes.

The product story is still:

> The bot is no longer an opaque practice opponent. The lab proves which
> behaviors and presets are real, then the UI turns those validated choices into
> normal play, replay review, puzzles, and "save this game" practice.

The implementation story changed during `0.4.0`: shallow tactical feature
experiments did not survive measurement. The stronger foundation is a measured,
explicit search pipeline plus behavior-preserving performance work.

## Goals

- Make bot identity and behavior understandable to players.
- Explore bot configuration dimensions through evidence before designing UI
  around them.
- Add enough lab/reporting structure that future bot changes can be evaluated
  without re-running ad hoc experiments.
- Bridge bot configuration across Rust, Wasm, web state, saved-match metadata,
  and future cloud profile/settings sync only after the useful knobs are clear.
- Keep `v0.4` exploratory: plan the first slices tightly, then let replay
  analysis, puzzles, and save-this-game compete based on what feels strongest.

## Non-goals

- No online PvP, matchmaking, ranked mode, or trusted match authority.
- No public replay sharing or published puzzle feed yet.
- No server-side strong bot unless browser-side Wasm is clearly not enough.
- No raw engine-settings dashboard for normal users. Raw knobs can exist in the
  lab/CLI while their value is being proven.
- No player-facing bot preset unless tournament/scenario evidence shows it is
  more than a label.

## `0.4.0` — Bot Lab Foundation

Purpose: establish the measurement, reporting, pipeline, and hot-path foundation
needed before productizing bot controls or analysis features.

Delivered scope:

- Explicit `SearchBotConfig` and stable lab specs such as `search-d2`,
  `search-d3`, and `search-d5`.
- Tactical scenario diagnostics for focused one-move probes.
- Multi-threaded tournament runs with CPU-time budgets, seeded openings, compact
  move capture, JSON output, and a reusable HTML report.
- Public bot-report publishing alongside existing asset previews.
- Search trace metrics for candidate source, legality gate, safety gate,
  alpha-beta nodes, safety-probe nodes, candidate counts, legality checks, eval
  calls, TT behavior, and budget exhaustion.
- A pipeline vocabulary: move source, rules legality, tactical annotation,
  optional safety gate, move ordering, alpha-beta search, and static eval.
- Rejected-experiment records for tactical candidates, tactical ordering, broad
  forced extension, and broad shape eval.
- Behavior-preserving core/search optimizations: Renju forbidden precheck,
  virtual immediate-win probes, bitboard board storage, and occupied-stone hot
  path iteration.

Intentional non-deliverables:

- No product settings page yet.
- No visible bot personality/style presets yet.
- No claim that the bot is meaningfully stronger because of one tactical
  shortcut.
- No full threat-space-search solver inside the practice bot.

Acceptance:

- Core, bot, eval, wasm, and web builds pass.
- The current bot/report docs explain what is measured, what failed, and what
  changed.
- A clean curated tournament report can be generated from a clean code commit
  and published as a follow-up artifact commit.
- `0.4.1` has a narrower search-improvement target instead of another broad
  tactical experiment.

## `0.4.1` — Tactical Ranking / Ordering Pass

Purpose: use the explicit pipeline to try one behavior-changing bot improvement
with real evaluation gates.

Work so far:

- Kept `search-d3` as the default baseline and compared it against shallow,
  uncapped-deep, capped-deep, and pattern-eval variants.
- Promoted the local-threat safety gate as the default cheap safety path.
- Used tactical facts for ordering and non-root child frontier caps rather than
  broad leaf scanning.
- Kept immediate win/block safety explicit and measured through hard tactical
  scenario gates.
- Compared candidate radius, safety gate, ordering, child caps, and static eval
  as separate lab axes before publishing the clean reference report.

Acceptance:

- The promoted change has focused tactical evidence and tournament evidence.
- The report surfaces the relevant pipeline metrics.
- Failed alternatives are removed from code and recorded in docs.

Current checkpoint:

- The active reference tournament set is now explicit:
  `search-d1`, `search-d3`, `search-d5`, `search-d5+tactical-cap-8`,
  `search-d7+tactical-cap-8`, `search-d3+pattern-eval`,
  `search-d5+tactical-cap-8+pattern-eval`, and
  `search-d7+tactical-cap-8+pattern-eval`.
- This set covers the easy/default/deep ladder, the current breadth-for-depth
  tactical-cap candidates, and the active pattern-eval ablation.
- A current tactical-scenario preflight across the non-pattern ladder
  (`search-d1`, `search-d3`, `search-d5`, `search-d5+tactical-cap-8`,
  `search-d7+tactical-cap-8`) passed all `20/20` hard safety-gate cases.
  Diagnostic misses remain expected and are not tournament blockers.
- The clean curated reference report has been generated and published from a
  clean `822045148556` provenance. It gives `0.4.1` a stable anchor for the
  depth ladder, tactical-cap hard-side candidates, and pattern-eval ablations.
- The current release decision is whether `0.4.1` should stop here as the
  bot-ladder/reporting checkpoint, or take one more narrow corridor-search
  slice.
  Do not add another broad tactical scan, broad leaf eval, or product-facing
  bot knob before that decision is explicit.

## `0.4.2` — Bot Exploration II

Purpose: use the stronger harness to do one more measured bot pass before
productizing settings. `0.4.1` gives us a real difficulty ladder and a usable
report; `0.4.2` should tighten that ladder and probe whether forced play can
create a more interesting practice opponent.

Expected work:

- Sweep existing knobs before adding new mechanisms: depth, child cap, candidate
  radius, pattern eval, and possible asymmetric candidate-source choices.
- Prefer `head-to-head` and batch `gauntlet` runs for tuning so the experiment
  matrix does not explode into every possible pairing. Batch gauntlets should
  play candidates against stable anchors only, not candidate-vs-candidate.
- Keep the published round-robin report as the anchor source; refresh it only
  after a candidate survives focused tests.
- Prototype bounded corridor search as lab-only, using local threat facts to
  derive concrete gain and defense replies instead of scanning the whole board
  at every leaf.
- Record explicit metrics for all non-alpha-beta work so "fewer nodes" cannot
  hide extra tactical cost.
- Treat bot style/character as a mechanism question, not a label question.
  Offensive and defensive variants should mean different budget allocation over
  own forcing chains versus opponent forcing-chain prevention.

Non-goals:

- No product settings page yet.
- No public offensive/defensive personality labels yet.
- No full TSS/proof search inside `SearchBot`.
- No broad tactical scan or broad leaf-eval experiment unless the focused
  harness proves why it is worth revisiting.

Current sweep status:

- Sweep A used a batch gauntlet, not a full round robin: `8` candidates against
  `8` clean `0.4.1` reference anchors, `32` games per pair, Renju,
  centered-suite openings, and `1000 ms` CPU time per move.
- Pattern eval remains the clearest strength signal but still has a wide cost
  range. Keep it lab-only until the survivor set is clearer.
- `tactical-cap-16` is not a general upgrade. The wider frontier often spends
  more budget without a clean score gain.
- `tactical-cap-4` is viable enough to keep testing because tactical ordering
  and the safety gate already remove many weak branches before the cap matters.
- Next step should be a smaller survivor comparison, not UI exposure.

Acceptance:

- The easy/default/hard ladder is either confirmed or refined with focused
  evidence.
- Any corridor-search prototype reports its own cost and either survives focused
  tournament checks or is removed and documented.
- Style language remains lab-internal unless it maps to a real search-budget
  mechanism.
- The next UI/settings slice has fewer raw knobs and clearer product presets to
  expose.

## `0.4.3` — Corridor-Bot Lab Pass

Purpose: test corridor search inside bot behavior before turning bot work into
product settings. `0.4.2` made corridor search a serious replay-analysis model;
`0.4.3` should find out whether that same model can guide live bot choices.

Detailed working notes live in
[`v0_4_3_corridor_bot_plan.md`](v0_4_3_corridor_bot_plan.md).

Expected work:

- Stabilize shared corridor-search entry points for analyzer and bot
  experiments.
- Add lab-only corridor-aware bot modes, aliases, or config flags.
- Try corridor move ordering, selective corridor extension, and escape-aware
  defense.
- Reinforce or optimize corridor search where integration exposes obvious cost
  or correctness gaps.
- Evaluate with tactical scenarios, focused tournaments, search-cost metrics,
  and replay-analysis reports.
- Keep all corridor knobs lab-only unless they become product language.

Non-goals:

- No settings page yet.
- No player-facing corridor-search controls.
- No full TSS/proof solver inside `SearchBot`.
- No offensive/defensive labels unless they map to real budget allocation.

Acceptance:

- We know whether corridor-aware search is a useful bot primitive, a deferred
  research thread, or mainly an analyzer feature for now.
- Any surviving mode has strength, cost, and replay-analysis evidence against
  current anchors.
- The next UI/settings slice has clearer product labels and fewer raw knobs.

## `0.4.4` — Settings And Bot Config Surface

Purpose: make customization real once the preset vocabulary is grounded by the
`0.4.1`, `0.4.2`, and `0.4.3` reports.

Expected work:

- Add a dedicated settings page/panel or equivalent settings surface.
- Move or mirror current preference-like controls there, starting with default
  rule and any validated bot preset.
- Keep Profile focused on identity, sync state, reset/delete, and history.
- Show active bot identity in Play player cards, for example
  `Practice Bot · Balanced`, only if the label is backed by lab evidence.
- Save exact bot identity/config in match metadata.
- Extend the web/Wasm bridge so the app can construct bots from a config object,
  not only one hard-coded depth.
- Keep advanced bot knobs in Rust CLI/docs or an explicitly debug-only path.

Open design questions:

- Settings should probably be reachable from the shared top nav, but the exact
  entry point and mobile treatment should be designed before implementation.
- Default rule and bot preset likely belong to local/cloud profile settings
  eventually; decide whether the persistence shape changes immediately or starts
  local-only and syncs later.
- Decide whether Reset/Delete Profile actions stay in Profile or move to a
  danger zone inside Settings. Do not move them just because the page exists.

Acceptance:

- A player can see which bot they are playing.
- A player can choose a bot preset without understanding search internals.
- A saved match records which bot preset/config was used.
- Existing local-first and signed-in profile flows keep working.

## Later `0.4.x` Exploration

Pick the next slice based on what feels strongest after the bot pipeline has a
validated ladder, corridor-aware bot work has been tested, and the UI has a
place for settings if settings are still the right bridge.

Likely candidates:

- **Replay critical moments:** Analyze a saved game and tag the moves where the
  game turned.
- **Better-move suggestions:** Show the bot's preferred move for a replay
  position and a short reason label.
- **Branch from review:** Turn an analyzed replay moment into a practice game.
- **Save this game:** Drop the player into a losing position and ask them to
  fight out.
- **Puzzle MVP:** Start with curated positions, then explore generated puzzles
  from saved games once the analysis pipeline is trustworthy.

Loose sequencing preference:

1. Replay analysis foundation.
2. Critical-moment UI.
3. Branch/save-this-game practice.
4. Puzzle MVP.

This remains intentionally loose. `v0.4` is exploratory, and the right product
shape should come from trying one polished lab-powered surface rather than
planning a large analysis suite up front.

## Implementation Notes

- Keep the lab-to-product path explicit: Rust bot config and analysis facts ->
  Wasm bridge -> web settings -> match metadata -> replay/analysis consumers.
- Do not expose a bot preset just because it sounds useful. Expose it only when
  the lab pass can demonstrate behavior or performance differences.
- Keep casual/local and signed-in/cloud profile settings aligned, but do not
  introduce backend complexity unless the setting needs cloud continuity.
- Prefer named presets for the product UI. Raw knobs are for the lab.
- Treat benchmarks as part of the feature, not a cleanup step.
- Avoid UI that looks like a debug panel. Analysis should explain gameplay, not
  expose engine internals.
