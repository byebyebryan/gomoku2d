import { afterEach, describe, expect, it } from "vitest";

import { DEFAULT_PRACTICE_BOT_CONFIG, type PracticeBotConfig } from "../core/practice_bot_config";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";

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

describe("local match session", () => {
  afterEach(() => {
    disposeLocalMatchSession();
    localProfileStore.setState(initialLocalProfileState, true);
  });

  it("keeps the active match store across repeated route entries", () => {
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: { practiceBot: DEFAULT_PRACTICE_BOT_CONFIG, preferredVariant: "freestyle" },
    });

    const first = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(first.getState().placeHumanMove(7, 7)).toBe(true);

    const second = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(second).toBe(first);
    expect(second.getState().moves).toHaveLength(1);
  });

  it("applies saved setup to the selected next game without mutating a game in progress", () => {
    const hard: PracticeBotConfig = { mode: "preset", preset: "hard", version: 1 };
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: { practiceBot: DEFAULT_PRACTICE_BOT_CONFIG, preferredVariant: "freestyle" },
    });

    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    localProfileStore.getState().updateSettings({ practiceBot: hard, preferredVariant: "renju" });
    applySavedLocalMatchSetup();

    expect(store.getState()).toMatchObject({
      currentPracticeBot: DEFAULT_PRACTICE_BOT_CONFIG,
      currentVariant: "freestyle",
      selectedPracticeBot: hard,
      selectedVariant: "renju",
    });

    startLocalMatchWithSavedSetup();

    expect(store.getState()).toMatchObject({
      currentPracticeBot: hard,
      currentVariant: "renju",
      moves: [],
      selectedPracticeBot: hard,
      selectedVariant: "renju",
    });
  });
});
