#!/usr/bin/env bash
# Prepare a release: bump gomoku-web/package.json, roll CHANGELOG
# [Unreleased] to the new version, commit, and tag. Does not push.
#
# Usage: scripts/release.sh <version>   e.g. scripts/release.sh 0.2.4

set -euo pipefail

if [ $# -ne 1 ]; then
  echo "Usage: $0 <version>   e.g. $0 0.2.4" >&2
  exit 1
fi

VERSION="$1"
TAG="v${VERSION}"
DATE="$(date -u +%Y-%m-%d)"

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

if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree dirty — commit or stash first" >&2
  exit 1
fi

if git rev-parse --verify --quiet "refs/tags/${TAG}" >/dev/null; then
  echo "Tag ${TAG} already exists" >&2
  exit 1
fi

if ! grep -q '^## \[Unreleased\]$' CHANGELOG.md; then
  echo "CHANGELOG.md has no [Unreleased] section" >&2
  exit 1
fi

UNRELEASED_BODY=$(awk '
  /^## \[Unreleased\]$/ {flag=1; next}
  /^## \[/ {flag=0}
  flag {print}
' CHANGELOG.md | grep -v '^[[:space:]]*$' | head -n 1)

if [ -z "$UNRELEASED_BODY" ]; then
  echo "CHANGELOG.md [Unreleased] section is empty — add notes before releasing" >&2
  exit 1
fi

echo "Preparing release ${TAG} (${DATE})"

# 1. Bump package.json (npm handles package-lock.json too). This is safe if
#    scripts/set-web-version.sh already prepared the same version for screenshots.
scripts/set-web-version.sh "$VERSION" >/dev/null

# 2. Roll CHANGELOG: rename [Unreleased] to [VERSION] - DATE, insert fresh
#    [Unreleased] above it, and update the compare-link refs at the bottom.
python3 - "$VERSION" "$DATE" <<'PY'
import re
import sys

version, date = sys.argv[1], sys.argv[2]

with open('CHANGELOG.md', encoding='utf-8') as f:
    content = f.read()

# 1. Insert a fresh [Unreleased] header above the old one, then rename the
#    old header to the released version.
replacement = f'## [Unreleased]\n\n## [{version}] - {date}'
new_content, n = re.subn(
    r'^## \[Unreleased\]$',
    replacement,
    content,
    count=1,
    flags=re.MULTILINE,
)
if n != 1:
    raise SystemExit('could not rewrite [Unreleased] header')
content = new_content

# 2. Update compare-link refs at the bottom, if present.
#    Expected shape: [Unreleased]: <base>/compare/<prev_tag>...HEAD
unreleased_ref_re = re.compile(
    r'^\[Unreleased\]: (?P<base>https?://\S+/compare/)(?P<prev>\S+?)\.\.\.HEAD$',
    re.MULTILINE,
)
m = unreleased_ref_re.search(content)
if m:
    base = m.group('base')
    prev = m.group('prev')
    new_tag = f'v{version}'
    replacement_refs = (
        f'[Unreleased]: {base}{new_tag}...HEAD\n'
        f'[{version}]: {base}{prev}...{new_tag}'
    )
    content = unreleased_ref_re.sub(replacement_refs, content, count=1)

with open('CHANGELOG.md', 'w', encoding='utf-8') as f:
    f.write(content)
PY

# 3. Commit
git add gomoku-web/package.json gomoku-web/package-lock.json CHANGELOG.md
git commit -m "release: ${TAG}"

# 4. Annotated tag
git tag -a "${TAG}" -m "${TAG}"

cat <<EOF

Release ${TAG} prepared locally.

  Review:  git show ${TAG}
  Push:    git push origin main && git push origin ${TAG}

Pushing the tag fires the deploy and release workflows.
EOF
