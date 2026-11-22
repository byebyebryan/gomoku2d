#!/usr/bin/env node

import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

function fail(message) {
  console.error(`Release validation failed: ${message}`);
  process.exit(1);
}

if (process.argv.length !== 3) {
  fail('usage: node scripts/validate_release_ref.mjs <tag> (for example v0.7.0-rc.1+build.2)');
}

const tag = process.argv[2];
const numeric = '(?:0|[1-9][0-9]*)';
const prereleaseIdentifier = '(?:0|[1-9][0-9]*|[0-9A-Za-z-]*[A-Za-z-][0-9A-Za-z-]*)';
const buildIdentifier = '[0-9A-Za-z-]+';
const semver = new RegExp(
  `^v(${numeric}\\.${numeric}\\.${numeric}(?:-${prereleaseIdentifier}(?:\\.${prereleaseIdentifier})*)?(?:\\+${buildIdentifier}(?:\\.${buildIdentifier})*)?)$`,
);
const match = semver.exec(tag);
if (!match) {
  fail(`tag ${JSON.stringify(tag)} is not vMAJOR.MINOR.PATCH with optional SemVer prerelease/build metadata`);
}

const version = match[1];
let packageVersion;
try {
  packageVersion = JSON.parse(readFileSync(resolve('gomoku-web/package.json'), 'utf8')).version;
} catch (error) {
  fail(`could not read gomoku-web/package.json: ${error.message}`);
}

if (packageVersion !== version) {
  fail(`gomoku-web/package.json version (${packageVersion}) does not exactly match ${tag}`);
}

let changelog;
try {
  changelog = readFileSync(resolve('CHANGELOG.md'), 'utf8');
} catch (error) {
  fail(`could not read CHANGELOG.md: ${error.message}`);
}

const lines = changelog.split(/\r?\n/);
const sectionHeader = `## [${version}] - `;
const releaseHeaderIndex = lines.findIndex((line) => line.startsWith(sectionHeader));
if (releaseHeaderIndex < 0) {
  fail(`CHANGELOG.md has no dated [${version}] release section`);
}

function sectionBody(headerIndex) {
  const nextHeaderIndex = lines.findIndex(
    (line, index) => index > headerIndex && /^## \[/.test(line),
  );
  const footerIndex = lines.findIndex(
    (line, index) => index > headerIndex && /^\[[^\]]+\]:\s/.test(line),
  );
  const endIndex = [nextHeaderIndex, footerIndex, lines.length]
    .filter((index) => index >= 0)
    .reduce((end, index) => Math.min(end, index), lines.length);
  return lines.slice(headerIndex + 1, endIndex).join('\n');
}

if (!sectionBody(releaseHeaderIndex).trim()) {
  fail(`CHANGELOG.md [${version}] release section is empty`);
}

const unreleasedHeaderIndex = lines.findIndex((line) => line === '## [Unreleased]');
if (unreleasedHeaderIndex < 0) {
  fail('CHANGELOG.md has no [Unreleased] section');
}
if (sectionBody(unreleasedHeaderIndex).trim()) {
  fail(`CHANGELOG.md [Unreleased] is not empty; move its notes under [${version}] before tagging`);
}

function compareUrl(label) {
  const prefix = `[${label}]: `;
  const line = lines.find((candidate) => candidate.startsWith(prefix));
  return line ? line.slice(prefix.length).trim() : null;
}

function hasCompareSuffix(url, expectedRight, expectedLeft = null) {
  if (!url || !url.includes('/compare/')) return false;
  const compareStart = url.indexOf('/compare/') + '/compare/'.length;
  const separator = url.lastIndexOf('...');
  const left = url.slice(compareStart, separator);
  return separator > compareStart
    && (expectedLeft === null ? left.length > 0 : left === expectedLeft)
    && url.slice(separator + 3) === expectedRight;
}

const unreleasedCompare = compareUrl('Unreleased');
if (!hasCompareSuffix(unreleasedCompare, 'HEAD', tag)) {
  fail(`CHANGELOG.md [Unreleased] compare link must end in ${tag}...HEAD`);
}

const releaseCompare = compareUrl(version);
if (!hasCompareSuffix(releaseCompare, tag)) {
  fail(`CHANGELOG.md [${version}] compare link must end in ...${tag}`);
}

console.log(`Release ${tag} package and changelog validation passed.`);
