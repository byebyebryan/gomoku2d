import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import {
  parseArguments,
  pathOverlaps,
  redactText,
  validateConfigShape,
  validateNewOutputPath,
  validateOutputPath,
} from "./capture_retrospective_ui.mjs";
import { parseArguments as parseReceiptArguments } from "./record_retrospective_build.mjs";

function validConfig() {
  return {
    schemaVersion: 1,
    sources: [
      {
        tag: "v0.2.4",
        adapter: "v024",
        expectedCommit: "a".repeat(40),
        worktree: "/tmp/v024",
        baseUrl: "http://127.0.0.1:46924/",
        buildReceipt: {},
      },
      {
        tag: "v0.6.2",
        adapter: "v062",
        expectedCommit: "b".repeat(40),
        worktree: "/tmp/v062",
        baseUrl: "http://127.0.0.1:46962/",
        buildReceipt: {},
      },
    ],
  };
}

test("capture config accepts the two exact tag adapters and loopback origins", () => {
  const parsed = validateConfigShape(validConfig());
  assert.deepEqual(parsed.sources.map((source) => source.adapter), ["v024", "v062"]);
});

test("capture config rejects adapter mismatches and reserved or duplicate ports", () => {
  const mismatch = validConfig();
  mismatch.sources[0].adapter = "v062";
  assert.throws(() => validateConfigShape(mismatch), /requires adapter v024/);

  const reserved = validConfig();
  reserved.sources[0].baseUrl = "http://127.0.0.1:8001/";
  assert.throws(() => validateConfigShape(reserved), /reserved or invalid port/);

  const duplicate = validConfig();
  duplicate.sources[1].baseUrl = duplicate.sources[0].baseUrl;
  assert.throws(() => validateConfigShape(duplicate), /different preview port/);
});

test("output checks reject overwrite and source traversal without creating output", async () => {
  const scratch = await mkdtemp(join(tmpdir(), "gomoku-retro-capture-test-"));
  try {
    const source = join(scratch, "source");
    await mkdir(source);
    const output = join(scratch, "new-output");
    assert.equal(await validateOutputPath(output, [source]), output);
    await assert.rejects(validateNewOutputPath(join(source, "nested.json"), [source], "file"), /disjoint/);

    const existing = join(scratch, "existing");
    await writeFile(existing, "keep me\n");
    await assert.rejects(validateNewOutputPath(existing, [source], "file"), /already exists/);
    assert.equal(pathOverlaps(join(source, "nested"), source), true);
    assert.equal(pathOverlaps(output, source), false);
  } finally {
    await rm(scratch, { recursive: true, force: true });
  }
});

test("CLI parser requires one explicit config and new output", () => {
  assert.deepEqual(parseArguments(["--config", "sources.json", "--output", "captures"]), {
    configPath: "sources.json",
    outputPath: "captures",
    validateOnly: false,
    help: false,
  });
  assert.throws(() => parseArguments(["--config", "sources.json"]), /required/);
  assert.throws(() => parseArguments(["--config", "a", "--config", "b", "--output", "c"]), /only once/);
});

test("receipt CLI parser requires explicit tag/worktree config and new output path", () => {
  assert.deepEqual(parseReceiptArguments(["--config", "build.json", "--output", "receipt.json"]), {
    configPath: "build.json",
    outputPath: "receipt.json",
    help: false,
  });
  assert.throws(() => parseReceiptArguments(["--config", "build.json"]), /required/);
});

test("manifest error redaction preserves token keys and removes URL and free-form secrets", () => {
  const redacted = redactText(
    "request https://identity.example.test/path?token=url-secret failed token=free-secret apiKey=key-secret",
  );
  assert.match(redacted, /https:\/\/identity\.example\.test\/path\?\[redacted\]/);
  assert.match(redacted, /token=\[redacted\]/);
  assert.match(redacted, /apiKey=\[redacted\]/);
  assert.doesNotMatch(redacted, /url-secret|free-secret|key-secret/);
});
