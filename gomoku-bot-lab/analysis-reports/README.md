# Gomoku2D Analysis Reports

Curated replay-analysis data for the analysis tab of the published `/lab/`
page. The JSON is published under `/analysis-report/report.json`.

The published analysis report is the companion to the published bot report:
sample the Easy/Normal/Hard preset triangle from `reports/report.json`, then
export the forced-corridor explanation data here. The web app renders the
analysis tab from this JSON.

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
  --selector preset-triangle \
  --published-report-json analysis-reports/report.json \
  --max-depth 4 \
  --max-scan-plies 64
```

`analyze-report-replays` intentionally uses the named selector here instead of
explicit entrants, so the lab report analysis tab stays aligned with the current
in-game bot presets.

Scratch analysis runs belong in ignored `outputs/`.
