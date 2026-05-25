# Gomoku2D Bot Reports

Curated bot-lab data for the ranking and search tabs of the published
`/lab-report/` page. The JSON remains available under `/bot-report/report.json`
for compatibility.

Rust owns the tournament run and data export. The web app owns report rendering,
so this folder should contain `report.json` only. Scratch and full diagnostic
reports belong in ignored `outputs/`.

Recommended flow, from `gomoku-bot-lab/`:

1. Commit bot/report tooling changes first.
2. Confirm the worktree is clean before running the curated tournament.
3. Generate the compact published report directly into this folder.
4. Commit `report.json` as the artifact update.

```sh
git status --short
mkdir -p reports
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d3+pattern-eval,search-d5+tactical-cap-16+pattern-eval,search-d7+tactical-cap-8+pattern-eval,search-d3+pattern-eval+corridor-proof-c16-d8-w4,search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 2000 \
  --search-budget-mode pooled \
  --search-cpu-reserve-ms 8000 \
  --search-cpu-max-move-ms 4000 \
  --max-moves 120 \
  --seed 63 \
  --threads 22 \
  --published-report-json reports/report.json
jq '.provenance | {git_commit, git_dirty}' reports/report.json
```

For local debug runs, add `--report-json outputs/full-tournament-report.json`
to keep the rich per-match telemetry outside the published artifact.

The curated published report uses pooled CPU budgeting: each move starts with a
`2000 ms` base budget, cheaper moves bank unused time into an `8000 ms` reserve
pool, and any single move is capped at `4000 ms`. This is closer to product-like
hard-bot behavior than the older strict-per-move report while keeping the run
bounded.

`report.json` can also be used as the anchor-rating source for focused gauntlet
runs with `--anchor-report reports/report.json`. Compact published reports omit
per-match side telemetry, so gauntlet anchors keep standings and pairwise
context but not reference pair search-cost drilldowns.

The curated replay-analysis report in `../analysis-reports/` is generated from
this `report.json` and should explain the Easy/Normal/Hard preset triangle used
by the current product bots.
