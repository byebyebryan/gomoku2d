# Docs

Gomoku2D keeps product decisions, implementation contracts, runbooks, and
release history beside the code. Use this page to enter the docs by area rather
than browsing the tree. Active docs describe current intent or behavior;
historical context belongs in [`archive/`](archive/).

## Current Status

The October `0.5.x` reconciliation, systematic code review, and selected mobile
Rules/Guide refinement are complete on `main`. The public release remains
`v0.5.4`; no new version or deployment is part of the review. Start with the
[October closeout](archive/reconciliation_2026_10/README.md) for decisions,
verification, and explicit acceptance deferrals, or the
[roadmap](reference/product/roadmap.md) for what follows.

## Choose A Path

| Area | Start with | Continue with |
|---|---|---|
| Product direction | [`Product Strategy`](reference/product/product_strategy.md) | [`Roadmap`](reference/product/roadmap.md) |
| Browser app | [`Web Code Overview`](reference/app/code_overview.md) | [`Architecture`](reference/app/architecture.md), [`App Design`](reference/app/app_design.md), [`UI Design`](reference/app/ui_design.md) |
| Board visuals | [`Game Visual`](reference/app/game_visual.md) | [`Asset Sources`](../gomoku-web/assets/README.md) |
| Rust rules, bots, and analyzer | [`Bot Lab Code Overview`](reference/lab/code_overview.md) | [`Search Bot`](reference/lab/search_bot.md), [`Game Analysis`](reference/lab/game_analysis.md) |
| Tactical model | [`Tactical Shapes`](reference/lab/tactical_shapes.md) | [`Lethal Threats`](reference/lab/lethal_threats.md), [`Corridor Search`](reference/lab/corridor_search.md) |
| Renju legality | [`Renju Rules`](reference/lab/renju_rules.md) | [`Renju Corpus`](reference/corpora/renju_corpus.md) |
| Testing and releases | [`Testing`](reference/ops/testing.md) | [`Tournament Eval`](reference/ops/tournament.md), [`Release`](reference/ops/release.md) |

## Public Surfaces

Player-facing explanations live in the web app rather than a second
documentation tree:

- `/rules/` — basic Gomoku and Renju rules
- `/guide/` — how to think about threats, combos, and forced sequences
- `/lab/` — bot tournament and replay-analysis reports
- `/visuals/` — visual guide and asset language
- `/privacy/`, `/terms/`, and Source — project and legal surfaces

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
[`archive/process_story/`](archive/process_story/). It is private source
material for a possible future devlog, not an active release dependency or
canonical product documentation.

## Maintenance Rule

Reference docs are current contracts. Working notes are temporary and need an
owner. Archive docs preserve context but are not authoritative. Dated lab logs,
old experiment tables, generated board dumps, and release diaries do not belong
in canonical references.
