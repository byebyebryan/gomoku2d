export interface AssetSpriteSheet {
  file: string;
  cols: number;
  rows: number;
  label: string;
}

export interface AssetSpriteFrame {
  group: string;
  name: string;
  role: string;
  file: string;
  cols: number;
  rows: number;
  tint?: string;
}

export interface AssetSpriteStaticPose extends AssetSpriteFrame {
  frame: number;
}

export interface AssetSpriteAnimation extends AssetSpriteFrame {
  start: number;
  end: number;
  fps: number;
}

export interface AssetSpriteZLayer extends Partial<AssetSpriteAnimation> {
  z: string;
  name: string;
  file: string;
  cols: number;
  rows: number;
  frame?: number;
  tint?: string;
}

export interface AssetSpriteZCase {
  name: string;
  title: string;
  note: string;
  sequence?: string;
  layers: AssetSpriteZLayer[];
}

export interface AssetManifest {
  schema_version: 1;
  title: string;
  summary: string;
  sprites: {
    frame_size: number;
    sheets: AssetSpriteSheet[];
    static_poses: AssetSpriteStaticPose[];
    animations: AssetSpriteAnimation[];
    z_cases: AssetSpriteZCase[];
  };
  icons: {
    manifest: string;
    directory: string;
  };
}

export interface IconManifest {
  source_sheet: string;
  cell_size: number;
  columns: number;
  rows: number;
  icons: Array<{
    name: string;
    row: number;
    col: number;
    category: string;
    label: string;
    note: string;
  }>;
}

const ASSET_MANIFEST_URL = `${import.meta.env.BASE_URL}assets/asset_manifest.json`;

export async function loadAssetManifest(): Promise<AssetManifest> {
  const response = await fetch(ASSET_MANIFEST_URL, { cache: "no-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load asset manifest (${response.status})`);
  }

  const data = (await response.json()) as unknown;
  if (!isAssetManifest(data)) {
    throw new Error("Asset manifest has an unsupported schema.");
  }
  return data;
}

export async function loadIconManifest(path: string): Promise<IconManifest> {
  const response = await fetch(`${import.meta.env.BASE_URL}assets/${path}`, { cache: "no-cache" });
  if (!response.ok) {
    throw new Error(`Failed to load icon manifest (${response.status})`);
  }

  const data = (await response.json()) as unknown;
  if (!isIconManifest(data)) {
    throw new Error("Icon manifest has an unsupported schema.");
  }
  return data;
}

function isAssetManifest(data: unknown): data is AssetManifest {
  if (!data || typeof data !== "object") {
    return false;
  }
  const manifest = data as Partial<AssetManifest>;
  return (
    manifest.schema_version === 1 &&
    typeof manifest.title === "string" &&
    !!manifest.sprites &&
    Array.isArray(manifest.sprites.sheets) &&
    Array.isArray(manifest.sprites.static_poses) &&
    Array.isArray(manifest.sprites.animations) &&
    Array.isArray(manifest.sprites.z_cases) &&
    !!manifest.icons &&
    typeof manifest.icons.manifest === "string"
  );
}

function isIconManifest(data: unknown): data is IconManifest {
  if (!data || typeof data !== "object") {
    return false;
  }
  const manifest = data as Partial<IconManifest>;
  return (
    typeof manifest.source_sheet === "string" &&
    typeof manifest.cell_size === "number" &&
    typeof manifest.columns === "number" &&
    typeof manifest.rows === "number" &&
    Array.isArray(manifest.icons)
  );
}
