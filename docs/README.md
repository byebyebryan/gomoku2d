# Docs

Gomoku2D keeps docs in the repo so product decisions, implementation contracts,
runbooks, and release history move with the code. Active docs should describe
current intent or current behavior. Historical notes belong in
[`archive/`](archive/).

## Start Here

| Question | Read |
|---|---|
| What is the project trying to be? | [`Product Strategy`](reference/product/product_strategy.md) |
| What is being built next? | [`Roadmap`](reference/product/roadmap.md) |
| How is the web app structured? | [`Architecture`](reference/app/architecture.md), [`Web Code Overview`](reference/app/code_overview.md) |
| How does the board/app UX work? | [`App Design`](reference/app/app_design.md), [`UI Design`](reference/app/ui_design.md), [`Game Visual`](reference/app/game_visual.md) |
| How do the Rust rules, bot, and analyzer fit together? | [`Bot Lab Code Overview`](reference/lab/code_overview.md) |
| How does search/replay analysis work? | [`Search Bot`](reference/lab/search_bot.md), [`Tactical Shapes`](reference/lab/tactical_shapes.md), [`Lethal Threats`](reference/lab/lethal_threats.md), [`Corridor Search`](reference/lab/corridor_search.md), [`Game Analysis`](reference/lab/game_analysis.md) |
| How does Renju legality work? | [`Renju Rules`](reference/lab/renju_rules.md), [`Renju Corpus`](reference/corpora/renju_corpus.md) |
| How do I test, release, or refresh reports? | [`Testing`](reference/ops/testing.md), [`Tournament Eval`](reference/ops/tournament.md), [`Release`](reference/ops/release.md) |

## Public Surfaces

Player-facing explanations live in the web app, not under `docs/public`:

- `/rules/` — basic Gomoku and Renju rules
- `/guide/` — how to think about threats, combos, and forced sequences
- `/lab/` — bot tournament and replay-analysis reports
- `/visuals/` — visual guide and asset language
- `/privacy/`, `/terms/`, and Source — project metadata/legal surfaces

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

- [`v0.5.4 Reconciliation Closeout`](working/v0_5_4_reconciliation_plan.md)
  owns the active repository, test, docs, tooling, and product-polish pass
  before `v0.6` planning.

Parked process-story extraction and curation lives under
[`archive/process_story/`](archive/process_story/). It is private source
material for a possible future devlog, not an active release dependency or
canonical product documentation.

## Maintenance Rule

Active reference docs should be concise and current. Do not keep dated lab logs,
old experiment tables, generated board dumps, or release diaries in canonical
docs. Move that material to archive, generated outputs, or working notes with a
clear owner.
