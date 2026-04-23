#!/usr/bin/env bash
# Backfill GitHub Releases for historical tags using CHANGELOG.md sections.
# Safe to re-run: skips tags that already have a release.
#
# Usage: scripts/backfill-releases.sh [tag ...]
#        scripts/backfill-releases.sh              # all known historical tags
#
# Requires: gh CLI authenticated (`gh auth login`).

set -euo pipefail

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI not found" >&2
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [ $# -gt 0 ]; then
  TAGS=("$@")
else
  TAGS=(v0.1 v0.2.1 v0.2.2 v0.2.3)
fi

PLAY_URL="https://dev.byebyebryan.com/gomoku2d/"
REPO_SLUG="$(gh repo view --json nameWithOwner -q .nameWithOwner)"

for TAG in "${TAGS[@]}"; do
  VERSION="${TAG#v}"

  if ! git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
    echo "skip ${TAG}: tag does not exist locally"
    continue
  fi

  if gh release view "${TAG}" >/dev/null 2>&1; then
    echo "skip ${TAG}: release already exists"
    continue
  fi

  BODY=$(awk -v v="$VERSION" '
    $0 ~ "^## \\["v"\\]" {flag=1; next}
    /^## \[/ {flag=0}
    flag {print}
  ' CHANGELOG.md)

  if [ -z "$BODY" ]; then
    echo "skip ${TAG}: no CHANGELOG section for [${VERSION}]" >&2
    continue
  fi

  THEME=$(printf '%s\n' "$BODY" | awk '
      /^\*\*Theme:/ {grab=1}
      grab {print}
      grab && /\*\*$/ {exit}
    ' | tr '\n' ' ' \
    | sed -E 's/^\*\*Theme: ?//; s/ ?\*\*.*$//; s/\.$//; s/`//g; s/[[:space:]]+$//')
  THEME_SHORT=$(printf '%s' "$THEME" | sed -E 's/ ?[—,].*//')

  if [ -n "$THEME_SHORT" ]; then
    TITLE="${TAG} — ${THEME_SHORT}"
  else
    TITLE="${TAG}"
  fi

  TMP_BODY=$(mktemp)
  trap 'rm -f "$TMP_BODY"' EXIT

  {
    echo "**Play this build:** ${PLAY_URL}"
    echo
    printf '%s\n' "$BODY"
    echo
    echo "---"
    echo
    echo "Full changelog: [CHANGELOG.md](https://github.com/${REPO_SLUG}/blob/main/CHANGELOG.md)"
  } > "$TMP_BODY"

  echo "creating release ${TAG}: ${TITLE}"
  gh release create "${TAG}" \
    --title "${TITLE}" \
    --notes-file "${TMP_BODY}" \
    --verify-tag

  rm -f "$TMP_BODY"
  trap - EXIT
done
