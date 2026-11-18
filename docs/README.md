# Docs

Gomoku2D keeps product decisions, implementation contracts, runbooks, and
release history beside the code. Use this page to enter the docs by area rather
than browsing the tree. Active docs describe current intent or behavior;
historical context belongs in [`archive/`](archive/).

## Current Status

`v0.6.4` completes the planned 0.6 line with replay thumbnails, private study
details and editable public listings. The release record tracks exact-source
CI, deployment, Chrome mobile layout and the retained account/device/human
acceptance limits. [Release record](archive/v0_6_4_release.md)

Curated learning remains unnumbered. Multiplayer remains deferred.
[Roadmap](reference/product/roadmap.md) ·
[Implementation archive](archive/v0_6_implementation.md)
The [October closeout](archive/reconciliation_2026_10/README.md)
retains the `0.5.x` reconciliation decisions and evidence.

The v0.6 sharing work shipped in `v0.6.1`. Its
[design draft](working/v0_6_profiles_and_replays.md) separates accepted scope
from the original proposals; final formats and storage contracts live in
[portable replay](reference/app/portable_replay.md) and
[sharing data](reference/backend/sharing_data.md). The
[implementation roadmap](working/v0_6_implementation_plan.md) sequences the guest
sharing demo, private library, public publishing, and release checks.
[Execution and evidence](working/v0_6_execution.md) retain the local acceptance
history. Multiplayer research is retained as deferred work.

The shared developer area first shipped in 0.6.3. Its
[contract](reference/app/developer_area.md) owns route, source and review-record
semantics; the [execution record](working/v0_6_3_execution.md) owns local
validation. The [0.6.3 release record](archive/v0_6_3_release.md) holds its
production results.
The [UI history implementation](working/v0_6_3_history_implementation.md) extends
that area with a milestone overview and individual screen trails, backed by
qualified recaptures from tagged sources.

Markdown is detailed agent/maintainer context. The developer area prepares
selected design references, inspectors, review comparisons and the screenshot checklist
for human reading; it does not automatically publish this tree.

## Choose A Path

| Area | Start with | Continue with |
|---|---|---|
| Product direction | [`Product Strategy`](reference/product/product_strategy.md) | [`Roadmap`](reference/product/roadmap.md) |
| Browser app | [`Web Code Overview`](reference/app/code_overview.md) | [`Architecture`](reference/app/architecture.md), [`App Design`](reference/app/app_design.md), [`UI Design`](reference/app/ui_design.md) |
| Cloud continuity | [`Backend`](reference/backend/backend.md) | [`Data Model`](reference/backend/data_model.md), [`Infra`](reference/ops/backend_infra.md), [`Cost`](reference/ops/backend_cost.md) |
| Developer presentation and review workflow | [`Developer area`](reference/app/developer_area.md) | [`Capture procedure`](working/ui_screenshot_review.md), [`Execution`](working/v0_6_3_execution.md) |
| Board visuals | [`Game Visual`](reference/app/game_visual.md) | [`Asset Sources`](../gomoku-web/assets/README.md) |
| Rust rules, bots, and analyzer | [`Bot Lab Code Overview`](reference/lab/code_overview.md) | [`Search Bot`](reference/lab/search_bot.md), [`Game Analysis`](reference/lab/game_analysis.md) |
| Tactical model | [`Tactical Shapes`](reference/lab/tactical_shapes.md) | [`Lethal Threats`](reference/lab/lethal_threats.md), [`Corridor Search`](reference/lab/corridor_search.md) |
| Renju legality | [`Renju Rules`](reference/lab/renju_rules.md) | [`Renju Corpus`](reference/corpora/renju_corpus.md) |
| Testing and releases | [`Testing`](reference/ops/testing.md) | [`Tournament Eval`](reference/ops/tournament.md), [`Release`](reference/ops/release.md) |

## Public Surfaces

Player-facing explanations live in the web app rather than a second
documentation tree:

- [Rules](https://gomoku2d.byebyebryan.com/rules/) — basic Gomoku and Renju rules
- [Guide](https://gomoku2d.byebyebryan.com/guide/) — tactics, replay/sharing help
  and account-free example replays
- [Dev site](https://gomoku2d.byebyebryan.com/dev/) — overview of the work behind the game
- [Visual design](https://gomoku2d.byebyebryan.com/visuals/) — visual system and asset inspection
- [UI design](https://gomoku2d.byebyebryan.com/dev/reviews/core-screens-062/) — current screens, UI history and design reviews
- [Bot lab](https://gomoku2d.byebyebryan.com/lab/) — bot tournament and replay-analysis reports
- [Privacy](https://gomoku2d.byebyebryan.com/privacy/) and
  [Terms](https://gomoku2d.byebyebryan.com/terms/) — public policy pages

The 0.6.4 design review records the replay editors and private conflict flow.
[Replay collections](https://gomoku2d.byebyebryan.com/dev/reviews/replay-collections-064/)

## Buckets

| Bucket | Purpose |
|---|---|
| [`reference/product/`](reference/product/) | Product thesis, current direction, and roadmap |
| [`reference/app/`](reference/app/) | Web app architecture, screen contracts, and visual language |
| [`reference/backend/`](reference/backend/) | Backend contracts, data model, and future backend design |
| [`reference/lab/`](reference/lab/) | Current bot, tactical, replay-analysis, corridor, lethal-threat, and Renju models |
| [`reference/corpora/`](reference/corpora/) | Fixture indexes and validation-corpus entrypoints |
| [`reference/ops/`](reference/ops/) | Release, infra, cost, tournament, and test runbooks |
| [`working/`](working/) | Active notes only; should be pruned or archived regularly |
| [`archive/`](archive/) | Historical context, retired plans, rejected paths, and old release notes |

## Internal Working Notes

The [working-note index](working/README.md) lists the maintained dependency,
performance, and screenshot-review notes. Completed October plans, code-review
dispositions, measurements, and captures are grouped in the
[reconciliation archive](archive/reconciliation_2026_10/README.md). They record
what was reviewed and decided; they are not a new implementation backlog.
The completed July `v0.5.4` material remains in [`archive/`](archive/).

Parked process-story extraction and curation lives under
[`archive/process_story/`](archive/process_story/). It is draft material
for a possible future devlog, not an active release dependency or
canonical product documentation.

## Maintenance Rule

Reference docs are current contracts. Working notes are temporary and need an
owner. Archive docs preserve context but are not authoritative. Dated lab logs,
old experiment tables, generated board dumps, and release diaries do not belong
in canonical references.
