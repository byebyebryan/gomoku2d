# v0.4.5 Bot Controls Plan

Purpose: turn the `0.4.x` bot-lab foundation into a player-facing bot control
surface without starting replay analysis UI yet.

## Context

`0.4.0` through `0.4.4` were deeper lab releases than originally expected. The
line did not produce a clean "corridor-powered strongest bot" story, but it did
produce a stronger product foundation:

- explicit `SearchBot` configuration and reproducible lab specs;
- tested easy/default/hard-side anchor candidates;
- tournament reports with provenance, cost, and ranking data;
- corridor-search vocabulary and analysis reports;
- rolling frontier, cached pattern evaluation, pooled budgets, and cleaner
  tactical ordering;
- enough negative evidence to retire corridor portals and raw knob chasing.

The next useful product step is not more lab tuning. It is exposing the bot as a
thing the player can understand and shape.

## Product Thesis

Gomoku2D should not only say "we made a stronger bot." The more distinctive
story is:

> The bot is configurable and inspectable because there is a real Rust bot lab
> behind it.

This means `0.4.5` should make the lab visible in the app, but in a controlled
way. Normal players need tested presets. Curious players should be able to open
an advanced layer and see the engine dimensions that make the bot behave
differently.

## Scope

`0.4.5` should focus on bot controls only:

- product-facing bot preset selection;
- advanced/explicit bot configuration;
- persistence for the chosen bot config;
- clear copy that connects the controls to the bot lab;
- device-local touch control for mobile pointer placement;
- human-only defensive hints for opponent imminent threats and counter-threat
  replies;
- local match plumbing so the selected config actually drives the practice bot;
- saved-match identity snapshots that preserve which bot config was used.

Replay analysis, critical-moment tagging, puzzle generation, and explanation
overlays stay out of this slice. They can build on the selected bot configs
later, but they should not compete with the settings/control surface now.
Defensive imminent-threat and counter-threat hints are in scope because they
reuse the same tactical vocabulary as the bot lab while staying inside the
existing board hint surface.

## Rough Work Plan

### 1. Config Model

Create a web-facing practice-bot config model before building UI on top of it.

- Define preset IDs and advanced config fields in TypeScript.
- Resolve presets and advanced controls into one runtime bot spec.
- Generate a reproducible lab spec string from the same model.
- Keep product config separate from report-only diagnostics.

### 2. Persistence And Saved-Match Identity

Make the selected config durable and auditable.

- Add bot config to local profile settings.
- Keep the first slice local-only unless cloud profile settings are deliberately
  expanded later.
- Update saved-match bot identity snapshots so replays/history preserve which
  bot config was used.
- Keep older saved-match hydration compatible enough for existing local test
  history, or make an explicit clean-break decision before changing schema.

### 3. Wasm And Worker Plumbing

Let the selected config drive actual bot play.

- Extend the web `BotSpec` beyond `{ kind: "baseline", depth }`.
- Add wasm/worker constructors or config plumbing for tactical cap, pattern
  eval, and corridor proof.
- Prefer structured config over parsing raw lab spec strings in the browser.
- Keep generated lab specs as display/reproducibility output.

### 4. UI Surface

Build a dedicated practice-bot control surface.

- Add a compact entry point from Home near `Play`.
- Add a compact next-game entry from Local Match.
- Use a dedicated route or panel for the full preset/advanced controls.
- Keep Profile focused on identity, cloud state, stats, and history.

### 5. Preset And Advanced Copy Polish

Make the controls readable as product UI.

- Use named preset cards with short player-facing descriptions.
- Keep advanced controls constrained and explained.
- Show generated lab spec as secondary detail.
- Warn that presets are tested while custom configs are experimental.

### 6. Validation

Treat this as a product-control release, not a new bot-strength release.

- Unit-test config resolution, persistence, and saved-match snapshots.
- Smoke-test local play for each preset.
- Verify the web build still works without cloud config.
- Run a small bot-lab sanity check only if preset mappings change from the
  current report-backed picks.
- Update roadmap, changelog, and release notes around "bot controls" rather
  than "stronger bot."

## UI Model

Use two layers.

### Preset Layer

This is the default player path. It should be accessible and stable. The UI can
use more interesting names, but it should still expose plain difficulty meaning
through short descriptions or small tier labels.

Current recommendation from the `v0.4.4` published report:

| Preset name | Tier | Lab spec | Why |
|---|---|---|---|
| `Easy` | easy | `search-d1` | Extremely fast (`~1 ms/move`) and clearly weaker, while still protected by the current safety gate. |
| `Normal` | standard | `search-d3+pattern-eval` | A stronger everyday bot than raw D3, still cheap (`~271 ms/move`), and it beat raw D3 `48-16` head-to-head in the current report. |
| `Hard` | hard | `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4` | Current top-ranked anchor; best expression of the tactical/corridor branch, at roughly `550 ms/move` in the pooled report. |

