import { copyFile, mkdir, readdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = join(root, "assets");
const distAssetRoot = join(root, "dist", "assets");

/**
 * @param {string} sourcePath
 * @param {string} targetPath
 */
async function copyAssetFile(sourcePath, targetPath) {
  await mkdir(dirname(targetPath), { recursive: true });
  await copyFile(sourcePath, targetPath);
}

async function copyTopLevelAssets() {
  await copyFolderFiles(".", [".json", ".md", ".png"]);
}

/**
 * @param {string} folder
 * @param {readonly string[]} allowedExtensions
 */
async function copyFolderFiles(folder, allowedExtensions) {
  const sourceDir = join(sourceRoot, folder);
  const targetDir = join(distAssetRoot, folder);
  await mkdir(targetDir, { recursive: true });

  const entries = await readdir(sourceDir, { withFileTypes: true });
  await Promise.all(
    entries
      .filter((entry) => entry.isFile())
      .filter((entry) => allowedExtensions.some((extension) => entry.name.endsWith(extension)))
      .map((entry) => copyAssetFile(join(sourceDir, entry.name), join(targetDir, entry.name))),
  );
}

await copyTopLevelAssets();
await copyFolderFiles("sprites", [".md", ".png"]);
await copyFolderFiles("icons", [".md", ".json", ".png", ".svg"]);
await copyFolderFiles("fonts", [".md", ".ttf", ".txt"]);

console.log("Published asset source files to dist/assets/.");
