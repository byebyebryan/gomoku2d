# FE Gap Analysis — Current State vs Backend Design

**Last updated:** 2026-04-18
**Context:** Cross-reference of the current `gomoku-web` frontend against the planned backend features in `docs/online_backend_design.md` and the project status in `docs/progress.md`.

---

## Phase mapping

The BE design phases and their actual FE prerequisites:

| BE Phase | Feature | FE blockers | Gap sections |
|----------|---------|-------------|--------------|
| **1** | Auth + profile | Firebase SDK init, auth state layer, auth UI in settings, profile model | §1, §2 |
| **2** | Replay sharing | WASM bridge for Replay, replay viewer, URL routing, share button, Firestore writes | §3, §7 |
| **3** | Match history | Multi-scene navigation, profile/history scene, global state layer | §4, §5 |
| **4** | Cloud Run + username | API client, username reservation UI, error handling | §6 |
| **5** | Replay verifier | Mostly backend; FE needs verified badge UI | §3 |
| **6+** | Leaderboard / puzzle / correspondence | New scenes, notification system, lobby UI | §4, §8 |

Sections below are ordered by BE phase relevance, not severity.

---

## 1. Architectural prerequisites for auth (BE phase 1)

These are the actual blockers for the first BE phase. None of them require multi-scene architecture.

### No auth integration path

The entire UI renders inside a Phaser canvas, and there is currently no auth integration at all. A Phaser button can still trigger Firebase Auth popup/redirect flows, so a DOM overlay is not strictly required just to sign in. The actual gap is that there is no auth service layer, no sign-in/out actions wired to UI, no auth-state subscription, and no pattern for reflecting auth state back into the game UI.

### No global state outside Phaser lifecycle

All game state lives in `GameScene`'s private members. Auth state (`uid`, provider data) needs to survive scene transitions, be accessible from multiple scenes, and persist when no scene is active. Currently there's nowhere to put it — no global store, no service objects, no scene-independent state layer. `main.ts` (29 lines) just creates the Phaser game. This is a prerequisite for any online feature.

### No Firebase SDK or auth UI

- **No Firebase SDK.** The only runtime dependencies are `phaser` and the local `gomoku-wasm` binding. No `firebase-auth` package.
- **No "Sign in" UI.** No button, no flow, no provider picker.
- **No auth state tracking.** No concept of "signed in" vs "anonymous" in the scene.
- **Player identity is purely local.** `profiles[]` is `{name, wins, isHuman}` — no uid, no avatar, no provider data.
- **No profile persistence.** Names, settings, and win counts are in-memory only. `profiles[].wins` survive the RESET button path but are explicitly zeroed when settings change triggers `applySettingsAndRestart()`. Nothing persists across page reloads.

**Phase 1 FE needs:**