Hold back a fourth default preset for now. The next candidate would be an
expert lane, but the current D5/D7 corridor lanes are close enough that adding
both may imply a cleaner strength ladder than the data supports. Keep the D5
corridor lane available in advanced controls instead:

```text
search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4
```

That config ranked third overall, is slightly cheaper than the D7 corridor
variant, and remains useful for advanced users who want a wide tactical search.

Preset copy should describe player experience, not raw lab syntax. For example:

- Easy: "Fast and forgiving."
- Normal: "Balanced practice with stronger board-shape scoring."
- Hard: "Slower, sharper, and more punishing."

### Advanced Layer

This is the lab-character path. It exposes the bot's inner workings without
making the main UI feel like a debug dashboard.

V1 controls:

- search depth: how many plies the bot tries to search;
- search width / tactical cap: how much breadth it keeps after ordering;
- pattern scoring: stronger board-shape evaluation;
- corridor proof: post-search forced-sequence proof attempts.

Do not keep this loose. Advanced config should be versioned and deliberately
small so old profiles remain understandable later:

```ts
type PracticeBotConfigV1 =
  | { version: 1; mode: "preset"; preset: "easy" | "normal" | "hard" }
  | {
      version: 1;
      mode: "custom";
      depth: 1 | 3 | 5 | 7;
      width: "none" | 8 | 16;
      patternScoring: boolean;
      corridorProof: boolean;
    };
```

`corridorProof: true` resolves to the current report-backed
`corridor-proof-c16-d8-w4` profile. If that underlying profile needs to change
as product behavior, bump the config version instead of silently changing old
saved settings.

Avoid exposing validation or fallback internals as normal controls:

- `rolling-frontier-shadow`;
- `scan-threat-view`;
- raw safety-gate ablations;
- retired portal/leaf-extension suffixes;
- tournament-only CPU budget modes.
- candidate radius/source;
- raw thinking budget.

The advanced layer should show a generated lab spec for transparency and
reproducibility, but the spec string should not be the primary UI:

```text
search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4
```

Primary label: `Hard + Pattern + Corridor Proof`

Secondary detail: the exact lab spec.

## Information Architecture

Do not put this inside Profile. Profile is the player record, cloud state, and
history screen. Bot controls are match setup / practice configuration.

Reasonable surfaces:

- a dedicated settings or practice-bot route;
- a compact entry point from Home near the Play action;
- a compact entry point from Local Match for "next game" bot changes;
- a passive current-opponent summary on Home and Local Match.

The surface should feel like configuring an arcade opponent, not administering
an account.

## Data And Persistence

The selected bot config is local-first:

- store the versioned product config in local profile settings first;
- include it in cloud profile settings for signed-in users through the existing
  deferred sync path;
- snapshot the resolved bot identity/config into each saved match, just like
  current saved-match bot identity snapshots.

Presets should resolve to explicit lab specs. Advanced controls should also
round-trip through the same resolved config path so saved matches, replays, and
reports can all identify what was played. The generated lab spec is derived
output; do not persist a raw lab-spec string as the user setting.

## First Implementation Slice: Config, Plumbing, Persistence

Before UI work, land the bot config contract end to end. This keeps later
screens simple: they only read/write a product config, and the runtime/saved
history layers know how to resolve it.

### Target Shape

Add a web-owned practice bot config model with two product modes:

- `preset`: one of `easy`, `normal`, or `hard`;
- `custom`: the constrained `PracticeBotConfigV1` advanced shape.

The web model should resolve to a runtime bot spec and to a lab spec string.
The lab spec is transparency/provenance, not the storage source of truth.

Initial preset mapping:

| Preset ID | UI label | Resolved lab spec |
|---|---|---|
| `easy` | `Easy` | `search-d1` |
| `normal` | `Normal` | `search-d3+pattern-eval` |
| `hard` | `Hard` | `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4` |

Keep advanced config constrained to fields we are willing to support as product
state:

- search depth;
- width: `none`, `8`, or `16`;
- pattern scoring on/off;
- corridor proof on/off, using the report-backed `c16-d8-w4` profile.

Do not expose report/debug axes in the product config: rolling shadow,
scan-threat-view, safety ablations, retired portal/leaf experiments, CPU budget
modes, candidate radius/source, raw thinking budget, or raw parser suffixes.

### Schema Decisions

Use clean schema bumps. There are no real online users yet, but local test
profiles exist and should not be silently broken.

- Local profile becomes `gomoku2d.local-profile.v4`.
- Local v4 imports deprecated `gomoku2d.local-profile.v3` once, preserving name,
  rules, stats, and match history while defaulting bot config to `normal`.
- The app stops writing v3 after import; if the storage API is available, remove
  the v3 key after a successful v4 write to avoid repeated imports.
- Unknown or invalid bot config sanitizes to `normal`.
- Cloud profile becomes schema v4 with `settings.practice_bot`.
- Existing cloud v3 documents are tolerated and upgraded on next profile write.
- Cloud writes still use the current coalesced sync cadence; changing bot config
  must not add per-click Firestore writes.
- Profile settings store the product config only; they do not store generated
  lab specs.
