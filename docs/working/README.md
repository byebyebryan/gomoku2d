# Working Notes

Active design drafts and maintained procedures. Current contracts live in the
[reference docs](../README.md#choose-a-path). The
[archive index](../archive/README.md) lists completed work.

## Follow-up Research

| Note | Owner and next review |
|---|---|
| [0.7 refinement plan](v0_7_refinement_plan.md) | Local candidate source, integration receipts, open release gates and later refinement work. |
| [0.7 UI evidence](v0_7_ui_evidence/README.md) | Local v0.6.5-to-candidate comparison and capture provenance for the candidate Dev route. |
| [v0.6 follow-up research](v0_6_followup_plan.md) | Planning snapshot with research for later curated learning. The roadmap tracks current scope. |

### Repository review follow-ups

- **Raw replay analysis.** A legal one-move replay declared as a Black win
  can yield a decisive-attack claim. Require an on-board win for tactical
  winner claims in the native analyzer and Wasm adapter. Keep valid terminal
  wins and capped draws readable. Fold the CLI's `--max-moves 0` behavior into
  the arena limit tests. [Analysis contract](../reference/lab/game_analysis.md)
- **Replay-library cost.** The repo-wide host-side Wasm probe took a median
  of 223 ms to hydrate 64 legal 225-ply games. Metadata edits revalidate the
  unchanged payloads. Profile opening the library and editing one title in
  browser with full and multi-owner collections. Reuse validation for
  identical bytes while retaining legality, fingerprint, and revision checks.
  [Replay collection review](v0_6_4_replay_collection_review.md)
- **Visuals URL compatibility.** The candidate inventory retains the legacy
  social-preview source and 12 tuning-font variants. Revisit those URLs if a
  future release retires downloads.
  [Publication inventory](../../gomoku-web/scripts/visual_design_asset_inventory.mjs)
- **Published screen reference.** The candidate dates the 0.6.2 gallery at its
  original URL and includes the matched play and replay comparison. Promote a
  new screen reference from the verified published build.
  [Developer area contract](../reference/app/developer_area.md)
- **Player journeys.** Make compact verdict qualifications reachable and
  recover on Back after a failed route load. Before adding Guide lessons,
  compare lazy initialization or static diagrams with the nine off-screen WebGL contexts
  observed on a compact Examples visit. Preserve the sprite output and
  accessible captions. Physical device and real screen-reader checks remain
  open.

## Published v0.6 Handoff

The [0.6 implementation index](../archive/v0_6_implementation.md) links the
accepted designs, review evidence, and published release records. Use the
[0.6.5 release record](../archive/v0_6_5_release.md) for production results and
open acceptance. The [0.6.5 capture record](v0_6_5_ui_evidence/README.md) retains
the comparison images and emulator scope.

## Historical Dev Site Evidence

The [archive index](../archive/README.md#historical-dev-site-evidence) points to
the frozen screen reference, capture pilot, and UI history. The build verifies
the capture and manifest paths and digests recorded in the Dev catalogs.

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
[roadmap](../reference/product/roadmap.md) tracks the 0.7 sequence.
