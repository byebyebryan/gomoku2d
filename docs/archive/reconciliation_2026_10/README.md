# October 2026 Reconciliation Closeout

Owner: Bryan. The reopened `0.5.x` review, repair, structural cleanup, and selected
UI refinement are complete. This directory preserves the review as historical
evidence. Current direction lives in the [roadmap](../../reference/product/roadmap.md);
current behavior and ownership live in the [reference docs](../../README.md).

## Decisions

- Retain the local-first play → replay analysis → practice loop, shared Rust
  semantics, and React/Phaser ownership split.
- Keep the existing UI and copy. The visual comparison was reviewed; only
  mobile Rules/Guide layouts were selected. Broader mock proposals are not an
  accepted roadmap or outstanding implementation batch.
- Close reproduced correctness, lifecycle, recovery, and payload-validation
  failures; retain bounded analyzer/Renju semantics and their documented limits.
- Complete targeted code cleanup. Further search-context, telemetry, and
  Profile-selector refactors are conditional maintenance options, not open
  requirements.
- Maintain dependencies directly on `main` with scoped review and validation.
  Choose the next player benefit before committing to online play, puzzles,
  sharing, or further engine research.

## Publication and Verification

| Surface | Recorded state at closeout |
|---|---|
| Runtime baseline | [`a067040`](https://github.com/byebyebryan/gomoku2d/commit/a0670401c8f9bff4af0bac4ccab5cf547a7b6256), including the Rules/Guide mobile fix |
| Automated checks | [Successful CI on that exact commit](https://github.com/byebyebryan/gomoku2d/actions/runs/38855047048): native checks, Wasm, web types/tests, Firestore emulator, build, and full Chromium/Firefox/WebKit journeys |
| Responsive follow-up | 18 Rules/Guide checks across nine viewport sizes from 320px to 1280px; [details and captures](v0_5_review_evidence.md#rules-and-guide-mobile-layout-follow-up) |
| Latest public release | [`v0.5.4`](https://github.com/byebyebryan/gomoku2d/releases/tag/v0.5.4); [latest Pages deployment](https://github.com/byebyebryan/gomoku2d/actions/runs/30063602884) at `8e47284` |
| Release media / Lab data | July release captures and June report measurements retain their original provenance; neither claims to measure the final October code |
| Release decision | No new version, tag, deployment, rules rollout, or production-account write was made by these review passes |

This records the runtime baseline before documentation reconciliation; it is not
a moving CI badge for future commits. Individual documents retain the source
revision and intermediate results they actually reviewed.

## Deferred Acceptance

Bryan explicitly deferred manual checks. Real Google OAuth and two-device
settings/history/reset, physical lower-powered/mobile performance and touch
comfort, actual browser zoom, screen-reader use, and novice/human visual
feedback remain unverified. Automation covered the available journeys,
semantics, accessibility audits, responsive layouts, and desktop profiling;
it does not turn those deferrals into passes.

The [acceptance record](v0_5_review_evidence.md#unattended-acceptance-and-manual-check-deferral)
also retains inconclusive gradient-contrast and illustrative-icon checks.
These deferrals do not keep the engineering review or next design checkpoint
open. Carry them into any later release decision.

## Review Record

| Document | Role |
|---|---|
| [Initial project review](project_review_2026_10_15.md) | Findings against the pre-repair baseline; not a list of current defects |
| [Deep review plan and dispositions](v0_5_reconciliation.md) | Scope, workstreams, finding register, and readiness decision |
| [Evidence ledger](v0_5_review_evidence.md) | Chronological traces, reproductions, measurements, acceptance, and mobile screenshots |
| [Systematic code review](code_review_2026_10_15.md) | Completed cleanup, retained boundaries, and conditional future refactors |

Earlier entries in the ledger describe intermediate states. The readiness
decision, manual-check deferral, and final follow-ups supersede earlier requests
to keep the review open. No historical measurement was relabeled as a new run.
