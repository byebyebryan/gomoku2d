# v0.5 Reconciliation Continuation

Status: reopened 2026-10-15 after the post-pause project review. This continues
the `0.5.x` line; `v0.5.4` remains the July release checkpoint. No new release
number or deployment is implied by reopening the work.

The [October review](project_review_2026_10_15.md) supplies the findings and
evidence. The [July closeout](../archive/v0_5_4_reconciliation_plan.md) remains
historical context. Preserve its settled architecture and scope boundaries.

## Active Finding Register

| Finding | Class | Acceptance | Status |
|---|---|---|---|
| Stale cloud flushes can patch a different current profile | fix | Operation-specific results; account/reset/overlap regressions; guarded follow-up history actions | implemented; automated checks passed |
| Storage failures escape local profile actions | fix | Play, settings, and finished replays remain usable in memory; persistence failure is visible | implemented; denied/quota browser cases passed |
| Replay copy discards analyzer confidence | fix | Possible, confirmed, and unresolved outcomes retain their meaning in replay, reports, and Guide | implemented; unit and published-Lab browser checks passed |
| Bot failures leave an unexplained blocked turn | fix | Failed-turn state and explicit retry; cancellation/new-game guards remain correct | implemented; repeated-worker-crash recovery passed on desktop/mobile |
| Board input and selected settings lack accessible semantics | fix | Keyboard board interaction and semantic settings state, with browser verification | implemented; keyboard/browser checks passed; real screen-reader acceptance pending |
| Shared Rust semantics and React/Phaser ownership | retain | Keep the existing boundaries and correctness oracles | confirmed |
| New online product scope, solver changes, and bot-strength research | defer | Fresh product/design checkpoint after reconciliation | deferred |
| Real OAuth and cross-device cloud continuity | acceptance | Manual real-account round trip before a release claiming cloud acceptance | pending |

## October Repair Batch

Cloud loads and promotions now return operation-specific results. Profile
patches require their source snapshot, and follow-up history actions check the
current session. The same protection covers the separate background-history
patch path found during implementation.

The profile and cloud-history cache share a best-effort storage boundary that
supports synchronous and asynchronous storage. After a failure, the store stays
in memory for the session; it does not overwrite data it could not read. The
local profile exposes a visible save-status notice outside its persisted state.

Replay uses E for confirmed and P for possible escapes; status/failure text,
published Lab decisions, and the Guide preserve that distinction. Bot failures
now expose an explicit retry while retaining the current position. A DOM grid
mirrors the canvas for keyboard navigation and cell descriptions, with one Tab
stop, visible focus, and placement through the existing game-store callback.

Validation on 2026-10-15:

- Web typecheck, including scripts and Playwright tools: passed.
- Web unit/contract/scenario tests: 326 passed across 35 files.
- Firestore emulator rules: 23 passed.
- Production build and all browser flows: 30 passed, including seven new
  reconciliation scenarios for keyboard completion, semantic settings, denied
  and quota storage failures, desktop/mobile worker recovery, and report confidence.
- Rendered keyboard-focus, storage-fallback, and desktop/mobile retry states
  inspected from the browser test screenshots.
- Curated report data, Rust behavior, and persisted schemas remain unchanged.

No release tag or deployment was made. The phase remains open: real-account
continuity/reset checks and actual screen-reader acceptance have not been
substituted with automated evidence. Representative slow-device performance
remains a separate acceptance question from the native tournament measurements.

## Completion Bar

- Resolve the demonstrated failure paths with focused regression coverage.
- Validate keyboard, desktop, mobile, finished-game, replay, and recovery flows.
- Run checks appropriate to the changed layers and record their results.
- Keep persisted schemas and curated report data stable unless a demonstrated
  defect requires a separately documented change.
- Keep manual cloud acceptance distinct from emulator and unit-test evidence.
- Reconcile active docs and findings before deciding whether to release.

This continuation does not start online implementation, a broad redesign, or a
new bot/analyzer research phase. `v0.6` planning follows its completion.
