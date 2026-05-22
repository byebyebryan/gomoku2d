# Docs

Gomoku2D keeps docs in the repo so product decisions, implementation contracts,
working notes, and release history move with the code. The docs are split by
audience:

| Bucket | Audience | Purpose |
|---|---|---|
| [`public/`](public/) | Players, visitors, release copy | Short explanations of the project, rules, analysis, and bot lab |
| [`reference/product/`](reference/product/) | Humans and agents | Product thesis, product model, and roadmap |
| [`reference/app/`](reference/app/) | Humans and agents | Web app architecture, UI contracts, and visual language |
| [`reference/backend/`](reference/backend/) | Humans and agents | Backend model and persisted data contracts |
| [`reference/lab/`](reference/lab/) | Humans and agents | Bot, tactical, replay-analysis, corridor, lethal-threat, and Renju models |
| [`reference/corpora/`](reference/corpora/) | Implementers | Validation corpora and scenario catalogs |
| [`reference/ops/`](reference/ops/) | Maintainers | Release, infra, cost, tournament, and test runbooks |
| [`working/`](working/) | Agents and maintainers | Active notes, benchmark logs, screenshot reviews, and current plans |
| [`archive/`](archive/) | Maintainers | Superseded historical docs and rejected paths |

## Public Docs

- [`About`](public/about.md)
- [`Rules And Renju`](public/rules.md)
- [`Replay Analysis`](public/analysis.md)
- [`Bot Lab`](public/bot-lab.md)

## Canonical References

- [`Project Thesis`](reference/product/project.md)
- [`Product`](reference/product/product.md)
- [`Roadmap`](reference/product/roadmap.md)
- [`Architecture`](reference/app/architecture.md)
- [`App Design`](reference/app/app_design.md)
- [`Search Bot`](reference/lab/search_bot.md)
- [`Corridor Search`](reference/lab/corridor_search.md)
- [`Game Analysis`](reference/lab/game_analysis.md)
- [`Renju Rules`](reference/lab/renju_rules.md)
- [`Release`](reference/ops/release.md)

## Maintenance Rule

Public docs explain the project. Reference docs define current behavior and
contracts. Working docs can be verbose and preserve raw context. Archive docs
should not be used as current direction unless a canonical doc explicitly points
there for historical evidence.
