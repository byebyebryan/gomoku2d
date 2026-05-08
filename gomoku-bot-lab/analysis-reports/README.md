# Gomoku2D Analysis Reports

Curated replay-analysis report artifacts for publishing.

The published analysis report is the companion to the published bot report:
sample the head-to-head games between the current top two standings in
`reports/latest.json`, then render the forced-corridor explanation report here.
Do not publish arbitrary scratch analysis runs from `outputs/`.

Recommended flow, from `gomoku-bot-lab/`:

1. Commit analyzer/report tooling changes first.
2. Confirm `git status --short` is clean enough for the source bot report you
   are analyzing.
3. Confirm `reports/latest.json` is the current published bot report.
4. Generate `analysis-reports/latest.json` and `analysis-reports/index.html`.
5. Commit the analysis artifacts as a follow-up artifact commit.

```sh
git status --short
mkdir -p analysis-reports
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --sample-size 64 \
  --include-proof-details \
  --report-json analysis-reports/latest.json \
  --report-html analysis-reports/index.html \
  --max-depth 4
```

`analyze-report-replays` intentionally omits explicit entrants here. By
default it selects standing #1 versus the highest different standing from the
published bot report, so `/analysis-report/` always explains the current
leaderboard's top matchup.

Keep the raw JSON next to the generated HTML so the report can be inspected or
re-rendered without rerunning analysis.
