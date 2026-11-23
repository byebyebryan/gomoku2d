import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { test } from "node:test";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  GENERATED_VISUAL_DESIGN_MANIFEST_PATH,
  validateVisualDesignAssetInventory,
  VISUAL_DESIGN_ASSET_INVENTORY,
} from "../visual_design_asset_inventory.mjs";

const assetRoot = fileURLToPath(new URL("../../assets", import.meta.url));

test("the declared inventory covers Visuals manifest references and retains legacy URLs", async () => {
  const assetManifest = JSON.parse(await readFile(join(assetRoot, "asset_manifest.json"), "utf8"));
  const iconManifest = JSON.parse(
    await readFile(join(assetRoot, assetManifest.icons.manifest), "utf8"),
  );
  const referenced = await validateVisualDesignAssetInventory({
    assetRoot,
    assetManifest,
    iconManifest,
  });
  const declaredPaths = new Set(VISUAL_DESIGN_ASSET_INVENTORY);

  assert([...referenced].every((path) => declaredPaths.has(path)));
  assert(declaredPaths.has(GENERATED_VISUAL_DESIGN_MANIFEST_PATH));
  assert(declaredPaths.has("og_source.png"));
  assert(declaredPaths.has("fonts/LICENSE.txt"));
  for (const role of assetManifest.guide.font_roles) assert(declaredPaths.has(role.file));
});

test("a missing declared source referenced by the manifest fails validation", async (t) => {
  const fixtureRoot = await mkdtemp(join(tmpdir(), "gomoku-visual-inventory-fixture-"));
  t.after(() => rm(fixtureRoot, { recursive: true, force: true }));

  const assetManifest = {
    guide: { font_roles: [{ file: "fonts/missing.ttf" }] },
    sprites: { sheets: [], static_poses: [], animations: [], z_cases: [] },
    icons: { manifest: "icons/manifest.json", directory: "icons" },
  };
  const iconManifest = { source_sheet: "icon_sheet.png", icons: [{ name: "home" }] };
  const inventory = [
    "asset_manifest.json",
    "fonts/LICENSE.txt",
    "fonts/missing.ttf",
    "icons/manifest.json",
    "icons/icon_sheet.png",
    "icons/home.svg",
  ];

  for (const path of inventory.filter((assetPath) => assetPath !== "fonts/missing.ttf")) {
    const filePath = join(fixtureRoot, ...path.split("/"));
    await mkdir(dirname(filePath), { recursive: true });
    await writeFile(filePath, "fixture");
  }

  await assert.rejects(
    validateVisualDesignAssetInventory({
      assetRoot: fixtureRoot,
      assetManifest,
      iconManifest,
      inventory,
    }),
    /Missing declared Visual Design source asset "fonts\/missing\.ttf"/,
  );
  await assert.rejects(
    validateVisualDesignAssetInventory({
      assetRoot: fixtureRoot,
      assetManifest,
      iconManifest,
      inventory: inventory.filter((path) => path !== "fonts/missing.ttf"),
    }),
    /Referenced Visual Design asset is not declared for publication: fonts\/missing\.ttf/,
  );
});
