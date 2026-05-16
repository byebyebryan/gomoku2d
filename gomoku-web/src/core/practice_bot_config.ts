import type { BotSpec, CorridorProofSpec } from "./bot_protocol";

export type PracticeBotPresetId = "easy" | "normal" | "hard";
export type PracticeBotDepth = 1 | 3 | 5 | 7;
export type PracticeBotWidth = "none" | 8 | 16;

export interface PracticeBotPresetConfigV1 {
  mode: "preset";
  preset: PracticeBotPresetId;
  version: 1;
}

export interface PracticeBotCustomConfigV1 {
  corridorProof: boolean;
  depth: PracticeBotDepth;
  mode: "custom";
  patternScoring: boolean;
  version: 1;
  width: PracticeBotWidth;
}

export type PracticeBotConfigV1 = PracticeBotPresetConfigV1 | PracticeBotCustomConfigV1;
export type PracticeBotConfig = PracticeBotConfigV1;

export const DEFAULT_PRACTICE_BOT_CONFIG: PracticeBotConfig = {
  mode: "preset",
  preset: "normal",
  version: 1,
};

export const CORRIDOR_PROOF_V1: CorridorProofSpec = {
  candidateLimit: 16,
  depth: 8,
  width: 4,
};

const PRESET_CUSTOM_CONFIGS: Record<PracticeBotPresetId, PracticeBotCustomConfigV1> = {
  easy: {
    corridorProof: false,
    depth: 1,
    mode: "custom",
    patternScoring: false,
    version: 1,
    width: "none",
  },
  hard: {
    corridorProof: true,
    depth: 7,
    mode: "custom",
    patternScoring: true,
    version: 1,
    width: 8,
  },
  normal: {
    corridorProof: false,
    depth: 3,
    mode: "custom",
    patternScoring: true,
    version: 1,
    width: "none",
  },
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value) && typeof value === "object";
}

function isPresetId(value: unknown): value is PracticeBotPresetId {
  return value === "easy" || value === "normal" || value === "hard";
}

function isDepth(value: unknown): value is PracticeBotDepth {
  return value === 1 || value === 3 || value === 5 || value === 7;
}

function isWidth(value: unknown): value is PracticeBotWidth {
  return value === "none" || value === 8 || value === 16;
}

export function isPracticeBotWidthAllowed(depth: PracticeBotDepth, width: PracticeBotWidth): boolean {
  if (depth === 7) {
    return width === 8;
  }

  if (depth === 5) {
    return width !== "none";
  }

  return true;
}

export function clampPracticeBotWidth(depth: PracticeBotDepth, width: PracticeBotWidth): PracticeBotWidth {
  if (isPracticeBotWidthAllowed(depth, width)) {
    return width;
  }

  return depth === 7 ? 8 : 16;
}

export function isPracticeBotConfig(value: unknown): value is PracticeBotConfig {
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
    && typeof value.patternScoring === "boolean"
    && typeof value.corridorProof === "boolean"
  );
}

export function sanitizePracticeBotConfig(value: unknown): PracticeBotConfig {
  if (!isRecord(value) || value.version !== 1) {
    return DEFAULT_PRACTICE_BOT_CONFIG;
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
    && typeof value.patternScoring === "boolean"
    && typeof value.corridorProof === "boolean"
  ) {
    return {
      corridorProof: value.corridorProof,
      depth: value.depth,
      mode: "custom",
      patternScoring: value.patternScoring,
      version: 1,
      width: clampPracticeBotWidth(value.depth, value.width),
    };
  }

  return DEFAULT_PRACTICE_BOT_CONFIG;
}

export function customConfigForPracticeBot(config: PracticeBotConfig): PracticeBotCustomConfigV1 {
  return config.mode === "preset" ? PRESET_CUSTOM_CONFIGS[config.preset] : config;
}

export function resolvePracticeBotConfig(config: PracticeBotConfig): BotSpec {
  const custom = customConfigForPracticeBot(config);
  return {
    childLimit: custom.width === "none" ? null : custom.width,
    corridorProof: custom.corridorProof ? CORRIDOR_PROOF_V1 : null,
    depth: custom.depth,
    kind: "search",
    patternEval: custom.patternScoring,
  };
}

export function labSpecForPracticeBot(config: PracticeBotConfig): string {
  const custom = customConfigForPracticeBot(config);
  const suffixes = [
    custom.width === "none" ? null : `tactical-cap-${custom.width}`,
    custom.patternScoring ? "pattern-eval" : null,
    custom.corridorProof ? "corridor-proof-c16-d8-w4" : null,
  ].filter(Boolean);

  return [`search-d${custom.depth}`, ...suffixes].join("+");
}

export function practiceBotConfigSummary(config: PracticeBotConfig): string {
  const custom = customConfigForPracticeBot(config);
  return [
    `D${custom.depth}`,
    custom.width === "none" ? "full" : `W${custom.width}`,
    custom.patternScoring ? "pattern" : "simple",
    custom.corridorProof ? "proof" : null,
  ].filter(Boolean).join(" · ");
}

export function practiceBotLabel(config: PracticeBotConfig): string {
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

export function practiceBotPlayerName(config: PracticeBotConfig): string {
  return `${practiceBotLabel(sanitizePracticeBotConfig(config))} Bot`;
}
