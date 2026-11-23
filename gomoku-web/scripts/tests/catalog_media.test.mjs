import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { readRepositoryPngIdentity } from "../catalog_media.mjs";
import { readHistoryCatalog } from "../history_catalog.mjs";
import { readReviewCatalog } from "../review_catalog.mjs";

const PNG_SIGNATURE = Buffer.from("89504e470d0a1a0a", "hex");

function pngHeader(width = 37, height = 23) {
  const bytes = Buffer.alloc(24);
  PNG_SIGNATURE.copy(bytes);
  bytes.writeUInt32BE(width, 16);
  bytes.writeUInt32BE(height, 20);
  return bytes;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function identityInput(sourcePath, bytes, overrides = {}) {
  return {
    sourcePath,
    sha256: sha256(bytes),
    width: 37,
    height: 23,
    ...overrides,
  };
}

function verifyMedia(repositoryRoot, media, errors = {}) {
  return readRepositoryPngIdentity(repositoryRoot, media, {
    outsideRepository: errors.outsideRepository ?? "Review media leaves repository",
    identityMismatch: errors.identityMismatch ?? `Review media identity mismatch: ${media.sourcePath}`,
  });
}

test("history and review catalog readers accept their current source catalogs", async () => {
  const [history, reviews] = await Promise.all([
    readHistoryCatalog(),
    readReviewCatalog(),
  ]);

  assert.ok(history.snapshots.length > 0);
  assert.ok(reviews.length > 0);
});

test("shared catalog PNG reader verifies raw bytes and IHDR identity", async (t) => {
  const root = await mkdtemp(join(tmpdir(), "catalog-media-test-"));
  t.after(async () => rm(root, { recursive: true, force: true }));
  const repositoryRoot = join(root, "repo");
  const mediaDirectory = join(repositoryRoot, "media");
  await mkdir(mediaDirectory, { recursive: true });

  const bytes = pngHeader();
  const sourcePath = "media/sample.png";
  await writeFile(join(repositoryRoot, sourcePath), bytes);
  const result = await verifyMedia(repositoryRoot, identityInput(sourcePath, bytes));

  assert.equal(result, join(repositoryRoot, sourcePath));
});

test("shared catalog PNG reader rejects short, corrupt, and dimension-mismatched files", async (t) => {
  const root = await mkdtemp(join(tmpdir(), "catalog-media-invalid-test-"));
  t.after(async () => rm(root, { recursive: true, force: true }));
  const repositoryRoot = join(root, "repo");
  const mediaDirectory = join(repositoryRoot, "media");
  await mkdir(mediaDirectory, { recursive: true });

  const short = pngHeader().subarray(0, 23);
  await writeFile(join(mediaDirectory, "short.png"), short);
  await assert.rejects(
    verifyMedia(repositoryRoot, identityInput("media/short.png", short)),
    /Review media identity mismatch: media\/short\.png/,
  );

  const corrupt = pngHeader();
  corrupt[0] = 0;
  await writeFile(join(mediaDirectory, "corrupt.png"), corrupt);
  await assert.rejects(
    verifyMedia(repositoryRoot, identityInput("media/corrupt.png", corrupt)),
    /Review media identity mismatch: media\/corrupt\.png/,
  );

  const dimensions = pngHeader();
  await writeFile(join(mediaDirectory, "dimensions.png"), dimensions);
  await assert.rejects(
    verifyMedia(repositoryRoot, identityInput("media/dimensions.png", dimensions, { width: 38 })),
    /Review media identity mismatch: media\/dimensions\.png/,
  );

  await assert.rejects(
    verifyMedia(repositoryRoot, identityInput("media/dimensions.png", dimensions, { sha256: "0".repeat(64) })),
    /Review media identity mismatch: media\/dimensions\.png/,
  );
});

test("shared catalog PNG reader enforces physical repository containment", async (t) => {
  const root = await mkdtemp(join(tmpdir(), "catalog-media-containment-test-"));
  t.after(async () => rm(root, { recursive: true, force: true }));
  const repositoryRoot = join(root, "repo");
  const mediaDirectory = join(repositoryRoot, "media");
  await mkdir(mediaDirectory, { recursive: true });

  const bytes = pngHeader();
  const outsidePath = join(root, "outside.png");
  await writeFile(outsidePath, bytes);
  await symlink(outsidePath, join(mediaDirectory, "linked.png"));

  await assert.rejects(
    verifyMedia(repositoryRoot, identityInput("media/linked.png", bytes)),
    /Review media leaves repository/,
  );
});
