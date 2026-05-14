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
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d3+pattern-eval,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval,search-d3+pattern-eval+corridor-proof-c16-d8-w4,search-d5+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 63 \
  --threads 22 \
  --report-json reports/latest.json
cargo run --release -p gomoku-eval -- report-html --input reports/latest.json --output reports/index.html --json-href latest.json
jq '.provenance | {git_commit, git_dirty}' reports/latest.json
```

Keep the raw JSON next to the generated HTML so the report can be inspected or
re-rendered without rerunning the tournament.

If only the HTML renderer changes, re-render `index.html` from the existing
clean `latest.json`; do not spend another full tournament just to refresh
presentation. The JSON provenance remains the match-data provenance, while the
HTML can track later renderer polish.

`latest.json` is also the default anchor-rating source for focused gauntlet
runs. Gauntlets can embed selected standings from this full round-robin report
with `--anchor-report reports/latest.json`, which keeps scratch comparisons
calibrated without maintaining a separate cache file. The gauntlet command
validates rule, opening, search budget, and match-cap compatibility before it
uses the reference standings.

The curated replay-analysis report in `../analysis-reports/` is generated from
this `latest.json` and should explain the current top-two matchup by default.
