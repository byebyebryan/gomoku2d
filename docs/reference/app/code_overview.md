# Web Code Overview

This is the orientation map for `gomoku-web`. It is intentionally not a full
API reference. Use it to find the right owner for a change before editing.

## Runtime Shape

`gomoku-web` is the product surface. React owns routes, panels, stores, and
copy. Phaser owns only the board renderer. Rust/Wasm owns rules, bot moves, and
analysis facts.

```text
React routes and stores
  -> Board view model
    -> Phaser board renderer
  -> wasm bridge
    -> gomoku-core / gomoku-bot / gomoku-analysis
  -> workers
    -> bot search and replay analysis off the UI thread
```

The main rule is: UI code may present game facts, but should not invent rule,
threat, bot, or replay-analysis semantics.

## Entry Points

| Area | Files | Contract |
|---|---|---|
| App shell | `src/main.tsx`, `src/app/App.tsx`, `src/app/global.css` | Router, global CSS tokens, and top-level app lifecycle |
| Routes | `src/routes/*Route.tsx` | Product screens: Home, Match, Replay, Profile, Settings |
| Board component | `src/components/Board/Board.tsx` | React wrapper around the Phaser board scene |
| Board renderer | `src/board/*` | Phaser scene, sprites, overlays, input mapping, and animation timing |
| Game session | `src/game/local_match_session.ts`, `src/game/local_match_store.ts` | Current local match state, bot turns, clocks, hints, undo, completion |
| Profile/settings | `src/profile/*` | Local profile, persisted settings, local history, active-history selection |
| Cloud sync | `src/cloud/*` | Optional Firebase auth/profile/history stores and local-to-cloud promotion |
| Saved matches | `src/match/saved_match.ts` | Compact persisted replay/history schema |
| Shared history | `src/match/match_history.ts` | Local/cloud summary validation, retention, archive statistics and identity projection without Firebase |
| Replay UI | `src/replay/*`, `src/routes/ReplayRoute.tsx` | Replay reconstruction, timeline, analysis overlays, cache, and worker runner |
| Replay examples/help | `src/replay/replay_examples.ts`, `src/routes/GuideRoute.tsx` | Two provenance-backed portable bot games, Guide section navigation, and local/private/public action help; reuses the shared viewer |
| Lab reports | `src/routes/LabReportRoute.tsx`, `src/routes/AnalysisReportRoute.tsx`, `src/routes/lab-report/*` | Report loading/tabs, tournament drilldowns, analysis rows, proof boards, and help/provenance |
| Bot worker | `src/core/bot_runner.ts`, `src/core/bot_worker.ts`, `src/core/bot_protocol.ts` | Async bot move requests with cancellation and worker restart safety |
| Wasm bridge | `src/core/wasm_bridge.ts` | Typed JSON parsing boundary over `gomoku-wasm` exports |
| Static publishing | `scripts/publish_reports.mjs`, `scripts/publish_visual_design_assets.mjs`, `scripts/publish_spa_routes.mjs` | Copies curated data/assets and SPA entries into `dist/` during build |
| Playwright journeys | `playtests/*.spec.ts` | Browser-level acceptance for play, layout, replay, reports, recovery, and no-config paths |
| Cloud acceptance | `cloudtests/*`, `scripts/run_cloud_acceptance.mjs` | Isolated emulated Auth/Firestore application journeys and two-client races |
| Opt-in profiling | `perftests/*`, `scripts/profile_workloads.mjs` | Frozen/current-core-validated workloads, production worker timing, cache and lifecycle evidence |

## Wasm Boundary

`src/core/wasm_bridge.ts` is the only place that should parse raw JSON strings
from `gomoku-wasm`.

Current bridge exports used by web:

- `createWasmBoard(variant)` and `wasmBoardFromFenWithVariant(...)`
- `applyWasmMove(board, row, col)`
- `readWasmThreatSnapshot(board)`
- `readWasmWinningCells(board)`
- `createWasmBotFromSpec(spec)`
- `chooseWasmBotMove(bot, board)`
- `parseWasmReplayAnalysisStep(json)`

The bridge validates payload shape before data reaches React stores. Nested
analyzer validation lives in `src/replay/replay_analysis_validation.ts` and is
shared with persistent cache reads so retained data meets the same contract. If Rust
adds a new wasm payload field, update the validator and the matching TypeScript
type here first.

## Bot API

The browser bot contract is `BotSpec` in `src/core/bot_protocol.ts`.

Current product shape:

- `kind: "human"` for a human slot.
- `kind: "search"` for Rust `SearchBot`.
- `depth`, `childLimit`, `patternEval`, `corridorProof`, and `maxTtEntries`
  mirror the narrow web-safe subset of the lab config.

`BotRunner` owns worker lifecycle and request cancellation. A route/store should
ask for `chooseMove(...)`; it should not call `WasmBot` directly. If a future
API-backed bot is added, it should fit this async request/response shape rather
than bypassing the runner.

## Match State

`src/game/local_match_store.ts` owns active game behavior:

- board FEN and side-to-move state
- local human moves and bot replies
- undo and new-game setup
- per-player clocks
- tactical hint snapshots from wasm
- saved-match creation on completion
- failed bot-turn state and explicit retry after the runner's bounded recovery

`src/game/local_match_session.ts` owns the singleton session around that store:
saved setup application, replay-branch resume seeds, and current match reuse
across routes.

