# Release Workflow

This is the release checklist for the web game and repo-level tags.

## Public Host

The canonical public URL is:

```text
https://gomoku2d.byebyebryan.com/
```

GitHub Pages is configured with that custom domain and HTTPS enforcement. DNS
is managed in Cloudflare with `gomoku2d` as a `CNAME` to
`byebyebryan.github.io`.

## Local Preview

Reserve port `8001` for the local production preview.

The tmux session name is:

```sh
gomoku-preview
```

To rebuild and restart it:

```sh
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd gomoku-web
npm run build
tmux kill-session -t gomoku2d-preview 2>/dev/null || true
tmux kill-session -t gomoku-preview 2>/dev/null || true
tmux new-session -d -s gomoku-preview -c /home/bryan/code/gomoku2d/gomoku-web \
  'npm run preview -- --host 0.0.0.0 --port 8001'
```

Open:

```text
http://localhost:8001/
```

The local preview build uses Vite's default root base path. GitHub Pages also
builds with `GOMOKU_BASE_PATH=/` because the public app is served from the
custom domain root:

```text
https://gomoku2d.byebyebryan.com/
```

## Version For Captures

If screenshots or recordings should show the upcoming version, bump the web
package version before capturing:

```sh
VERSION=0.4.1
scripts/set-web-version.sh "$VERSION"
```

This updates `gomoku-web/package.json` and `gomoku-web/package-lock.json`
without touching the changelog, committing, or tagging.

## Release Asset Pass

Run this pass for visual releases, asset changes, or releases where README/social
captures should change. For backend/docs-only releases, record that the existing
captures remain current and skip the refresh.

When needed before tagging:

- capture the desktop/mobile screenshot set
- capture or refresh the README hero GIF
- update `gomoku-web/assets/og_source.png`
- regenerate `gomoku-web/public/og.png` at `1200x630`
- update `docs/ui_screenshot_review.md`, `docs/ui_design.md`, `docs/roadmap.md`,
  and `CHANGELOG.md`
- leave `CHANGELOG.md` with an empty `[Unreleased]` section and a dated
  `[x.y.z]` section for the release

## Checks

Run the same checks the release candidate should pass:

```sh
cd gomoku-bot-lab
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

cd ..
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler

cd gomoku-web
npm test
npm run test:rules
npm audit --omit=dev
GOMOKU_BASE_PATH=/ npm run build
PLAYWRIGHT_BASE_URL=http://127.0.0.1:8001 npm run playtest:smoke
```

The Vite chunk-size warning for Phaser is expected and is not currently
release-blocking.

## Bot And Analysis Report Refresh

Curated bot-lab report artifacts live in `gomoku-bot-lab/reports/` and are
published under `/bot-report/` by the web build. Scratch reports belong in the
ignored `gomoku-bot-lab/outputs/` folder.

Curated replay-analysis report artifacts live in
`gomoku-bot-lab/analysis-reports/` and are published under
`/analysis-report/`. The analysis report is intentionally tied to the bot report:
it samples the head-to-head games between the current top two standings in
`gomoku-bot-lab/reports/latest.json`.

The report JSON captures git provenance when the tournament command runs, so
refresh it only from a clean committed toolchain:

1. Commit bot/report code changes first.
2. Confirm `git status --short` is clean.
3. Run the curated tournament into `gomoku-bot-lab/reports/latest.json`.
4. Render `gomoku-bot-lab/reports/index.html` from that JSON.
5. Confirm `reports/latest.json` says `"git_dirty": false`.
6. Generate the top-two analysis report into
   `gomoku-bot-lab/analysis-reports/latest.json` and
   `gomoku-bot-lab/analysis-reports/index.html`.
7. Commit the report artifacts as a follow-up commit.

If only the HTML report renderer changed after a clean tournament, do not rerun
the long tournament just to update presentation. Re-render
`gomoku-bot-lab/reports/index.html` from the existing `latest.json`, confirm the
JSON is still clean, and commit the HTML together with the renderer change or as
its own report-render refresh. The JSON provenance remains the tournament
provenance, not the renderer commit.

