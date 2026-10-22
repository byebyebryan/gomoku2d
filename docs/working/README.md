# Working Notes

Active design drafts and maintained procedures. Completed reviews belong in
the [archive](../archive/README.md); current product and implementation contracts
belong in the [reference docs](../README.md#choose-a-path).

## Active Design

| Note | Owner and next review |
|---|---|
| [v0.6 public profiles and shareable replays](v0_6_profiles_and_replays.md) | Bryan; settle profile visibility, library persistence, publication scope, and the portable replay contract before implementation |
| [v0.6 implementation roadmap](v0_6_implementation_plan.md) | Bryan; start with portable replay contracts/fixtures, then track milestone acceptance and remaining decisions |
| [v0.6 execution record](v0_6_execution.md) | Bryan and the active Luna goal loop; update milestone status, decisions, commits, and acceptance evidence as work lands |

## Completed Follow-up

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
which records the retained UI and deferred manual acceptance. The v0.6 draft
tracks the separate next feature direction.
