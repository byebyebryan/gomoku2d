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
| Replay UI | `src/replay/*`, `src/routes/ReplayRoute.tsx` | Replay reconstruction, timeline, analysis overlays, cache, and worker runner |
| Bot worker | `src/core/bot_runner.ts`, `src/core/bot_worker.ts`, `src/core/bot_protocol.ts` | Async bot move requests with cancellation and worker restart safety |
| Wasm bridge | `src/core/wasm_bridge.ts` | Typed JSON parsing boundary over `gomoku-wasm` exports |
| Static publishing | `scripts/publish_*.mjs` | Copies visual-guide assets and curated reports into `dist/` during build |
| Playwright smoke | `playtests/*.spec.ts` | Browser-level release smoke for play, layout, replay, reports, and no-config paths |

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

The bridge validates payload shape before data reaches React stores. If Rust
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

`src/game/local_match_session.ts` owns the singleton session around that store:
saved setup application, replay-branch resume seeds, and current match reuse
across routes.

## Replay Analysis

Replay analysis is intentionally progressive:

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

## Board Rendering

`BoardViewModel` is the React-to-Phaser data contract. Route/store code should
construct a model and let `Board` / `board_scene.ts` render it.

The board layer owns:

- sprite loading and animation keys
- stone, hover, hint, marker, and highlighter z-order
- pointer/touchpad input mapping
- responsive canvas sizing
- visual-only animation timing

The board layer should not decide whether a cell is forbidden, threatening, or
an escape. Those facts come from the game store and wasm analysis snapshots.

## Persistence And Cloud

Local profile state is the default. Cloud is optional and layered on top.

- `local_profile_store.ts` persists local profile, settings, and local history.
- `profile_settings.ts` defines the persisted settings schema.
- `cloud_profile.ts` / `cloud_profile_store.ts` define cloud profile shape.
- `cloud_history.ts` / `cloud_history_store.ts` own private cloud history.
- `cloud_promotion.ts` moves local profile/history/settings into cloud after
  sign-in.
- Firestore rules live at repo root and are tested through `npm run test:rules`.

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
| Change board visuals | `src/board/*`, `assets/sprites/*` | `public/assets/sprites/*`, visual design reference, screenshot review |
| Change static report publishing | `scripts/publish_*` | release runbook and Playwright report smoke |

## Verification

Use the release-level stack when changing public behavior:

```sh
cd /home/bryan/code/gomoku2d/gomoku-web
npm run typecheck
npm test
npm run test:rules
GOMOKU_BASE_PATH=/ npm run build
npm run playtest:smoke
```

If the change crosses the wasm bridge, rebuild wasm first:

```sh
cd /home/bryan/code/gomoku2d
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
```