## Replay Analysis

Replay Analysis is intentionally progressive. The product enters it from
end-game `Analyze` or Profile `Inspect`, then streams analysis while preserving
normal replay navigation:

```text
SavedMatchV2
  -> replay_analysis_core.ts converts to core Replay JSON
  -> ReplayAnalysisRunner starts replay_analysis_worker.ts
  -> WasmReplayAnalyzer.step(...)
  -> replay_analysis_overlays.ts converts annotations to board overlays/status
```

Important boundaries:

- `replay_analysis_core.ts` owns saved-match to core-replay conversion.
- `replay_analysis_protocol.ts` mirrors the wasm JSON payload shape.
- `replay_analysis_runner.ts` owns cancellation and progress callbacks.
- `replay_analysis_overlays.ts` owns player-facing overlay/status translation.
- `replay_analysis_cache.ts` caches analysis results locally only.

The static analysis report is not loaded by the Replay page. The report is a
published lab artifact; the Replay page runs the same analyzer in wasm.

The [example corpus](replay_examples.md) copies only two complete move sequences
and small content metadata from that report. It preserves bot participants,
uses the ordinary portable codec/viewer and adds no cloud or played-history
side effects on open. Chosen example plies are link viewing state, not saved
library metadata.

## Board Rendering

`BoardViewModel` is the React-to-Phaser data contract. Route/store code should
construct a model and let `Board` / `board_scene.ts` render it.

The board layer owns:

- sprite loading and animation keys
- stone, hover, hint, marker, and highlighter z-order
- pointer/touchpad input mapping
- responsive canvas sizing
- an accessible DOM cell grid with keyboard navigation and placement
- visual-only animation timing

The board layer should not decide whether a cell is forbidden, threatening, or
an escape. Those facts come from the game store and wasm analysis snapshots.

## Persistence And Cloud

Local profile state is the default. Cloud is optional and layered on top.

- `local_profile_store.ts` persists local profile, settings, and local history.
- `src/match/match_history.ts` owns the shared history model. Guest persistence
  imports it directly; cloud record conversion and Firestore profile I/O remain
  in `cloud/`. The cloud profile facade retains its existing exported names.
- `best_effort_storage.ts` keeps profile and cloud-cache actions usable in memory
  after storage access or writes fail. A profile save failure shows a session-only
  notice; unreadable persistent data is not overwritten by a fallback profile.
- Local writes detect changes since this tab's last read and become session-only
  on conflict, preserving the newer saved document. Use one writing tab; this
  does not provide an atomic multi-tab merge.
- `profile_settings.ts` defines the persisted settings schema.
- `cloud_profile.ts` / `cloud_profile_store.ts` define cloud profile shape.
- `cloud_history.ts` / `cloud_history_store.ts` own private cloud history.
- `cloud_promotion.ts` moves local profile/history/settings into cloud after
  sign-in. `cloud_setup.ts` restores the account setup once before controller or
  background uploads; later local edits remain authoritative for that session.
- `cloud_promotion.ts` owns the promoted-history projection used by both sync
  preflight and the write path, including reset filtering and retention.
- Firestore rules live at repo root and are tested through `npm run test:rules`.
  `npm run test:cloud` separately exercises the application/SDK across isolated
  Auth/Firestore emulator clients using those rules.
- A no-op cloud flush retains the server write timestamp; only a real write
  closes the client sync gate.

Cloud loads and promotions return their own operation result or `null` after
invalidation. A background profile patch must supply the exact source snapshot;
the store rejects it after another load, reset, or patch. Follow-up history
actions also check the current authentication session.

Keep guest-local history, private cloud history, and future public/shared replay
surfaces separate.

## Common Change Map

| Change | Start here | Also check |
|---|---|---|
| Add a bot preset or advanced knob | `src/core/bot_config.ts` | `bot_protocol.ts`, `wasm_bridge.ts`, `gomoku-wasm`, Settings UI, saved-match snapshots |
| Change hints or overlays | `local_match_store_hints.test.ts`, `board_overlay_renderer.ts` | wasm `threat_snapshot`, sprite docs/previews |
| Change replay analysis UI | `ReplayRoute.tsx`, `replay_analysis_overlays.ts` | `replay_analysis_protocol.ts`, wasm analyzer tests, Playwright replay smoke |
| Change persisted settings | `profile_settings.ts`, `local_profile_store.ts` | cloud profile schema, Firestore rules tests, Settings UI |
| Change saved match schema | `src/match/saved_match.ts` | replay conversion, profile history, cloud history, Playwright replay tests |
| Change board visuals | `src/board/*`, `assets/sprites/*` | visual design reference, screenshot review |
| Change report presentation | `src/routes/lab-report/*`, `AnalysisReportRoute.tsx` | report JSON types, route tests, Playwright report smoke |
| Change static report publishing | `scripts/publish_reports.mjs` | release runbook and Playwright report smoke |

## Verification

Use the [testing command matrix](../ops/testing.md#command-matrix) for current
typecheck, unit, rules, browser, and cloud checks. Public behavior changes need
the relevant browser journeys; the full CI lane covers all three engines.

If the change crosses the wasm bridge, rebuild wasm from the repo root first:

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```

Then refresh the local dependency with `npm install` in `gomoku-web/` before
running web checks, as described in the [web README](../../../gomoku-web/README.md#local-development).
