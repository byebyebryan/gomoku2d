import { build } from "vite";
import { copyFile, mkdir, readFile, rename, writeFile } from "node:fs/promises";
import { resolve, relative } from "node:path";
import { readReviewCatalog, webRoot, repoRoot } from "./review_catalog.mjs";
const [id, directory] = process.argv.slice(2);
if (!id || !directory)
  throw new Error("Usage: node scripts/export_review.mjs <curated-id> <new-export-directory>");
const records = await readReviewCatalog(),
  record = records.find((item) => item.id === id);
if (!record) throw new Error(`Unknown curated review ${id}`);
const output = resolve(directory);
// Export only to the dedicated ignored output area, never overwrite source/dist.
if (!relative(resolve(webRoot, "test-results/review-exports"), output).match(/^[^.][^]*$/))
  throw new Error("Use a new directory under gomoku-web/test-results/review-exports/");
await mkdir(resolve(webRoot, "test-results/review-exports"), { recursive: true });
await mkdir(output, { recursive: false });
await build({
  root: webRoot,
  publicDir: false,
  base: "./",
  build: {
    outDir: output,
    emptyOutDir: false,
    rolldownOptions: { input: resolve(webRoot, "review.html") },
  },
});
const html = await readFile(resolve(output, "review.html"), "utf8");
if (!html.includes('"__REVIEW_RECORD__"')) throw new Error("Review export marker missing");
await writeFile(
  resolve(output, "review.html"),
  html.replace('"__REVIEW_RECORD__"', JSON.stringify(record).replaceAll("<", "\\u003c")),
);
await rename(resolve(output, "review.html"), resolve(output, "index.html"));
await mkdir(resolve(output, "media"));
for (const pair of record.comparisons)
  for (const media of pair.media)
    await copyFile(resolve(repoRoot, media.sourcePath), resolve(output, "media", media.file));
await copyFile(resolve(webRoot, "assets/fonts/LICENSE.txt"), resolve(output, "LICENSE.txt"));
await writeFile(resolve(output, "record.json"), JSON.stringify(record, null, 2) + "\n");
console.log(`Prepared ${id} using the same ReviewCase component and record as the site: ${output}`);
