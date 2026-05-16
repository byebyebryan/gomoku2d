import { afterEach, describe, expect, it } from "vitest";

import { DEFAULT_BOT_CONFIG, type BotConfig } from "../core/bot_config";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";

import {
  applySavedLocalMatchSetup,
  disposeLocalMatchSession,
  ensureLocalMatchSession,
  startLocalMatchWithSavedSetup,
} from "./local_match_session";

const initialLocalProfileState = localProfileStore.getState();
const localProfile = {
  avatarUrl: null,
  createdAt: "2026-05-15T00:00:00.000Z",
  displayName: "Bryan",
  id: "local-1",
  kind: "local" as const,
  updatedAt: "2026-05-15T00:00:00.000Z",
  username: null,
};
const noOpBotRunner = {
  chooseMove: async () => null,
  configure: () => undefined,
  dispose: () => undefined,
};

const defaultSettings = createDefaultProfileSettings();

function settingsWith(ruleset: "freestyle" | "renju", botConfig: BotConfig = DEFAULT_BOT_CONFIG) {
  return {
    ...defaultSettings,
    botConfig,
    gameConfig: {
      opening: "standard" as const,
      ruleset,
    },
  };
}

describe("local match session", () => {
  afterEach(() => {
    disposeLocalMatchSession();
    localProfileStore.setState(initialLocalProfileState, true);
  });

  it("keeps the active match store across repeated route entries", () => {
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: settingsWith("freestyle"),
    });

    const first = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(first.getState().placeHumanMove(7, 7)).toBe(true);

    const second = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(second).toBe(first);
    expect(second.getState().moves).toHaveLength(1);
  });

  it("applies saved setup to the selected next game without mutating a game in progress", () => {
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: settingsWith("freestyle"),
    });

    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    localProfileStore.getState().updateSettings({
      botConfig: hard,
      gameConfig: {
        opening: "standard",
        ruleset: "renju",
      },
    });
    applySavedLocalMatchSetup();

    expect(store.getState()).toMatchObject({
      currentBotConfig: DEFAULT_BOT_CONFIG,
      currentVariant: "freestyle",
      selectedBotConfig: hard,
      selectedVariant: "renju",
    });

    startLocalMatchWithSavedSetup();

    expect(store.getState()).toMatchObject({
      currentBotConfig: hard,
      currentVariant: "renju",
      moves: [],
      selectedBotConfig: hard,
      selectedVariant: "renju",
    });
  });

  it("resumes replay branches with the current bot config and replay rule", () => {
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: settingsWith("freestyle", hard),
    });

    const store = ensureLocalMatchSession({
      botRunner: noOpBotRunner,
      resumeState: {
        currentPlayer: 1,
        moves: [
          { col: 7, moveNumber: 1, player: 1, row: 7 },
          { col: 8, moveNumber: 2, player: 2, row: 7 },
        ],
        variant: "renju",
      },
    });

    expect(store.getState()).toMatchObject({
      currentBotConfig: hard,
      currentVariant: "renju",
      selectedBotConfig: hard,
      selectedVariant: "renju",
    });
    expect(store.getState().players[1]).toMatchObject({
      kind: "bot",
      name: "Hard Bot",
      stone: "white",
    });
  });
});
