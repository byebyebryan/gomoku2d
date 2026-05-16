import type { BotSpec, CorridorProofSpec } from "./bot_protocol";

export type BotPresetId = "easy" | "normal" | "hard";
export type BotDepth = 1 | 3 | 5 | 7;
export type BotWidth = "full" | 8 | 16;
export type BotScoring = "pattern" | "simple";
export type BotExtraPass = "corridor_proof" | "none";

export interface BotPresetConfigV1 {
  mode: "preset";
  preset: BotPresetId;
  version: 1;
}

export interface BotCustomConfigV1 {
  depth: BotDepth;
  extraPass: BotExtraPass;
  mode: "custom";
  scoring: BotScoring;
  version: 1;
  width: BotWidth;
}

export type BotConfigV1 = BotPresetConfigV1 | BotCustomConfigV1;
export type BotConfig = BotConfigV1;

export const DEFAULT_BOT_CONFIG: BotConfig = {
  mode: "preset",
  preset: "normal",
  version: 1,
};

export const CORRIDOR_PROOF_V1: CorridorProofSpec = {
  candidateLimit: 16,
  depth: 8,
  width: 4,
};

const PRESET_CUSTOM_CONFIGS: Record<BotPresetId, BotCustomConfigV1> = {
  easy: {
    depth: 1,
    extraPass: "none",
    mode: "custom",
    scoring: "simple",
    version: 1,
    width: "full",
  },
  hard: {
    depth: 7,
    extraPass: "corridor_proof",
    mode: "custom",
    scoring: "pattern",
    version: 1,
    width: 8,
  },
  normal: {
    depth: 3,
    extraPass: "none",
    mode: "custom",
    scoring: "pattern",
    version: 1,
    width: "full",
  },
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object";
}

function isPresetId(value: unknown): value is BotPresetId {
  return value === "easy" || value === "normal" || value === "hard";
}

function isDepth(value: unknown): value is BotDepth {
  return value === 1 || value === 3 || value === 5 || value === 7;
}

function isWidth(value: unknown): value is BotWidth {
  return value === "full" || value === 8 || value === 16;
}

function isScoring(value: unknown): value is BotScoring {
  return value === "pattern" || value === "simple";
}

function isExtraPass(value: unknown): value is BotExtraPass {
  return value === "corridor_proof" || value === "none";
}

export function isBotWidthAllowed(depth: BotDepth, width: BotWidth): boolean {
  if (depth === 7) {
    return width === 8;
  }

  if (depth === 5) {
    return width !== "full";
  }

  return true;
}

export function clampBotWidth(depth: BotDepth, width: BotWidth): BotWidth {
  if (isBotWidthAllowed(depth, width)) {
    return width;
  }

  return depth === 7 ? 8 : 16;
}

export function isBotConfig(value: unknown): value is BotConfig {
  if (!isRecord(value) || value.version !== 1) {
    return false;
  }

  if (value.mode === "preset") {
    return Object.keys(value).length === 3 && isPresetId(value.preset);
  }

  return (
    value.mode === "custom"
    && Object.keys(value).length === 6
    && isDepth(value.depth)
    && isWidth(value.width)
    && isScoring(value.scoring)
    && isExtraPass(value.extraPass)
  );
}

export function sanitizeBotConfig(value: unknown): BotConfig {
  if (!isRecord(value) || value.version !== 1) {
    return DEFAULT_BOT_CONFIG;
  }

  if (value.mode === "preset" && isPresetId(value.preset)) {
    return {
      mode: "preset",
      preset: value.preset,
      version: 1,
    };
  }

  if (
    value.mode === "custom"
    && isDepth(value.depth)
    && isWidth(value.width)
    && isScoring(value.scoring)
    && isExtraPass(value.extraPass)
  ) {
    return {
      depth: value.depth,
      extraPass: value.extraPass,
      mode: "custom",
      scoring: value.scoring,
      version: 1,
      width: clampBotWidth(value.depth, value.width),
    };
  }

  return DEFAULT_BOT_CONFIG;
}

export function customConfigForBot(config: BotConfig): BotCustomConfigV1 {
  return config.mode === "preset" ? PRESET_CUSTOM_CONFIGS[config.preset] : config;
}

export function resolveBotConfig(config: BotConfig): BotSpec {
  const custom = customConfigForBot(config);
  return {
    childLimit: custom.width === "full" ? null : custom.width,
    corridorProof: custom.extraPass === "corridor_proof" ? CORRIDOR_PROOF_V1 : null,
    depth: custom.depth,
    kind: "search",
    patternEval: custom.scoring === "pattern",
  };
}

export function labSpecForBot(config: BotConfig): string {
  const custom = customConfigForBot(config);
  const suffixes = [
    custom.width === "full" ? null : `tactical-cap-${custom.width}`,
    custom.scoring === "pattern" ? "pattern-eval" : null,
    custom.extraPass === "corridor_proof" ? "corridor-proof-c16-d8-w4" : null,
  ].filter(Boolean);

  return [`search-d${custom.depth}`, ...suffixes].join("+");
}

export function botConfigSummary(config: BotConfig): string {
  const custom = customConfigForBot(config);
  return [
    `D${custom.depth}`,
    custom.width === "full" ? "full" : `W${custom.width}`,
    custom.scoring,
    custom.extraPass === "corridor_proof" ? "proof" : null,
  ].filter(Boolean).join(" · ");
}

export function botLabel(config: BotConfig): string {
  if (config.mode === "preset") {
    switch (config.preset) {
      case "easy":
        return "Easy";
      case "hard":
        return "Hard";
      case "normal":
        return "Normal";
    }
  }

  return "Custom";
}

export function botPlayerName(config: BotConfig): string {
  return `${botLabel(sanitizeBotConfig(config))} Bot`;
}
