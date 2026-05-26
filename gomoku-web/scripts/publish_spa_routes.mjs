import { copyFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const distRoot = join(root, "dist");
const indexPath = join(distRoot, "index.html");

const staticRoutes = [
  "guide",
  "lab",
  "match/local",
  "profile",
  "rules",
  "settings",
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

await Promise.all([
  copyFile(indexPath, join(distRoot, "404.html")),
  ...staticRoutes.map(copyIndexToRoute),
]);

console.log(`Published SPA route entries for ${staticRoutes.map((route) => `/${route}`).join(", ")}.`);
