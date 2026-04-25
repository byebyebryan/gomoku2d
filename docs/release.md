# Release Workflow

This is the release checklist for the web game and repo-level tags.

## Local Preview

Reserve port `8001` for the local production preview.

The tmux session name is:

```sh
gomoku2d-preview
```

To rebuild and restart it:

```sh
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd gomoku-web
npm run build
tmux kill-session -t gomoku2d-preview 2>/dev/null || true
tmux new-session -d -s gomoku2d-preview -c /home/bryan/code/gomoku2d/gomoku-web \
  'npm run preview -- --host 0.0.0.0 --port 8001'
```

Open:

```text
http://localhost:8001/
```

The local preview build uses Vite's default root base path. GitHub Pages deploy
still builds with `GOMOKU_BASE_PATH=/gomoku2d/`.

## Version For Captures

If screenshots or recordings should show the upcoming version, bump the web
package version before capturing:

```sh
scripts/set-web-version.sh 0.2.4
```

This updates `gomoku-web/package.json` and `gomoku-web/package-lock.json`
without touching the changelog, committing, or tagging.

## Release Asset Pass

Before tagging:

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
GOMOKU_BASE_PATH=/gomoku2d/ npm run build
PLAYWRIGHT_BASE_URL=http://127.0.0.1:8001 npm run playtest:smoke
```

The Vite chunk-size warning for Phaser is expected and is not currently
release-blocking.

## Finalize

After reviewing `git status` and the diff, finalize the prepared release:

```sh
scripts/release.sh --check 0.2.4
scripts/release.sh 0.2.4
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
git push origin v0.2.4
```

Pushing `main` fires:

- `.github/workflows/deploy.yml`

Pushing the tag fires:

- `.github/workflows/release.yml`
