import { stat } from "node:fs/promises";
import { join } from "node:path";

/** Generated from asset_manifest.json after runtime palette token hydration. */
export const GENERATED_VISUAL_DESIGN_MANIFEST_PATH = "asset_manifest.json";

/** Explicit public Visual Design URLs; unrelated files are not published. */
export const VISUAL_DESIGN_ASSET_INVENTORY = Object.freeze([
  "README.md",
  GENERATED_VISUAL_DESIGN_MANIFEST_PATH,
  "favicon.png",
  // Legacy /assets/og_source.png URL; the social image is separate at /og.png.
  "og_source.png",
  "fonts/LICENSE.txt",
  "fonts/README.md",
  // The three font files linked by the Visual Design runtime manifest.
  "fonts/PixelOperator.ttf",
  "fonts/PixelOperator-Bold.ttf",
  "fonts/PixelOperator8-Bold.ttf",
  // Legacy font variants retained for older Visual Design source URLs.
  "fonts/PixelOperator8.ttf",
  "fonts/PixelOperatorHB.ttf",
  "fonts/PixelOperatorHB8.ttf",
  "fonts/PixelOperatorHBSC.ttf",
  "fonts/PixelOperatorMono-Bold.ttf",
  "fonts/PixelOperatorMono.ttf",
  "fonts/PixelOperatorMono8-Bold.ttf",
  "fonts/PixelOperatorMono8.ttf",
  "fonts/PixelOperatorMonoHB.ttf",
  "fonts/PixelOperatorMonoHB8.ttf",
  "fonts/PixelOperatorSC-Bold.ttf",
  "fonts/PixelOperatorSC.ttf",
  "icons/README.md",
  "icons/manifest.json",
  "icons/icon_sheet.png",
  "icons/back.svg",
  "icons/bot.svg",
  "icons/circle.svg",
  "icons/close.svg",
  "icons/confirm.svg",
  "icons/copy.svg",
  "icons/double_next.svg",
  "icons/double_prev.svg",
  "icons/exit.svg",
  "icons/fast_forward.svg",
  "icons/fast_rewind.svg",
  "icons/filled_circle.svg",
  "icons/first.svg",
  "icons/forbidden.svg",
  "icons/forward.svg",
  "icons/grid.svg",
  "icons/help.svg",
  "icons/home.svg",
  "icons/human.svg",
  "icons/info.svg",
  "icons/last.svg",
  "icons/minus.svg",
  "icons/next.svg",
  "icons/pause.svg",
  "icons/play.svg",
  "icons/play_reverse.svg",
  "icons/plus.svg",
  "icons/prev.svg",
  "icons/profile.svg",
  "icons/publish.svg",
  "icons/replay.svg",
  "icons/reset.svg",
  "icons/save.svg",
  "icons/settings.svg",
  "icons/share.svg",
  "icons/sign_in.svg",
  "icons/sign_out.svg",
  "icons/sync.svg",
  "icons/trash.svg",
  "icons/undo.svg",
  "icons/win.svg",
  "sprites/README.md",
  "sprites/caution.png",
  "sprites/highlighter.png",
  "sprites/hover.png",
  "sprites/marker.png",
  "sprites/pointer.png",
  "sprites/stone.png",
  "sprites/transform.png",
]);

/**
 * @param {unknown} value
 * @param {string} label
 * @returns {string}
 */
export function validateRelativeVisualAssetPath(value, label) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.startsWith("/") ||
    value.includes("\\") ||
    value.split("/").some((part) => part === "" || part === "." || part === "..")
  ) {
    throw new Error(`Invalid relative Visual Design asset path in ${label}`);
  }
  return value;
}

/** @param {unknown} value @param {string} label @returns {any[]} */
function array(value, label) {
  if (!Array.isArray(value)) throw new Error(`Expected ${label} in Visual Design manifest`);
  return value;
}

/**
 * Returns the raw files referenced by the fields consumed by the Visuals route.
 * It deliberately ignores labels, notes and other prose fields in the manifests.
 *
 * @param {any} assetManifest
 * @param {any} iconManifest
 * @returns {Set<string>}
 */
export function referencedVisualDesignAssetPaths(assetManifest, iconManifest) {
  const paths = new Set([GENERATED_VISUAL_DESIGN_MANIFEST_PATH, "fonts/LICENSE.txt"]);
  /** @param {unknown} value @param {string} label */
  const add = (value, label) => paths.add(validateRelativeVisualAssetPath(value, label));

  const fontRoles = array(assetManifest?.guide?.font_roles, "guide.font_roles");
  for (const [index, role] of fontRoles.entries()) add(role?.file, `guide.font_roles[${index}].file`);

  const sprites = assetManifest?.sprites;
  for (const field of ["sheets", "static_poses", "animations"]) {
    for (const [index, item] of array(sprites?.[field], `sprites.${field}`).entries()) {
      add(item?.file, `sprites.${field}[${index}].file`);
    }
  }
  for (const [caseIndex, item] of array(sprites?.z_cases, "sprites.z_cases").entries()) {
    for (const [layerIndex, layer] of array(item?.layers, `sprites.z_cases[${caseIndex}].layers`).entries()) {
      add(layer?.file, `sprites.z_cases[${caseIndex}].layers[${layerIndex}].file`);
    }
  }

  const iconDirectory = validateRelativeVisualAssetPath(assetManifest?.icons?.directory, "icons.directory");
  add(assetManifest?.icons?.manifest, "icons.manifest");
  const sourceSheet = validateRelativeVisualAssetPath(iconManifest?.source_sheet, "icons.manifest.source_sheet");
  add(`${iconDirectory}/${sourceSheet}`, "icons.manifest.source_sheet");
  for (const [index, icon] of array(iconManifest?.icons, "icons.manifest.icons").entries()) {
    if (typeof icon?.name !== "string" || !/^[A-Za-z0-9_-]+$/.test(icon.name)) {
      throw new Error(`Invalid icon name in icons.manifest.icons[${index}]`);
    }
    add(`${iconDirectory}/${icon.name}.svg`, `icons.manifest.icons[${index}].name`);
  }
  return paths;
}

/**
 * Confirms that every consumed path has a declaration and every declared path
 * has a source file. The generated manifest is included as a source input.
 *
 * @param {{assetRoot: string, assetManifest: any, iconManifest: any, inventory?: readonly string[]}} options
 * @returns {Promise<Set<string>>}
 */
export async function validateVisualDesignAssetInventory({
  assetRoot,
  assetManifest,
  iconManifest,
  inventory = VISUAL_DESIGN_ASSET_INVENTORY,
}) {
  const declared = new Set();
  for (const item of inventory) {
    const path = validateRelativeVisualAssetPath(item, "publication inventory path");
    if (declared.has(path)) throw new Error(`Duplicate Visual Design publication path: ${path}`);
    declared.add(path);
  }

  const referenced = referencedVisualDesignAssetPaths(assetManifest, iconManifest);
  for (const path of referenced) {
    if (!declared.has(path)) {
      throw new Error(`Referenced Visual Design asset is not declared for publication: ${path}`);
    }
  }

  for (const path of declared) {
    try {
      if (!(await stat(join(assetRoot, ...path.split("/")))).isFile()) {
        throw new Error("not a regular file");
      }
    } catch {
      throw new Error(
        `Missing declared Visual Design source asset "${path}"`,
      );
    }
  }
  return referenced;
}
