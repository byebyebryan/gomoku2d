# Docs

Gomoku2D keeps product decisions, implementation contracts, runbooks, and
release history beside the code. Use this page to enter the docs by area rather
than browsing the tree. Active docs describe current intent or behavior;
historical context belongs in [`archive/`](archive/).

## Current Status

`v0.6.1` is published with shareable replays, explicit saved copies, and opt-in
public profiles, statistics and game catalogs. The
[release record](archive/v0_6_1_release.md) owns deployment and automated
production evidence, the pending real signed-in sharing check, and retained
manual acceptance deferrals. [GitHub Releases](https://github.com/byebyebryan/gomoku2d/releases/latest)
identifies the published build; the [roadmap](reference/product/roadmap.md)
tracks `0.6.2` UI/presentation, `0.6.3` replay collection tools and `0.6.4`
curated learning. The [0.6.2 candidate handoff](working/v0_6_2_candidate.md)
owns the completed local implementation, validation, preview and open manual
checks. It includes public-profile/mobile refinements, Guide sharing help,
two account-free examples, inline enable confirmation and refreshed media.
It has not been pushed, tagged or published. The
[follow-up plan](working/v0_6_followup_plan.md) retains the research and future
scope; the [implementation archive](archive/v0_6_implementation.md) indexes
completed planning and evidence without moving their existing links.
The earlier [October closeout](archive/reconciliation_2026_10/README.md)
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

## Choose A Path

| Area | Start with | Continue with |
|---|---|---|
| Product direction | [`Product Strategy`](reference/product/product_strategy.md) | [`Roadmap`](reference/product/roadmap.md) |
| Browser app | [`Web Code Overview`](reference/app/code_overview.md) | [`Architecture`](reference/app/architecture.md), [`App Design`](reference/app/app_design.md), [`UI Design`](reference/app/ui_design.md) |
| Cloud continuity | [`Backend`](reference/backend/backend.md) | [`Data Model`](reference/backend/data_model.md), [`Infra`](reference/ops/backend_infra.md), [`Cost`](reference/ops/backend_cost.md) |
| Board visuals | [`Game Visual`](reference/app/game_visual.md) | [`Asset Sources`](../gomoku-web/assets/README.md) |
| Rust rules, bots, and analyzer | [`Bot Lab Code Overview`](reference/lab/code_overview.md) | [`Search Bot`](reference/lab/search_bot.md), [`Game Analysis`](reference/lab/game_analysis.md) |
| Tactical model | [`Tactical Shapes`](reference/lab/tactical_shapes.md) | [`Lethal Threats`](reference/lab/lethal_threats.md), [`Corridor Search`](reference/lab/corridor_search.md) |
| Renju legality | [`Renju Rules`](reference/lab/renju_rules.md) | [`Renju Corpus`](reference/corpora/renju_corpus.md) |
| Testing and releases | [`Testing`](reference/ops/testing.md) | [`Tournament Eval`](reference/ops/tournament.md), [`Release`](reference/ops/release.md) |

## Public Surfaces

Player-facing explanations live in the web app rather than a second
documentation tree:

- [Rules](https://gomoku2d.byebyebryan.com/rules/) — basic Gomoku and Renju rules
- [Guide](https://gomoku2d.byebyebryan.com/guide/) — threats, combos and forced sequences;
  `0.6.2` adds direct Replays & sharing and Examples sections (candidate only)
- [Lab](https://gomoku2d.byebyebryan.com/lab/) — bot tournament and replay-analysis reports
- [Visuals](https://gomoku2d.byebyebryan.com/visuals/) — visual guide and asset language
- [Privacy](https://gomoku2d.byebyebryan.com/privacy/) and
  [Terms](https://gomoku2d.byebyebryan.com/terms/) — public policy pages

The root [`README`](../README.md) is the public repo landing page.

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
