import type { GameVariant } from "../core/bot_protocol";
import {
  DEFAULT_BOT_CONFIG,
  sanitizeBotConfig,
  type BotConfig,
} from "../core/bot_config";

export const DEFAULT_RULE_OPENING = "standard";

export type TouchControlMode = "pointer" | "touchpad";
export type ImmediateHintMode = "off" | "win" | "win_threat";
export type ImminentHintMode = "off" | "threat" | "threat_counter";

export interface BoardHintSettings {
  immediate: ImmediateHintMode;
  imminent: ImminentHintMode;
}

export interface GameConfig {
  opening: typeof DEFAULT_RULE_OPENING;
  ruleset: GameVariant;
}

export interface ProfileSettings {
  boardHints: BoardHintSettings;
  botConfig: BotConfig;
  gameConfig: GameConfig;
  touchControl: TouchControlMode;
}

export const DEFAULT_BOARD_HINTS: BoardHintSettings = {
  immediate: "win_threat",
  imminent: "threat_counter",
};

export function createDefaultBoardHints(): BoardHintSettings {
  return { ...DEFAULT_BOARD_HINTS };
}

export function createDefaultGameConfig(): GameConfig {
  return {
    opening: DEFAULT_RULE_OPENING,
    ruleset: "freestyle",
  };
}

export function createDefaultProfileSettings(): ProfileSettings {
  return {
    boardHints: createDefaultBoardHints(),
    botConfig: DEFAULT_BOT_CONFIG,
    gameConfig: createDefaultGameConfig(),
    touchControl: "touchpad",
  };
}

export function sanitizeTouchControl(value: unknown): TouchControlMode {
  return value === "pointer" ? "pointer" : "touchpad";
}

export function sanitizeImmediateHintMode(value: unknown): ImmediateHintMode {
  return value === "off" || value === "win" || value === "win_threat"
    ? value
    : DEFAULT_BOARD_HINTS.immediate;
}

export function sanitizeImminentHintMode(value: unknown): ImminentHintMode {
  return value === "off" || value === "threat" || value === "threat_counter"
    ? value
    : DEFAULT_BOARD_HINTS.imminent;
}

export function sanitizeBoardHints(value: unknown): BoardHintSettings {
  const candidate = value as Partial<Record<keyof BoardHintSettings, unknown>> | null;

  return {
    immediate: sanitizeImmediateHintMode(candidate?.immediate),
    imminent: sanitizeImminentHintMode(candidate?.imminent),
  };
}

export function sanitizeGameConfig(value: unknown): GameConfig {
  const candidate = value as Partial<Record<keyof GameConfig, unknown>> | null;

  return {
    opening: DEFAULT_RULE_OPENING,
    ruleset: candidate?.ruleset === "renju" ? "renju" : "freestyle",
  };
}

export function sanitizeProfileSettings(value: unknown): ProfileSettings {
  const candidate = value as Partial<Record<keyof ProfileSettings, unknown>> | null;

  return {
    boardHints: sanitizeBoardHints(candidate?.boardHints),
    botConfig: sanitizeBotConfig(candidate?.botConfig),
    gameConfig: sanitizeGameConfig(candidate?.gameConfig),
    touchControl: sanitizeTouchControl(candidate?.touchControl),
  };
}

export function profileSettingsEqual(left: ProfileSettings, right: ProfileSettings): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}