- Firebase Auth init (new `core/auth.ts` or in `main.ts`)
- Auth state observer exposing `uid | null` at global scope
- Sign-in buttons in settings panel (the existing panel can accommodate this — it's a container with rows)
- Anonymous auth on first load (invisible to user)
- Profile model expanded: `uid`, `avatar_url`, `auth_providers[]`, `username`

**Open questions in the BE design that affect FE:** whether username is required at sign-in or only when entering public features, whether public profiles ship in v1, whether anonymous sessions get garbage-collected. These are deferred in the BE doc and should stay deferred here too.

---

## 2. Settings panel capacity

Current settings cover: variant, player type toggle (player name / BOT), name editing. These are the only controls — `SettingsPanel.getValues()` returns `{variant, p1IsHuman, p2IsHuman, p1Name, p2Name}`.

**For phase 1**, the existing panel can accommodate auth controls (sign-in button, auth status, link/unlink provider) as additional rows. The `ToggleGroup` and `TextButton` components are generic and reusable.

**For later phases**, the panel will need:

- **Bot difficulty picker** — Progress.md has a detailed plan for `bot_presets.ts` with RANDOM/EASY/MEDIUM/STRONG. No BE dependency; ready to implement now. The BE's "stronger bot" feature (unscheduled) would add an API-routed preset (name TBD — the BE doc uses "CHAMPION" as an example, not a committed name).
- **Username field** — for the Cloud Run username reservation endpoint (phase 4).
- **Cloud-synced settings** — listed in the BE feature catalog as unscheduled. Not a near-term concern.

**Portrait layout** is the tighter constraint: top bar + board + bottom buttons fills the screen. Adding controls requires either scrollable content or a redesign. The landscape sidebar has more room.

---

## 3. Replay system (BE phase 2)

`gomoku-core::Replay` is fully built (schema versioning, per-move notation + timing + Zobrist hash + optional bot trace, JSON serde, reconstruction tests). But none of it reaches the web:

- **WasmBoard doesn't expose Replay.** No `toReplay()` / `fromReplay()` / `getMoveHistory()` in the WASM bridge. The web only has `toFen()` which snapshots board state — not a move-by-move replay.
- **No replay viewer.** No step-forward/step-back UI. Progress.md defers this explicitly.
- **No replay persistence.** Nothing writes a `Replay` to disk, Firestore, or `localStorage`. Games are entirely ephemeral.
- **Move sequence numbers are a visual band-aid.** `showMoveSequence()` just overlays numbers on stones. Not navigable, not exportable, not shareable.
- **No "Share" button.** Phase 2 needs a Replay object to share, a URL for loading shared replays, and a viewer to play them back.

**Phase 2 FE needs:**

- Expose `Replay` through WASM (or at minimum expose the move list with notation)
- Accumulate replay data during gameplay (move times, hashes) — currently thrown away
- Replay viewer (could be a mode within GameScene rather than a separate scene — the board renderer, stone placement, and container architecture are all reusable)
- URL-based replay loading (exact scheme TBD — the BE design lists URL shape as an open question)
- "Share replay" button on result screen → writes to Firestore, returns URL

---

## 4. Scene architecture (BE phase 3+)

`game.ts` is 1012 lines. The BE design eventually implies multiple views:

- **No menu/lobby scene.** Where does a returning user see their profile, match history, leaderboard?
- **No profile scene.** Needed for match history view, public profiles, stats.
- **No leaderboard scene.** The BE design scopes public leaderboards by bot preset.
- **No puzzle scene.** Daily puzzle is a distinct mode with restricted move input.
- **No correspondence/lobby scene.** Async human-vs-human needs invite flow, match list, turn notifications.

The Phaser scene model already supports this — `main.ts` registers `[BootScene, GameScene]`, so the multi-scene scaffolding exists. The existing container layering (`boardContent` with three children: `boardSurfaceContent` → `boardOverlayContent` → `boardPointerContent`, plus a sibling `settingsContent`) and the slide animation pattern (`animateSettingsSwap`) are reusable for new overlay types.

Multi-scene navigation is **not** a prerequisite for phases 1–2. Phase 1 needs settings panel changes; phase 2 needs a replay viewer (which can live inside GameScene). Multi-scene becomes necessary at phase 3 (match history).

---

## 5. Async operations and error handling

The current codebase has minimal async surface: bot worker calls and WASM init. Both silently fail:

- Bot worker errors log to console only (`console.error("[bot] worker failed:", error)`)
- WASM init failure in `main.ts` produces a blank page — no user-visible feedback

Adding online features (Firestore writes, Cloud Run calls, auth popups, replay sharing) dramatically increases the async failure surface. The FE currently has:

- **No loading spinner** for async operations
- **No error toast** for user-visible failures
- **No retry UI** for transient network errors
- **No confirmation dialogs** for destructive actions ("Share this replay publicly?", "Link Google account?")
- **No reactive state update mechanism.** The BE design uses Firestore snapshot listeners for correspondence play (async turn-based). The FE has zero event-driven state updates — everything is synchronous scene-local state. Adding Firestore listeners would need a reactive update pattern (e.g., a store that scenes subscribe to).
- **No cold-start awareness.** The BE design says Cloud Run "scales to zero when idle." A cold start can take 5–30s. The FE has no "warming up" state, no timeout handling, no fallback for slow API responses. This matters for the stronger bot endpoint and username reservation.

This isn't a single-component gap — it's infrastructure. The fixed-canvas Phaser rendering makes complex overlays (scrollable lists, form validation, error states) significantly harder than in DOM. The project will eventually need to decide between building complex UI in Phaser vs. introducing a DOM overlay layer for non-game UI. That decision doesn't need to happen for phase 1, but it becomes pressing at phase 3+.

---

## 6. Bot system (BE phase 4+)

- **Only `baseline` bot type in worker.** `bot_worker.ts` only handles `kind: "baseline"`. `BotSpec` doesn't have `kind: "random"` yet (planned in progress.md).
- **No depth/time UI.** Hardcoded `depth: 3`. The BE's "stronger bot" needs configurable presets.
- **No bot trace.** `SearchBot` exposes `last_info` (depth, nodes, score) but WasmBot doesn't pass it through. Progress.md notes this as a "future hook."
- **No API-routed bot.** The BE design proposes `POST /bot/move` for deeper search. The `BotRunner` architecture is well-suited for extension — it already implements request sequencing with stale-request cancellation, promise-based async with per-request resolve/reject, worker lifecycle management, and pending-request cleanup. An API-backed bot could reuse this pattern by adding a new `kind` to `BotSpec` that sends HTTP requests instead of worker messages, using the same cancellation semantics.

Auth gating and rate limiting for bot API calls are server-side responsibilities. The FE's job is to handle 429/auth-required responses gracefully.

---

## 7. Wasm API Gaps

Several core capabilities aren't exposed to the web:

| Rust core capability | Exposed to WASM? | Used by web? |
|---|---|---|
| `Replay` (full move history, JSON) | ❌ No | ❌ No |
| `Board.hash()` (Zobrist) | ❌ No | ❌ No |
| `Move.to_notation()` / `from_notation()` | ❌ No | ❌ No |
| `SearchBot.last_info` (trace) | ❌ No | ❌ No |
| `RuleConfig` details | Partial (variant only) | Partial |
| `undoLastMove()` | ✅ Yes | ❌ Not wired |
| `cloneBoard()` | ✅ Yes | ❌ Not wired |
| `legalMoves()` | ✅ Yes | ❌ Not wired |
| `immediateWinningMovesFor()` | ✅ Yes | ✅ Yes |
| `forbiddenMovesForCurrentPlayer()` | ✅ Yes | ✅ Yes |

---

## 8. Data That's Thrown Away

Every game currently discards data that BE features would need:

- **Per-move timing** — `accumulatedMs` tracks cumulative time per player, but individual move durations are lost. The `Replay` struct expects `time_ms` per move.
- **Move history** — `moveOrder` maps cell→move number. JavaScript `Map` preserves insertion order, so iterating yields moves in play order. But there's no explicit `[{row, col, moveNum}]` array, and the board's internal move history isn't exposed through WASM.
- **Game result metadata** — no record of who played, what variant, what bot preset, game duration, etc.
- **Win/loss stats** — tracked in `profiles[].wins` and survive the RESET button path (plain round restart), but explicitly reset to 0 when settings change triggers `applySettingsAndRestart()`. No persistent cumulative record across page reloads.

---

## 9. Quick Wins (no BE dependency)

Some gaps can be closed now without any backend:

- **Undo button** — `undoLastMove()` exists in WASM, just not wired. Useful for casual play.
- **Move history export** — Expose move list through WASM, let users copy FEN or move notation.
- **localStorage persistence** — Settings, win counts, and recent game summaries could persist locally without Firestore.
- **Bot presets** — The progress.md plan is detailed and ready to implement. No BE dependency.
- **Replay viewer (local)** — Even without cloud persistence, a step-through viewer for the current game is achievable once the WASM bridge exposes move history.

### Existing infrastructure ready to extend

These patterns in the current codebase can be reused for online features:

- **`BotRunner`** — general-purpose async request/response pipeline with sequencing, cancellation, and error propagation. An API-backed bot preset follows the same interface.
- **`ToggleGroup`** — generic multi-option selector. Reusable for bot presets, auth provider selection, and filter controls.
- **`animateSettingsSwap()`** — slide-in/slide-out panel transition with mask clipping. Reusable for auth panels, profile views, any overlay.
- **Container layering** — `boardContent` (depth 0) with three children (`boardSurfaceContent` → `boardOverlayContent` → `boardPointerContent`) plus a sibling `settingsContent` (depth 1). Adding a notification or modal layer follows this pattern.
- **Boot→Game scene transition** — the multi-scene pattern already exists. Adding scenes follows the established flow.