Current curated command, from `gomoku-bot-lab/`:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d5,search-d5+tactical-cap-8,search-d7+tactical-cap-8,search-d3+pattern-eval,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval \
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
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --sample-size 64 \
  --include-proof-details \
  --report-json analysis-reports/latest.json \
  --report-html analysis-reports/index.html \
  --max-depth 4
jq '{source, total, analyzed, failed, model, summary}' analysis-reports/latest.json
```

For the curated analysis report, sanity-check at minimum:

- `source == "reports/latest.json:Top 2 entrants"`
- `analyzed == total` and `failed == 0`
- `model.reply_policy == "corridor_replies"`
- `model.max_depth == 4` and `model.max_scan_plies == 64`
- summary counts look intentional for the current report sample

## Push And CI Baseline

For normal hardening commits, push `main` and let CI prove the integration
branch before dispatching any deploy workflow:

```sh
git push origin main
gh run list --branch main --limit 5
gh run watch <run-id> --exit-status
```

`main` pushes run `.github/workflows/ci.yml` only. They do not publish the web
site and do not deploy Firestore rules.

If a run fails, inspect the failing job before retrying:

```sh
gh run view <run-id> --log-failed
```

## Finalize

After reviewing `git status` and the diff, finalize the prepared release:

```sh
VERSION=0.4.1
scripts/release.sh --check "$VERSION"
scripts/release.sh "$VERSION"
```

`release.sh` validates:

- current branch is `main`
- the local and remote tag do not already exist
- `gomoku-web/package.json` matches the requested version
- `CHANGELOG.md` has a dated release section for the version
- `CHANGELOG.md` has an empty `[Unreleased]` section
- compare links point from the new tag correctly

If the working tree has changes, the script stages them, commits
`release: vX.Y.Z`, and creates an annotated tag. If the tree is already clean,
it tags the current `HEAD`.

Push when ready:

```sh
git push origin main
git push origin "v$VERSION"
```

Pushing `main` updates the release commit. It does not publish the site.

Pushing the tag fires:

- `.github/workflows/release.yml`
- `.github/workflows/deploy.yml`
- `.github/workflows/deploy-firestore.yml`

For a custom-domain smoke redeploy without cutting a release, run the Pages
workflow manually after the relevant deploy config has been pushed to `main`:

```sh
gh workflow run deploy.yml --ref main
```

For a Firestore-rules smoke deploy without cutting a release, run the Firestore
workflow manually from `main`:

```sh
gh workflow run deploy-firestore.yml --ref main
```

Use manual dispatch only when the current `main` source is intentionally
deployable. If Firestore rules require a new web write shape, deploy the
matching web build as well or cut a tag so Pages and rules deploy from the same
source.

Watch deploy status:

```sh
gh run list --workflow deploy.yml --limit 3
gh run list --workflow deploy-firestore.yml --limit 3
```

## Production Smoke

After a tag release or manual deploy, run the smallest smoke that covers the
changed surface:

- Home loads from `https://gomoku2d.byebyebryan.com/`.
- `/profile`, `/privacy/`, and `/terms/` return `200`.
- If report publishing changed: `/bot-report/`, `/bot-report/latest.json`,
  `/analysis-report/`, and `/analysis-report/latest.json` return `200`.
- Local-only match/replay still works without signing in.
- If auth/profile changed: sign in from production, refresh, sign out, sign in
  again, and confirm history/profile continuity.
- If match history or rules changed: finish one signed-in match, confirm it
  appears in Profile, open Replay, and confirm Firestore rules deployment used
  the intended source ref.
- If Reset Profile changed: reset a signed-in test profile, confirm old rows do
  not reappear, then save one post-reset match.
- If Delete Cloud changed: delete a signed-in test profile, confirm the app
  signs out and Firestore no longer has `profiles/{uid}`, then sign in again
  only if recreating a fresh cloud profile is intended.
- If profile schema changed without migration: confirm old alpha test documents
  were deleted or rewritten before deploying stricter rules.
