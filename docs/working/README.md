# Working Notes

Active design drafts and maintained procedures. Completed reviews belong in
the [archive](../archive/README.md); current product and implementation contracts
belong in the [reference docs](../README.md#choose-a-path).

## Active Design

| Note | Owner and next review |
|---|---|
| [0.6.3 developer-area brief](v0_6_3_developer_site_plan.md) | Bryan; visual study accepted; [implementation/execution](v0_6_3_execution.md) owns local integration, evidence and remaining release/manual checks |
| [Retrospective UI capture pilot](v0_6_3_retrospective_capture.md) | Bryan; compare v0.2.4 and v0.6.2 with a reusable capture contract; generic portrait profile remains provisional pending phone measurements |
| [UI history implementation](v0_6_3_history_implementation.md) | 24 milestones / 15 screen histories / 274 selected captures; first human pass broadly positive, latest-first ordering implemented; generic portrait remains provisional |
| [Design-review backfill](v0_6_3_design_reviews.md) | Eight prepared cases; source/media and shared-export/Snap delivery verified; follow-up findings are tracked in the overall review below |
| [0.6.3 overall review and fixes](v0_6_3_review_fixes.md) | All five findings from the [original review](v0_6_3_overall_review.md) resolved at `c1c3af6`; 556 units, 44 tool tests, 105 affected browser tests and six cloud journeys pass; preview verified from Snap; human/release gates remain separate |
| [0.6.3 release-readiness fixes](v0_6_3_release_fixes.md) | All findings from the [release-wide review](v0_6_3_release_review.md) resolved at `e34129f`; 558 unit, 44 tool, 297 browser, 49 rules and 120 cloud/repetition checks pass; ready for separate release preparation |
| [0.6.3 final review](v0_6_3_final_review.md) | Candidate `b4fdeb4`; no new blockers; clean shallow-checkout Node-24 CI command passes 44/44; source/evidence and audits verified; packaging and publication are next |
| [v0.6 follow-up research](v0_6_followup_plan.md) | Bryan; revised sequence and retained research; replay metadata compatibility belongs to 0.6.4, curated learning later |

Completed 0.6.2 design, implementation, review, mobile and completion-plan
checkpoints are indexed in the [implementation archive](../archive/v0_6_implementation.md).
Their paths and original evidence remain intact; they are snapshots, not
competing active plans.

## Published v0.6 Handoff

The [0.6.2 release record](../archive/v0_6_2_release.md) owns current publication,
automated production results and open acceptance. The executed
[release plan](v0_6_2_release_plan.md) and historical
[candidate checkpoint](v0_6_2_candidate.md) retain preparation details.

The earlier [0.6.1 release record](../archive/v0_6_1_release.md) retains its
publication evidence and original real-account handoff. Its [release plan](v0_6_1_release_plan.md) and
[acceptance ledger](v0_6_1_evidence/README.md) retain the executed sequence and
manual handoff. The [original design](v0_6_profiles_and_replays.md),
[implementation plan](v0_6_implementation_plan.md),
[execution record](v0_6_execution.md) and
[functional profile pass](v0_6_1_public_profile.md) retain completed design and
implementation context. They are not the active plan for the remaining line;
see the [implementation archive](../archive/v0_6_implementation.md) for the
completed-note index. Their existing paths and release evidence are preserved.

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
which records the retained UI and deferred manual acceptance. The
[roadmap](../reference/product/roadmap.md) and follow-up plan track the remaining
`0.6.3`/`0.6.4` direction and later curated learning.