- Saved matches use a new bot identity snapshot that records the resolved bot
  config and generated lab spec.
- Legacy saved-match bot identity remains readable and maps to the old depth-3
  baseline for local history compatibility.

### Code Slices

1. Add a focused web config module.

   Files:

   - create `gomoku-web/src/core/practice_bot_config.ts`
   - create `gomoku-web/src/core/practice_bot_config.test.ts`
   - modify `gomoku-web/src/core/bot_protocol.ts`

   Responsibilities:

   - define preset IDs and custom config types;
   - provide `DEFAULT_PRACTICE_BOT_CONFIG`;
   - sanitize unknown persisted values to `normal`;
   - reject loose/raw lab-spec persistence;
   - resolve product config to worker `BotSpec`;
   - generate the display lab spec.

2. Add runtime plumbing without UI.

   Files:

   - modify `gomoku-web/src/core/bot_worker.ts`
   - modify `gomoku-web/src/core/bot_runner.ts`
   - modify `gomoku-bot-lab/gomoku-wasm/src/lib.rs`

   Responsibilities:

   - keep `createBaseline(depth)` for compatibility;
   - add a structured search-bot constructor for depth, child cap, pattern eval,
     and corridor proof;
   - avoid parsing lab spec strings in the browser worker.

3. Bump local profile settings.

   Files:

   - modify `gomoku-web/src/profile/local_profile_store.ts`
   - modify `gomoku-web/src/profile/local_profile_store.test.ts`

   Responsibilities:

   - add `settings.practiceBot`;
   - migrate deprecated local v3 storage into v4;
   - preserve existing v1 saved matches during migration;
   - default missing bot config to `normal`.

4. Bump saved-match bot identity.

   Files:

   - modify `gomoku-web/src/match/saved_match.ts`
   - update cloud/local saved-match tests that import practice-bot constants

   Responsibilities:

   - add a new resolved practice-bot identity snapshot;
   - snapshot preset ID, UI label, engine/config version, structured config, and
     generated lab spec;
   - keep legacy depth-3 identity readable for old local history.

5. Bump cloud profile schema and rules.

   Files:

   - modify `gomoku-web/src/cloud/cloud_profile.ts`
   - modify `gomoku-web/src/cloud/cloud_profile.test.ts`
   - modify `firestore.rules`
   - modify `gomoku-web/src/cloud/firestore_rules.rules.ts`

   Responsibilities:

   - add `settings.practice_bot`;
   - accept/upgrade schema v3 to v4 in app code;
   - validate exactly the constrained `PracticeBotConfigV1` shape in Firestore
     rules;
   - keep cloud sync cost-neutral by using existing deferred writes.

6. Wire local matches to the selected config.

   Files:

   - modify `gomoku-web/src/game/local_match_store.ts`
   - modify `gomoku-web/src/routes/LocalMatchRoute.tsx`
   - update related local match tests

   Responsibilities:

   - pass the resolved bot spec into the match store;
   - snapshot the selected bot identity into finished matches;
   - keep current default behavior equivalent to `Normal`.

### Verification For This Slice

Run targeted checks before UI work starts:

- `npm --prefix gomoku-web test -- practice_bot_config local_profile_store saved_match cloud_profile cloud_match cloud_promotion`
- `npm --prefix gomoku-web run test:rules`
- `npm --prefix gomoku-web run typecheck`
- `cargo test --manifest-path gomoku-bot-lab/Cargo.toml -p gomoku-wasm`
- `npm --prefix gomoku-web run build`

The first slice is done when a hard-coded or test-injected `Easy`/`Normal`/`Hard`
config can drive the worker and be persisted/snapshotted correctly, even before
there is a visible settings page.

## Copy Direction

Use product language first and lab language second.

Good:

- "Practice Bot"
- "Bot Lab"
- "Depth"
- "Width"
- "Pattern scoring"
- "Corridor proof"
- "Generated lab spec"

Avoid:

- presenting `SearchBotConfig` field names as UI labels;
- calling raw knobs "AI" settings;
- implying corridor proof makes the bot a solver;
- suggesting every advanced config is tested equally.

The advanced layer needs a modest warning:

> Advanced bot settings are experimental. Presets are tested; custom configs may
> be slower, weaker, or just different.

## Done When

`0.4.5` is complete when:

- players can choose a tested bot preset before or between local bot matches;
- advanced users can inspect and adjust the main bot dimensions;
- the selected config persists locally and drives the actual practice bot;
- mobile players can choose pointer-style or touchpad-style cursor movement;
- saved matches record the bot config used;
- the UI connects back to the bot-lab identity without exposing report-only
  diagnostics;
- no replay-analysis UI ships in this slice.

## Open Design Questions

- Should the dedicated surface be `/settings`, `/bot`, `/practice`, or a modal
  from Home/Local Match?
- Should the advanced UI present the fixed v1 fields as a standalone custom
  form or as preset-plus-overrides?
- Should the web game remain uncapped for hard bots, or should advanced config
  show a soft "expected thinking time" based on lab reports?
