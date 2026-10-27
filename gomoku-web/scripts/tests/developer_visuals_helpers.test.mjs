import assert from "node:assert/strict";
import { test } from "node:test";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  decodeReviewImages,
  persistDeveloperAudit,
} from "../developer_visuals_helpers.mjs";

function fakeImage({ fail = false } = {}) {
  const listeners = new Map();
  let loading = "lazy";
  const image = {
    complete: false,
    naturalWidth: 0,
    naturalHeight: 0,
    currentSrc: "review-capture.png",
    decoded: false,
    addEventListener(name, callback) {
      const callbacks = listeners.get(name) ?? [];
      callbacks.push(callback);
      listeners.set(name, callbacks);
    },
    async decode() {
      assert.equal(this.complete, true, "decode must follow the image request");
      this.decoded = true;
    },
  };
  Object.defineProperty(image, "loading", {
    get: () => loading,
    set: (value) => {
      loading = value;
      if (value === "eager") {
        queueMicrotask(() => {
          image.complete = true;
          image.naturalWidth = fail ? 0 : 800;
          image.naturalHeight = fail ? 0 : 600;
          for (const callback of listeners.get(fail ? "error" : "load") ?? []) callback();
        });
      }
    },
  });
  return image;
}

test("review image readiness loads hidden lazy captures without changing disclosure or scroll state", async () => {
  const captures = [fakeImage(), fakeImage()];
  const page = {
    scrollY: 0,
    disclosureOpen: false,
    locator(selector) {
      assert.equal(selector, ".review-capture img");
      return {
        count: async () => captures.length,
        nth(index) {
          return {
            evaluate: async (callback) => callback(captures[index]),
          };
        },
      };
    },
  };

  await decodeReviewImages(page);
  assert.equal(page.scrollY, 0);
  assert.equal(page.disclosureOpen, false);
  assert.deepEqual(
    captures.map((capture) => ({ loading: capture.loading, decoded: capture.decoded })),
    [
      { loading: "eager", decoded: true },
      { loading: "eager", decoded: true },
    ],
  );
});

test("review image readiness fails when an explicitly requested capture has no pixels", async () => {
  const capture = fakeImage({ fail: true });
  const page = {
    locator() {
      return {
        count: async () => 1,
        nth() {
          return {
            scrollIntoViewIfNeeded: async () => {},
            evaluate: async (callback) => callback(capture),
          };
        },
      };
    },
  };

  await assert.rejects(decodeReviewImages(page), /Review capture failed to load/);
});

test("audit persistence retains the result array and records incomplete or failed status", async (t) => {
  const output = await mkdtemp(join(tmpdir(), "gomoku-visual-audit-"));
  t.after(() => rm(output, { recursive: true, force: true }));
  const results = [{ engine: "firefox", label: "review", width: 1280, violations: [] }];

  const incomplete = await persistDeveloperAudit({
    output,
    results,
    expectedViews: 66,
    status: "incomplete",
  });
  assert.deepEqual(JSON.parse(await readFile(join(output, "accessibility.json"), "utf8")), results);
  assert.equal(incomplete.status, "incomplete");
  assert.equal(incomplete.complete, false);
  assert.equal(incomplete.completedViews, 1);
  assert.equal(incomplete.expectedViews, 66);

  const failed = await persistDeveloperAudit({
    output,
    results,
    expectedViews: 66,
    status: "failed",
    error: new Error("later audit step failed"),
  });
  assert.equal(failed.status, "failed");
  assert.equal(failed.failure.message, "later audit step failed");
  assert.deepEqual(JSON.parse(await readFile(join(output, "accessibility.json"), "utf8")), results);
});
