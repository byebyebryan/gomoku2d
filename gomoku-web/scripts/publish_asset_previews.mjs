import { copyFile, mkdir, readdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = join(root, "assets");
const distAssetRoot = join(root, "dist", "assets");

async function copyAssetFile(sourcePath, targetPath) {
  await mkdir(dirname(targetPath), { recursive: true });
  await copyFile(sourcePath, targetPath);
}

async function copyTopLevelPreview() {
  const source = join(sourceRoot, "preview.html");
  await copyAssetFile(source, join(distAssetRoot, "index.html"));
  await copyAssetFile(source, join(distAssetRoot, "preview.html"));
  await copyAssetFile(join(sourceRoot, "README.md"), join(distAssetRoot, "README.md"));
}

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

await copyTopLevelPreview();
await copyFolderFiles("sprites", [".html", ".md", ".png"]);
await copyFolderFiles("icons", [".html", ".md", ".json", ".png", ".svg"]);
await copyFolderFiles("fonts", [".html", ".md", ".ttf", ".txt"]);

console.log("Published asset previews to dist/assets/.");
