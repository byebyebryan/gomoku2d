# Working Notes

Active design drafts and maintained procedures. Completed reviews belong in
the [archive](../archive/README.md); current product and implementation contracts
belong in the [reference docs](../README.md#choose-a-path).

## 0.6.5 Candidate

The reviewed candidate adds final-board previews to My games and starts a
replay publish draft directly from an eligible row. The [capture receipt](v0_6_5_ui_evidence/README.md)
records the application source, comparison images, and emulator scope. The
[roadmap](../reference/product/roadmap.md#current-status) records release status.
Use the [release runbook](../reference/ops/release.md#release-prep) for packaging,
exact-source CI, and deployment checks.

## Follow-up Research

| Note | Owner and next review |
|---|---|
| [v0.6 follow-up research](v0_6_followup_plan.md) | Planning snapshot with research for later curated learning. The roadmap tracks current scope. |

## Published v0.6 Handoff

The [0.6 implementation index](../archive/v0_6_implementation.md) links the
accepted designs, review evidence, and published release records. Use the
[0.6.4 release record](../archive/v0_6_4_release.md) for the latest production
results and open acceptance.

## Completed Follow-up

The [Gomoku2D portfolio handoff](devlog_gomoku2d_handoff.md) was published on
October 28 at Devlog commit `f5a2645`: refreshed card/reel, Dev site link and
pinned provenance. The receipt distinguishes owner validation, independent
Gomoku checks and the remaining human/device acceptance.

The [v0.5 cloud and Wasm evidence](../archive/reconciliation_2026_10/v0_5_unattended_evidence.md)
records the fixed profiling matrix, cloud repair, passing three-engine CI,
and the completed closeout decision. Its [execution plan](../archive/reconciliation_2026_10/v0_5_unattended_followup.md)
is retained with the October review. The earlier reconciliation remains a
completed review.

## Maintained Notes

| Note | Use and next review |
|---|---|
| [Dependency maintenance](dependency_maintenance.md) | Overrides, development-tool audit exceptions, and deferred major updates; revisit during dependency maintenance or release preparation |
| [Performance tuning](performance_tuning.md) | Native benchmarking process and conditional optimization work; revisit when changing search or starting a measured performance investigation |
| [UI screenshot review](ui_screenshot_review.md) | Capture and visual QA procedure; use when changing a visible surface or refreshing release media |

Bryan owns these design and maintenance decisions. The maintenance notes do not
reopen the completed [October reconciliation](../archive/reconciliation_2026_10/README.md),
which records the retained UI and deferred manual acceptance. The
[roadmap](../reference/product/roadmap.md) tracks later curated learning.
