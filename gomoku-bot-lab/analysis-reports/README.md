# Gomoku2D Analysis Reports

Curated replay-analysis data for the analysis tab of the published
`/lab-report/` page. The JSON remains available under
`/analysis-report/report.json` for compatibility.

The published analysis report is the companion to the published bot report:
sample the head-to-head games between the current top two standings in
`reports/report.json`, then export the forced-corridor explanation data here.
The web app renders the analysis tab from this JSON.

Recommended flow, from `gomoku-bot-lab/`:

1. Commit analyzer/report tooling changes first.
2. Confirm `reports/report.json` is the current published bot report.
3. Generate `analysis-reports/report.json`.
4. Commit the analysis artifact as a follow-up artifact update.

```sh
git status --short
mkdir -p analysis-reports
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/report.json \
  --sample-size 64 \
  --include-proof-details \
  --report-json analysis-reports/report.json \
  --max-depth 4 \
  --max-scan-plies 64
```

`analyze-report-replays` intentionally omits explicit entrants here. By default
it selects standing #1 versus the highest different standing from the published
bot report, so the lab report analysis tab always explains the current top
matchup.

Scratch analysis runs belong in ignored `outputs/`.
