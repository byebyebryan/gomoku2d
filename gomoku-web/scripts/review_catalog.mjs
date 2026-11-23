import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { readRepositoryPngIdentity } from "./catalog_media.mjs";
const { isReviewRecord } = await import(
  new URL("../src/developer/review_record.ts", import.meta.url).href
);
export const webRoot = resolve(import.meta.dirname, "..");
export const repoRoot = resolve(webRoot, "..");
export async function readReviewCatalog() {
  const records = JSON.parse(
    await readFile(resolve(webRoot, "src/developer/reviews.json"), "utf8"),
  );
  if (
    !Array.isArray(records) ||
    !records.every(isReviewRecord) ||
    new Set(records.map((record) => record.id)).size !== records.length
  )
    throw new Error("Invalid curated review catalog");
  for (const record of records) {
    const seen = new Map();
    for (const pair of record.comparisons)
      for (const media of pair.media) {
        await readRepositoryPngIdentity(repoRoot, media, {
          outsideRepository: "Review media leaves repository",
          identityMismatch: `Review media identity mismatch: ${media.sourcePath}`,
        });
        if (seen.has(media.file) && seen.get(media.file) !== media.sha256)
          throw new Error("Conflicting review file names");
        seen.set(media.file, media.sha256);
      }
  }
  return records;
}
