# Gomoku2D Bot Reports

Curated bot-lab report artifacts for publishing.

Scratch tournament output belongs in ignored `outputs/`. Put only reports that
are worth publishing here. The web build copies this folder to
`/bot-report/` on GitHub Pages.

Recommended flow, from `gomoku-bot-lab/`:

1. Commit the bot/report tooling change first.
2. Confirm the worktree is clean before running the curated tournament.
3. Generate the report directly into this folder.
4. Commit `latest.json` and `index.html` as a follow-up artifact commit.

The report captures git provenance. If the worktree is dirty when the
tournament runs, the published report will show a `_dirty` revision and a
development-run warning. That is useful for scratch reports, but release-quality
curated reports should come from a clean committed toolchain.

```sh
git status --short
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament --bots fast,balanced,deep --games-per-pair 64 --opening-plies 4 --search-cpu-time-ms 1000 --max-moves 120 --seed 42 --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
```

Keep the raw JSON next to the generated HTML so the report can be inspected or
re-rendered without rerunning the tournament.
