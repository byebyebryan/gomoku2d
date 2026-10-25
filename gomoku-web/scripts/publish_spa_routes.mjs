import { copyFile, mkdir } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { readReviewCatalog } from "./review_catalog.mjs";
import { readHistoryCatalog } from "./history_catalog.mjs";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const distRoot = resolve(root, process.argv[2] ?? "dist");
const indexPath = join(distRoot, "index.html");

const staticRoutes = [
  "dev",
  "dev/reviews",
  "dev/process",
  "guide",
  "lab",
  "match/local",
  "privacy",
  "profile",
  "player",
  "replay/shared",
  "rules",
  "settings",
  "terms",
  "visuals",
];

/**
 * @param {string} route
 */
async function copyIndexToRoute(route) {
  const targetPath = join(distRoot, route, "index.html");
  await mkdir(dirname(targetPath), { recursive: true });
  await copyFile(indexPath, targetPath);
}

const reviews = await readReviewCatalog();
staticRoutes.push(...reviews.map((record) => `dev/reviews/${record.id}`));
const history = await readHistoryCatalog();
staticRoutes.push(
  "dev/reviews/history",
  ...history.screens.map((screen) => `dev/reviews/history/${screen.id}`),
);

await Promise.all([
  copyFile(indexPath, join(distRoot, "404.html")),
  ...staticRoutes.map(copyIndexToRoute),
]);

console.log(
  `Published SPA route entries for ${staticRoutes.map((route) => `/${route}`).join(", ")}.`,
);
