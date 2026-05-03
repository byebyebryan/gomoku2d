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

Likely work:

- Start from `search-d3` as the primary product-strength target.
- Use tactical facts for move ordering or staged candidate ranking, not broad
  leaf scanning.
- Keep immediate win/block safety explicit and cheap.
- Compare candidate radius, safety gate, and ordering changes independently.
- Promote a change only if it improves reached depth, runtime, or tournament
  strength under the same CPU budget while keeping tactical scenarios green.

Acceptance:

- The promoted change has focused tactical evidence and tournament evidence.
- The report surfaces the relevant pipeline metrics.
- Failed alternatives are removed from code and recorded in docs.

## `0.4.2` — Settings And Bot Config Surface

Purpose: make customization real once the preset vocabulary is grounded.

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

Pick the next slice based on what feels strongest after the bot pipeline has one
validated improvement and the UI has a place for settings.

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

- Keep the lab-to-product path explicit: Rust bot config -> Wasm bridge -> web
  settings -> match metadata -> replay/analysis consumers.
- Do not expose a bot preset just because it sounds useful. Expose it only when
  the lab pass can demonstrate behavior or performance differences.
- Keep casual/local and signed-in/cloud profile settings aligned, but do not
  introduce backend complexity unless the setting needs cloud continuity.
- Prefer named presets for the product UI. Raw knobs are for the lab.
- Treat benchmarks as part of the feature, not a cleanup step.
- Avoid UI that looks like a debug panel. Analysis should explain gameplay, not
  expose engine internals.
