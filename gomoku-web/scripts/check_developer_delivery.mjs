// Read-only parity check against a deliberately selected production-style preview.
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, relative } from "node:path";
import { readReviewCatalog, webRoot, repoRoot } from "./review_catalog.mjs";
import {
  GENERATED_VISUAL_DESIGN_MANIFEST_PATH,
  VISUAL_DESIGN_ASSET_INVENTORY,
} from "./visual_design_asset_inventory.mjs";
const base = process.env.DEVELOPER_CHECK_URL;
if (!base) throw new Error("Set DEVELOPER_CHECK_URL to an owned preview (trailing slash)");
/** @param {Uint8Array} bytes */
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const dist = resolve(webRoot, "dist"),
  records = await readReviewCatalog();
let checked = 0;
/** @param {string} path @param {Buffer} expected */
async function parity(path, expected) {
  const response = await fetch(new URL(path, base));
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  const actual = Buffer.from(await response.arrayBuffer());
  if (sha(actual) !== sha(expected)) throw new Error(`Served bytes differ: ${path}`);
  checked++;
}
for (const route of [
  "dev/",
  "dev/process/",
  "dev/reviews/",
  "lab/",
  "visuals/",
  ...records.map((record) => `dev/reviews/${record.id}/`),
])
  await parity(route, await readFile(resolve(dist, route, "index.html")));
for (const path of ["bot-report/report.json", "analysis-report/report.json"]) {
  await parity(path, await readFile(resolve(dist, path)));
}
for (const [source, destination] of [
  ["reports/lab/bot-report.json", "bot-report/report.json"],
  ["reports/lab/analysis-report.json", "analysis-report/report.json"],
])
  if (
    sha(await readFile(resolve(repoRoot, source))) !==
    sha(await readFile(resolve(dist, destination)))
  )
    throw new Error("Recorded report changed");
for (const record of records) {
  await parity(
    `reviews/${record.id}/record.json`,
    Buffer.from(JSON.stringify(record, null, 2) + "\n"),
  );
  for (const pair of record.comparisons)
    for (const media of pair.media)
      await parity(
        `reviews/${record.id}/${media.file}`,
        await readFile(resolve(repoRoot, media.sourcePath)),
      );
}
for (const assetPath of VISUAL_DESIGN_ASSET_INVENTORY) {
  const path = `assets/${assetPath}`;
  const expected = await readFile(resolve(dist, path));
  if (assetPath !== GENERATED_VISUAL_DESIGN_MANIFEST_PATH) {
    const source = await readFile(resolve(webRoot, "assets", assetPath));
    if (sha(source) !== sha(expected)) {
      throw new Error(`Built Visual Design bytes differ from declared source: ${assetPath}`);
    }
  }
  await parity(path, expected);
}
console.log(
  JSON.stringify({
    base,
    checked,
    records: records.length,
    recorded_reports_unchanged: true,
    sha256_parity: true,
    local_build: relative(repoRoot, dist),
  }),
);
