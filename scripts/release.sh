#!/usr/bin/env bash
# Finalize a prepared release: validate version/changelog, commit the current
# release changes if needed, and create an annotated tag. Does not push.
#
# Expected workflow:
#   1. scripts/set-web-version.sh <version>
#   2. Capture release screenshots/assets and update docs + CHANGELOG.md.
#   3. Run release checks.
#   4. scripts/release.sh <version>
#
# Usage:
#   scripts/release.sh --check <version>
#   scripts/release.sh <version>        e.g. scripts/release.sh 0.2.4

set -euo pipefail

CHECK_ONLY=0
if [ "${1:-}" = "--check" ]; then
  CHECK_ONLY=1
  shift
fi

if [ $# -ne 1 ]; then
  echo "Usage: $0 [--check] <version>   e.g. $0 0.2.4" >&2
  exit 1
fi

VERSION="$1"
TAG="v${VERSION}"

if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$'; then
  echo "Invalid version (want MAJOR.MINOR.PATCH[-prerelease]): $VERSION" >&2
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

if [ "$(git branch --show-current)" != "main" ]; then
  echo "Must run on main (current: $(git branch --show-current))" >&2
  exit 1
fi

if git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
  echo "Tag ${TAG} already exists locally" >&2
  exit 1
fi

if git remote get-url origin >/dev/null 2>&1 &&
  git ls-remote --exit-code --tags origin "refs/tags/${TAG}" >/dev/null 2>&1; then
  echo "Tag ${TAG} already exists on origin" >&2
  exit 1
fi

PKG_VERSION="$(node -p "require('./gomoku-web/package.json').version")"
if [ "$PKG_VERSION" != "$VERSION" ]; then
  echo "gomoku-web/package.json version (${PKG_VERSION}) does not match ${TAG}" >&2
  echo "Run scripts/set-web-version.sh ${VERSION} before finalizing." >&2
  exit 1
fi

if ! grep -q "^## \\[${VERSION}\\] - " CHANGELOG.md; then
  echo "CHANGELOG.md has no [${VERSION}] release section" >&2
  echo "Move the prepared notes from [Unreleased] to [${VERSION}] before finalizing." >&2
  exit 1
fi

UNRELEASED_BODY="$(
  awk '
    /^## \[Unreleased\]$/ {flag=1; next}
    /^## \[/ {flag=0}
    flag {print}
  ' CHANGELOG.md | grep -v '^[[:space:]]*$' | head -n 1 || true
)"

if [ -n "$UNRELEASED_BODY" ]; then
  echo "CHANGELOG.md [Unreleased] is not empty." >&2
  echo "Release notes for ${TAG} should live under [${VERSION}] before tagging." >&2
  exit 1
fi

if ! grep -q "^\\[Unreleased\\]: .*${TAG}\\.\\.\\.HEAD$" CHANGELOG.md; then
  echo "CHANGELOG.md [Unreleased] compare link does not point from ${TAG} to HEAD" >&2
  exit 1
fi

if ! grep -q "^\\[${VERSION}\\]: .*\\.\\.\\.${TAG}$" CHANGELOG.md; then
  echo "CHANGELOG.md [${VERSION}] compare link is missing or malformed" >&2
  exit 1
fi

if [ "$CHECK_ONLY" -eq 1 ]; then
  echo "Release ${TAG} is prepared correctly."
  exit 0
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "Committing prepared release changes for ${TAG}"
  git add -A
  git commit -m "release: ${TAG}"
else
  echo "Working tree clean; tagging current HEAD as ${TAG}"
fi

git tag -a "${TAG}" -m "${TAG}"

cat <<EOF

Release ${TAG} prepared locally.

  Review:  git show ${TAG}
  Push:    git push origin main && git push origin ${TAG}

Pushing main fires the deploy workflow. Pushing the tag fires the GitHub
Release workflow.
EOF
