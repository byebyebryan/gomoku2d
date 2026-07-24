# Release Workflow

Purpose: release checklist for the web game and repo-level tags.

This file owns release sequencing. Test commands live in
[`testing.md`](testing.md). Tournament/report generation lives in
[`tournament.md`](tournament.md).

## Public Host

Canonical URL:

```text
https://gomoku2d.byebyebryan.com/
```

GitHub Pages serves the custom domain root. Production builds use
`GOMOKU_BASE_PATH=/`.

## Local Preview

Reserve port `8001` and tmux session `gomoku-preview`:

```sh
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd gomoku-web
npm run build
tmux kill-session -t gomoku2d-preview 2>/dev/null || true
tmux kill-session -t gomoku-preview 2>/dev/null || true
tmux new-session -d -s gomoku-preview -c /home/bryan/code/gomoku2d/gomoku-web \
  'npm run preview -- --host 0.0.0.0 --port 8001'
```

Open `http://localhost:8001/`.

## Release Prep

1. Decide whether screenshots/OG/README hero need refresh.
2. If captures should show the new version, run:

   ```sh
   scripts/set-web-version.sh x.y.z
   ```

3. Run relevant checks from [`testing.md`](testing.md).
   Production npm and Rust dependency audits are required for every release.
4. Refresh curated reports only when bot/analyzer/report behavior or source data
   changed; use [`tournament.md`](tournament.md).
5. Update `CHANGELOG.md` with an empty `[Unreleased]` section and dated release
   section.
6. When authentication, cloud sync, profile persistence, or Firestore behavior
   changed, repeat one live Google sign-in and cloud-sync round trip. Automated
   rules and no-config tests do not replace this account-level check.
7. Review `git diff` for accidental generated or scratch output.

## Report Artifact Gate

Before a release that includes reports:

- curated report JSON is committed under `reports/lab/`;
- report provenance says `"git_dirty": false`;
- `GOMOKU_ALLOW_MISSING_REPORTS=1` is not used for the production build;
- web build copies `/bot-report/report.json`, `/analysis-report/report.json`,
  and all SPA route entries configured in `publish_spa_routes.mjs`.

## Push And CI

```sh
git push origin main
gh run list --branch main --limit 5
gh run watch <run-id> --exit-status
```

If a run fails:

```sh
gh run view <run-id> --log-failed
```

## Cut Release

```sh
VERSION=x.y.z
scripts/release.sh --check "$VERSION"
scripts/release.sh "$VERSION"
```

The release script validates version/changelog/tag state. Pushing the release
tag dispatches GitHub Release, Pages deploy, and Firestore rules deploy
workflows. After deployment, smoke the public URL manually.
