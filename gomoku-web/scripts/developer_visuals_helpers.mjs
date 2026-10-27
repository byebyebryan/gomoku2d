import { writeFile } from "node:fs/promises";
import { resolve } from "node:path";

/** @param {import("@playwright/test").Page} page */
export async function decodeReviewImages(page) {
  const images = page.locator(".review-capture img");
  const count = await images.count();
  for (let index = 0; index < count; index += 1) {
    const image = images.nth(index);
    await image.evaluate(async (node) => {
      const capture = /** @type {HTMLImageElement} */ (node);
      capture.loading = "eager";
      if (!capture.complete) {
        await new Promise((resolve, reject) => {
          capture.addEventListener("load", resolve, { once: true });
          capture.addEventListener(
            "error",
            () => reject(new Error(`Review capture failed to load: ${capture.currentSrc}`)),
            { once: true },
          );
        });
      }
      await capture.decode();
      if (!capture.naturalWidth || !capture.naturalHeight)
        throw new Error(`Review capture has no decoded dimensions: ${capture.currentSrc}`);
    });
  }
}

/**
 * @param {{ output: string; results: Array<Record<string, unknown>>; expectedViews: number; status: "incomplete" | "failed" | "complete"; error?: unknown }} options
 */
export async function persistDeveloperAudit({
  output,
  results,
  expectedViews,
  status,
  error,
}) {
  await writeFile(
    resolve(output, "accessibility.json"),
    JSON.stringify(results, null, 2) + "\n",
  );
  const receipt = {
    schema: 1,
    status,
    complete: status === "complete",
    completedViews: results.length,
    expectedViews,
    ...(error === undefined
      ? {}
      : {
          failure: {
            name: error instanceof Error ? error.name : "Error",
            message: error instanceof Error ? error.message : String(error),
          },
        }),
  };
  await writeFile(
    resolve(output, "accessibility-status.json"),
    JSON.stringify(receipt, null, 2) + "\n",
  );
  return receipt;
}
