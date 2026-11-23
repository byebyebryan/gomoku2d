import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  validateVisualDesignAssetInventory,
  validateRelativeVisualAssetPath,
  GENERATED_VISUAL_DESIGN_MANIFEST_PATH,
  VISUAL_DESIGN_ASSET_INVENTORY,
} from "./visual_design_asset_inventory.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const sourceRoot = join(root, "assets");
const distAssetRoot = join(resolve(root, process.argv[2] ?? "dist"), "assets");

const manifest = JSON.parse(
  await readFile(join(sourceRoot, GENERATED_VISUAL_DESIGN_MANIFEST_PATH), "utf8"),
);
const iconManifestPath = validateRelativeVisualAssetPath(
  manifest.icons?.manifest,
  "icons.manifest",
);
if (!VISUAL_DESIGN_ASSET_INVENTORY.includes(iconManifestPath)) {
  throw new Error(`Referenced Visual Design asset is not declared for publication: ${iconManifestPath}`);
}
const iconManifest = JSON.parse(
  await readFile(join(sourceRoot, iconManifestPath), "utf8"),
);
await validateVisualDesignAssetInventory({
  assetRoot: sourceRoot,
  assetManifest: manifest,
  iconManifest,
});

// Roles are editorial; published palette values follow runtime CSS tokens.
const css = await readFile(join(root, "src/app/global.css"), "utf8");
const tokens = Object.fromEntries(
  [...css.matchAll(/(--[\w-]+):\s*([^;]+);/g)].map((match) => [match[1], match[2].trim()]),
);
for (const group of manifest.guide.palette) {
  for (const token of group.tokens) {
    if (!tokens[token.name]) throw new Error(`Missing runtime token ${token.name}`);
    token.value = tokens[token.name];
  }
}

for (const assetPath of VISUAL_DESIGN_ASSET_INVENTORY) {
  if (assetPath === GENERATED_VISUAL_DESIGN_MANIFEST_PATH) continue;
  const destination = join(distAssetRoot, assetPath);
  await mkdir(dirname(destination), { recursive: true });
  await copyFile(join(sourceRoot, assetPath), destination);
}
await mkdir(distAssetRoot, { recursive: true });
await writeFile(
  join(distAssetRoot, GENERATED_VISUAL_DESIGN_MANIFEST_PATH),
  JSON.stringify(manifest, null, 2) + "\n",
);

console.log(
  `Published ${VISUAL_DESIGN_ASSET_INVENTORY.length} declared Visual Design assets to dist/assets/.`,
);
