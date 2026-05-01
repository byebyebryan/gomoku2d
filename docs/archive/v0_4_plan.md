# `v0.4` Working Plan

Status: ad-hoc planning note. This doc captures the current direction before
implementation. `docs/roadmap.md` remains the canonical phase-level roadmap.

## Frame

`v0.4` should make the bot lab visible in normal play before trying to sell a
full analysis product. The first pass should stay honest about what the current
bot can actually support: discover useful bot knobs first, then expose the ones
that are proven enough for players.

The product story is:

> The bot is no longer an opaque practice opponent. The lab first proves which
> behaviors and presets are real, then the UI turns those validated choices into
> normal play settings. The same plumbing later powers replay review, puzzles,
> and "save this game" practice.

This keeps the line connected to the `v0.4` roadmap goal, but starts with the
current primary play mode: local play against the practice bot.

## Goals

- Make bot identity and behavior understandable to players.
- Explore which bot configuration dimensions are actually meaningful before
  designing UI around them.
- Add a proper settings surface so Profile does not become the dumping ground
  for every preference and customization option.
- Bridge bot configuration across Rust, Wasm, web state, saved-match metadata,
  and future cloud profile/settings sync.
- Create enough preset/config structure that normal play and later analysis can
  choose the right bot without hard-coding another one-off path.
- Keep `v0.4` exploratory: plan the first slices tightly, then let replay
  analysis, puzzles, and save-this-game compete based on what feels strongest.

## Non-goals

- No online PvP, matchmaking, ranked mode, or trusted match authority.
- No public replay sharing or published puzzle feed yet.
- No server-side strong bot unless browser-side Wasm is clearly not enough.
- No raw engine-settings dashboard for normal users. Raw knobs can exist in the
  lab/CLI while their value is being proven.
- No promise that every `v0.4.x` slice is known up front.

## `0.4.0` — Bot Lab Discovery Pass

Purpose: figure out which bot knobs and styles are real enough to productize.

Expected work:

- Add a real Rust `SearchBotConfig` or equivalent config object.
- Keep the initial experiments in Rust CLI, eval tooling, tests, and benchmarks.
- Try strength and behavior levers before committing to product vocabulary.
- Add behavior-oriented scenarios for bot identity, not just legal-move checks.
- Tune candidate presets against benchmark scenarios, curated positions, and
  self-play.
- Consider style levers such as offense/defense weighting, threat blocking
  priority, candidate ordering, or evaluation profile.
- Keep the default practice bot safe: it must still block obvious wins and
  avoid dumb tactical mistakes.
- Keep outputs deterministic enough for repeatable benchmarks and future
  analysis.

Candidate preset directions:

- `Fast`: quick, weaker, useful on slow devices or when the user wants speed.
- `Balanced`: default practice bot, tuned for general practice.
- `Deep`: slower, stronger, useful for review-like play if performance allows.
- `Aggressive`: prefers initiative, extensions, and attacking threats when it
  can do so safely.
- `Defensive`: more eager to block and stabilize, useful for learning attack
  patterns.

Open design questions:

- Are aggressive/defensive presets feasible with the current search/eval model,
  or do they collapse into shallow labels?
- Are strength presets enough for `0.4.1`, with style presets deferred until
  they genuinely feel different?
- Which knobs belong in durable saved-match metadata versus lab-only traces?
- Does analysis need a separate preset from normal play, or can `Deep` cover
  the first review pass?

Acceptance:

- Presets or candidate configs have documented differences.
- Benchmarks cover at least speed, tactical correctness, and a few
  behavior-shaping positions.
- The team can say which bot dimensions are worth exposing and which are lab
  internals.
- No player-facing settings UI is required yet.

## `0.4.1` — Settings And Bot Config Surface

Purpose: make customization real once the preset vocabulary is grounded.

Expected work:

- Add a dedicated settings page/panel or equivalent settings surface.
- Move or mirror current preference-like controls there, starting with default
  rule and validated bot preset.
- Keep Profile focused on identity, sync state, reset/delete, and history.
- Expose named presets that survived the `0.4.0` discovery pass.
- Show active bot identity in Play player cards, for example
  `Practice Bot · Balanced`.
- Save exact bot identity/config in match metadata.
- Extend the web/Wasm bridge so the app can construct bots from a config object,
  not only `createBaseline(depth)`.
- Keep advanced bot knobs in Rust CLI/docs or an explicitly debug-only path.

Open design questions:

- Settings should probably be reachable from the shared top nav, but the exact
  entry point and mobile treatment should be designed before implementation.
- Default rule and bot preset likely belong to local/cloud profile settings
  eventually; decide whether `0.4.1` changes persistence shape immediately or
  starts local-only and syncs later.
- Decide whether Reset/Delete profile actions stay in Profile or move to a
  danger zone inside Settings. Do not move them just because the page exists.

Acceptance:

- A player can see which bot they are playing.
- A player can choose a bot preset without understanding search depth.
- A saved match records which bot preset/config was used.
- The web UI only exposes presets that `0.4.0` proved meaningful enough.
- Existing local-first and signed-in profile flows keep working.

## Later `0.4.x` Exploration

After `0.4.0` and `0.4.1`, pick the next slice based on what feels strongest.

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

This is intentionally loose. `v0.4` is exploratory, and the right product shape
should come from trying one polished lab-powered surface rather than planning a
large analysis suite up front.

## Implementation Notes

- Keep the lab-to-product path explicit: Rust bot config -> Wasm bridge -> web
  settings -> match metadata -> replay/analysis consumers.
- Do not expose a bot preset just because it sounds useful. Expose it only when
  the lab pass can demonstrate behavior or performance differences.
- Keep casual/local and signed-in/cloud profile settings aligned, but do not
  introduce backend complexity unless the setting needs cloud continuity.
- Prefer named presets for the product UI. Raw knobs are for the lab.
- Treat benchmarks as part of the feature, not a cleanup step.
- Avoid UI that looks like a debug panel. Analysis should explain gameplay,
  not expose engine internals.
