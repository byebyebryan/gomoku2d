import assert from 'node:assert/strict';
import { chmodSync, existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

const testDirectory = path.dirname(fileURLToPath(import.meta.url));
const repositoryRoot = path.resolve(testDirectory, '../..');
const validatorPath = path.join(repositoryRoot, 'scripts/validate_release_ref.mjs');

function withReleaseFixture(
  version,
  callback,
  { changelogVersion = version, releaseBody = '**Theme: fixture release.**' } = {},
) {
  const root = mkdtempSync(path.join(os.tmpdir(), 'gomoku2d-release-validator-'));
  mkdirSync(path.join(root, 'gomoku-web'));
  writeFileSync(
    path.join(root, 'gomoku-web/package.json'),
    JSON.stringify({ name: 'gomoku-web', version }, null, 2),
  );

  const tag = `v${changelogVersion}`;
  writeFileSync(
    path.join(root, 'CHANGELOG.md'),
    [
      '# Changelog',
      '',
      '## [Unreleased]',
      '',
      `## [${changelogVersion}] - 2026-11-21`,
      '',
      releaseBody,
      '',
      `[Unreleased]: https://example.test/compare/${tag}...HEAD`,
      `[${changelogVersion}]: https://example.test/compare/v0.6.5...${tag}`,
      '',
    ].join('\n'),
  );

  try {
    callback(root, tag);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function validate(root, tag) {
  return spawnSync(process.execPath, [validatorPath, tag], {
    cwd: root,
    encoding: 'utf8',
  });
}

test('shared validator preserves prerelease and build metadata exactly', () => {
  const version = '1.2.3-rc.1+build.7';
  withReleaseFixture(version, (root, tag) => {
    const result = validate(root, tag);
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /v1\.2\.3-rc\.1\+build\.7/);
  });
});

test('shared validator rejects malformed SemVer and nonmatching package versions', () => {
  withReleaseFixture('1.2.3', (root) => {
    const badShape = validate(root, 'v01.2.3');
    assert.notEqual(badShape.status, 0);
    assert.match(badShape.stderr, /not vMAJOR\.MINOR\.PATCH/);
  });

  withReleaseFixture('1.2.3+build.7', (root, tag) => {
    const mismatch = validate(root, tag);
    assert.notEqual(mismatch.status, 0);
    assert.match(mismatch.stderr, /does not exactly match/);
  }, { changelogVersion: '1.2.3+build.8' });
});

test('shared validator ignores footer links when checking release notes and pins the Unreleased compare base', () => {
  withReleaseFixture('1.2.3', (root, tag) => {
    const changelogPath = path.join(root, 'CHANGELOG.md');
    const withWrongCompareBase = readFileSync(changelogPath, 'utf8')
      .replace(`/compare/${tag}...HEAD`, '/compare/v1.2.2...HEAD');
    writeFileSync(changelogPath, withWrongCompareBase);
    const wrongBase = validate(root, tag);
    assert.notEqual(wrongBase.status, 0);
    assert.match(wrongBase.stderr, /compare link must end in v1\.2\.3\.\.\.HEAD/);
  });

  withReleaseFixture('1.2.3', (root, tag) => {
    const footerOnly = validate(root, tag);
    assert.notEqual(footerOnly.status, 0);
    assert.match(footerOnly.stderr, /release section is empty/);
  }, { releaseBody: '' });
});

test('release.sh requires origin, fetches exact origin/main, distinguishes tag lookup failure, and creates only a local tag', () => {
  const fixtureRoot = mkdtempSync(path.join(os.tmpdir(), 'gomoku2d-release-git-'));
  const repository = path.join(fixtureRoot, 'work');
  const origin = path.join(fixtureRoot, 'origin.git');
  const version = '1.2.3-rc.1+build.7';
  const tag = `v${version}`;

  function git(args, cwd, options = {}) {
    const result = spawnSync('git', args, { cwd, encoding: 'utf8', ...options });
    assert.equal(result.status, 0, result.stderr || result.stdout);
    return result.stdout;
  }

  function release(args, cwd, env = process.env) {
    return spawnSync('bash', [path.join(cwd, 'scripts/release.sh'), ...args], {
      cwd,
      env,
      encoding: 'utf8',
    });
  }

  try {
    git(['init', '--bare', '--initial-branch=main', origin], fixtureRoot);
    git(['init', '--initial-branch=main', repository], fixtureRoot);
    git(['config', 'user.name', 'Fixture'], repository);
    git(['config', 'user.email', 'fixture@example.test'], repository);
    git(['config', 'commit.gpgsign', 'false'], repository);
    mkdirSync(path.join(repository, 'gomoku-web'));
    mkdirSync(path.join(repository, 'scripts'));
    writeFileSync(
      path.join(repository, 'gomoku-web/package.json'),
      JSON.stringify({ name: 'gomoku-web', version }, null, 2),
    );
    for (const script of ['release.sh', 'validate_release_ref.mjs']) {
      const source = path.join(repositoryRoot, 'scripts', script);
      const target = path.join(repository, 'scripts', script);
      writeFileSync(target, readFileSync(source));
    }
    chmodSync(path.join(repository, 'scripts/release.sh'), 0o755);
    writeFileSync(
      path.join(repository, 'CHANGELOG.md'),
      [
        '# Changelog',
        '',
        '## [Unreleased]',
        '',
        `## [${version}] - 2026-11-21`,
        '',
        '**Theme: fixture release.**',
        '',
        `[Unreleased]: https://example.test/compare/${tag}...HEAD`,
        `[${version}]: https://example.test/compare/v0.6.5...${tag}`,
        '',
      ].join('\n'),
    );
    git(['add', '.'], repository);
    git(['commit', '-m', 'Fixture release preparation'], repository);

    const missingOrigin = release(['--check', version], repository);
    assert.notEqual(missingOrigin.status, 0);
    assert.match(missingOrigin.stderr, /Remote origin is required/);

    git(['remote', 'add', 'origin', origin], repository);
    git(['push', '-u', 'origin', 'main'], repository);
    git(['update-ref', '-d', 'refs/remotes/origin/main'], repository);
    git(['config', '--replace-all', 'remote.origin.fetch', '+refs/heads/other:refs/remotes/origin/other'], repository);

    const absentTag = release(['--check', version], repository);
    assert.equal(absentTag.status, 0, absentTag.stderr || absentTag.stdout);
    assert.match(absentTag.stdout, new RegExp(`Release ${tag.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')} is prepared correctly`));

    git(['tag', 'main', 'HEAD'], repository);
    git(['commit', '--allow-empty', '-m', 'Unpushed work'], repository);
    const shadowedMain = release(['--check', version], repository);
    assert.notEqual(shadowedMain.status, 0);
    assert.match(shadowedMain.stderr, /Local main has unpushed commits/);
    git(['tag', '-d', 'main'], repository);
    git(['push', 'origin', 'HEAD:refs/heads/main'], repository);

    git(['tag', '-a', tag, '-m', tag], repository);
    git(['push', 'origin', tag], repository);
    git(['tag', '-d', tag], repository);
    const existingRemoteTag = release(['--check', version], repository);
    assert.notEqual(existingRemoteTag.status, 0);
    assert.match(existingRemoteTag.stderr, new RegExp(`Tag ${tag.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')} already exists on origin`));
    git(['push', 'origin', `:refs/tags/${tag}`], repository);

    const mockBin = path.join(fixtureRoot, 'mock-bin');
    mkdirSync(mockBin);
    const realGit = process.env.PATH.split(path.delimiter)
      .map((directory) => path.join(directory, 'git'))
      .find((candidate) => existsSync(candidate));
    assert.ok(realGit, 'system Git executable is available');
    writeFileSync(
      path.join(mockBin, 'git'),
      [
        '#!/usr/bin/env bash',
        "if [ \"$1\" = 'ls-remote' ]; then",
        "  printf '%s\\n' 'mocked remote query failure' >&2",
        '  exit 128',
        'fi',
        'exec "$REAL_GIT" "$@"',
        '',
      ].join('\n'),
    );
    chmodSync(path.join(mockBin, 'git'), 0o755);
    const failedQuery = release(['--check', version], repository, {
      ...process.env,
      PATH: `${mockBin}${path.delimiter}${process.env.PATH}`,
      REAL_GIT: realGit,
    });
    assert.notEqual(failedQuery.status, 0);
    assert.match(failedQuery.stderr, /Failed to query origin for tag/);
    assert.match(failedQuery.stderr, /mocked remote query failure/);

    const createTag = release([version], repository);
    assert.equal(createTag.status, 0, createTag.stderr || createTag.stdout);
    assert.match(git(['cat-file', '-p', `refs/tags/${tag}`], repository), new RegExp(`^tag ${tag.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}$`, 'm'));
    assert.equal(git(['rev-parse', `refs/tags/${tag}^{commit}`], repository), git(['rev-parse', 'HEAD'], repository));
  } finally {
    rmSync(fixtureRoot, { recursive: true, force: true });
  }
});

test('all tag workflows validate before publish/build/auth and manual guards remain', () => {
  const release = readFileSync(path.join(repositoryRoot, '.github/workflows/release.yml'), 'utf8');
  const ci = readFileSync(path.join(repositoryRoot, '.github/workflows/ci.yml'), 'utf8');
  const pages = readFileSync(path.join(repositoryRoot, '.github/workflows/deploy.yml'), 'utf8');
  const firestore = readFileSync(path.join(repositoryRoot, '.github/workflows/deploy-firestore.yml'), 'utf8');

  assert.ok(release.indexOf('node scripts/validate_release_ref.mjs') < release.indexOf('uses: softprops/action-gh-release'));
  assert.ok(ci.indexOf('name: Release validation fixtures') < ci.indexOf('name: Install wasm-pack'));
  assert.ok(pages.indexOf('node scripts/validate_release_ref.mjs') < pages.indexOf('run: npm run build'));
  assert.ok(firestore.indexOf('node scripts/validate_release_ref.mjs') < firestore.indexOf('name: Authenticate to Google Cloud'));
  assert.ok(firestore.indexOf('Validate exact manual source SHA') < firestore.indexOf('name: Authenticate to Google Cloud'));

  assert.match(pages, /workflow_dispatch:/);
  assert.match(firestore, /expected_sha:[\s\S]*?required: true/);
  assert.match(firestore, /\^\[0-9a-f\]\{40\}\$/);
});
