import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import {
  historyRepoRoot,
  historyWebRoot,
  readHistoryCatalog,
} from "./history_catalog.mjs";

/** @param {string} [destination] */
export async function publishHistory(
  destination = resolve(historyWebRoot, "dist"),
) {
  const catalog = await readHistoryCatalog();
  const target = resolve(destination, "history");
  await mkdir(resolve(target, "manifests"), { recursive: true });
  await writeFile(
    resolve(target, "catalog.json"),
    JSON.stringify(catalog, null, 2) + "\n",
  );
  const copied = new Set();
  for (const snapshot of catalog.snapshots) {
    if (!copied.has(snapshot.manifest.sha256)) {
      await copyFile(
        resolve(historyRepoRoot, snapshot.manifest.sourcePath),
        resolve(target, "manifests", snapshot.manifest.sha256 + ".json"),
      );
      copied.add(snapshot.manifest.sha256);
    }
    for (const media of Object.values(snapshot.media)) {
      if (!media || copied.has(media.file)) continue;
      await copyFile(
        resolve(historyRepoRoot, media.sourcePath),
        resolve(target, media.file),
      );
      copied.add(media.file);
    }
  }
  console.log(
    `Published ${catalog.milestones.length} UI milestones and ${catalog.snapshots.length} qualified snapshots.`,
  );
}
if (process.argv[1] === new URL(import.meta.url).pathname)
  await publishHistory(process.argv[2]);
