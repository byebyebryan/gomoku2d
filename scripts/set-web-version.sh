#!/usr/bin/env bash
# Bump gomoku-web/package.json and package-lock.json without rolling the
# changelog, committing, or tagging.
#
# Use this before release screenshots when the UI should show the upcoming
# version but the release tag is not ready yet.
#
# Usage: scripts/set-web-version.sh <version>   e.g. scripts/set-web-version.sh 0.2.4

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <version>   e.g. $0 0.2.4" >&2
  exit 1
fi

VERSION="$1"

if ! echo "$VERSION" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$'; then
  echo "Invalid version (want MAJOR.MINOR.PATCH[-prerelease]): $VERSION" >&2
  exit 1
fi

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

VERSION_FILES=(gomoku-web/package.json gomoku-web/package-lock.json)

if ! git diff --quiet -- "${VERSION_FILES[@]}"; then
  echo "Version files already have unstaged changes; commit, stash, or restore them first." >&2
  exit 1
fi

if ! git diff --cached --quiet -- "${VERSION_FILES[@]}"; then
  echo "Version files already have staged changes; commit, unstage, or restore them first." >&2
  exit 1
fi

(
  cd gomoku-web
  npm version "$VERSION" --no-git-tag-version --allow-same-version >/dev/null
)

echo "Updated gomoku-web package version to ${VERSION}."
