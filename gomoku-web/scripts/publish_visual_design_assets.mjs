import { copyFile, mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = join(root, "assets");
const distAssetRoot = join(resolve(root, process.argv[2] ?? "dist"), "assets");

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

console.log("Published visual design asset source files to dist/assets/.");

// Roles are editorial; published palette values follow runtime CSS tokens.
const css=await readFile(join(root,"src/app/global.css"),"utf8");
const tokens=Object.fromEntries([...css.matchAll(/(--[\w-]+):\s*([^;]+);/g)].map(match=>[match[1],match[2].trim()]));
const manifest=JSON.parse(await readFile(join(sourceRoot,"asset_manifest.json"),"utf8"));
for(const group of manifest.guide.palette)for(const token of group.tokens){if(!tokens[token.name])throw new Error(`Missing runtime token ${token.name}`);token.value=tokens[token.name];}
await writeFile(join(distAssetRoot,"asset_manifest.json"),JSON.stringify(manifest,null,2)+"\n");
