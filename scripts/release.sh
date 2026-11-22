#!/usr/bin/env bash
# Finalize a committed release: validate version/changelog and create an
# annotated tag. Does not stage, commit, or push.
#
# Expected workflow:
#   1. scripts/set-web-version.sh <version>
#   2. Capture release screenshots/assets and update docs + CHANGELOG.md.
#   3. Run release checks.
#   4. Commit reviewed release files, push main, and verify CI on that commit.
#   5. scripts/release.sh <version>
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

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

node scripts/validate_release_ref.mjs "$TAG"

if [ "$(git branch --show-current)" != "main" ]; then
  echo "Must run on main (current: $(git branch --show-current))" >&2
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree is dirty. Commit reviewed release files and push main before releasing." >&2
  exit 1
fi

LOCAL_MAIN="$(git rev-parse --verify HEAD)"

if ! git remote get-url origin >/dev/null 2>&1; then
  echo "Remote origin is required; configure origin before releasing." >&2
  exit 1
fi

if FETCH_OUTPUT="$(git fetch --quiet --no-tags origin +refs/heads/main:refs/remotes/origin/main 2>&1)"; then
  :
else
  FETCH_STATUS=$?
  echo "Failed to fetch origin/main (git fetch exited ${FETCH_STATUS}); cannot verify the release source." >&2
  if [ -n "$FETCH_OUTPUT" ]; then
    echo "$FETCH_OUTPUT" >&2
  fi
  exit 1
fi

if ! git show-ref --verify --quiet refs/remotes/origin/main; then
  echo "Remote origin did not provide origin/main; cannot verify the release source." >&2
  exit 1
fi

ORIGIN_MAIN="$(git rev-parse --verify refs/remotes/origin/main)"
FETCHED_MAIN="$(git rev-parse --verify FETCH_HEAD)"

if [ "$FETCHED_MAIN" != "$ORIGIN_MAIN" ]; then
  echo "Fetched origin/main does not match refs/remotes/origin/main; cannot verify the release source." >&2
  exit 1
fi

if [ "$LOCAL_MAIN" != "$ORIGIN_MAIN" ]; then
  MERGE_BASE="$(git merge-base "$LOCAL_MAIN" "$ORIGIN_MAIN" || true)"
  if [ "$LOCAL_MAIN" = "$MERGE_BASE" ]; then
    echo "Local main is behind origin/main; pull before releasing." >&2
  elif [ "$ORIGIN_MAIN" = "$MERGE_BASE" ]; then
    echo "Local main has unpushed commits; push main before releasing." >&2
  else
    echo "Local main and origin/main have diverged; resolve before releasing." >&2
  fi
  exit 1
fi

if git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
  echo "Tag ${TAG} already exists locally" >&2
  exit 1
fi

if REMOTE_TAG_OUTPUT="$(git ls-remote --exit-code --tags origin "refs/tags/${TAG}" 2>&1)"; then
  echo "Tag ${TAG} already exists on origin" >&2
  exit 1
else
  REMOTE_TAG_STATUS=$?
  if [ "$REMOTE_TAG_STATUS" -ne 2 ]; then
    echo "Failed to query origin for tag ${TAG} (git ls-remote exited ${REMOTE_TAG_STATUS}); cannot confirm that the tag is absent." >&2
    if [ -n "$REMOTE_TAG_OUTPUT" ]; then
      echo "$REMOTE_TAG_OUTPUT" >&2
    fi
    exit 1
  fi
fi

if [ "$CHECK_ONLY" -eq 1 ]; then
  echo "Release ${TAG} is prepared correctly."
  exit 0
fi

echo "Working tree clean; tagging verified commit ${LOCAL_MAIN} as ${TAG}"
git tag -a "${TAG}" "${LOCAL_MAIN}" -m "${TAG}"

cat <<EOF

Release ${TAG} prepared locally.

  Review:  git show ${TAG}
  Push:    git push origin main && git push origin ${TAG}

Pushing main updates the release commit without publishing the site. Pushing
the tag fires the GitHub Release, Pages deploy, and Firestore rules deploy
workflows.
EOF
