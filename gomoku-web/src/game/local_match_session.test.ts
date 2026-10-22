import { afterEach, describe, expect, it, vi } from "vitest";

import { DEFAULT_BOT_CONFIG, type BotConfig } from "../core/bot_config";
import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import { portableReplayFromSavedMatch } from "../replay/portable_replay";
import * as cloudSync from "../cloud/cloud_sync";
import {
  createLocalProfileTestState,
  noOpBotRunner,
} from "../test/local_match_fixtures";

import {
  applySavedLocalMatchSetup,
  disposeLocalMatchSession,
  ensureLocalMatchSession,
  localMatchSessionStore,
  startLocalMatchWithSavedSetup,
} from "./local_match_session";

const initialLocalProfileState = localProfileStore.getState();
const initialCloudAuthState = cloudAuthStore.getState();
const defaultSettings = createDefaultProfileSettings();
const ORIGIN = { sourceFingerprint: "A".repeat(43), inheritedPly: 8 };

function testUser(uid: string): CloudAuthUser {
  return {
    avatarUrl: null,
    displayName: `User ${uid}`,
    email: null,
    providerIds: [],
    uid,
  };
}

function setSignedIn(uid: string): void {
  cloudAuthStore.setState({ ...initialCloudAuthState, status: "signed_in", user: testUser(uid) });
}

function setSignedOut(): void {
  cloudAuthStore.setState({ ...initialCloudAuthState, status: "signed_out", user: null });
}

function practiceBranch() {
  const cells = [110, 0, 111, 1, 112, 2, 113, 3];
  return {
    currentPlayer: 1 as const,
    moves: cells.map((cell, index) => ({
      col: cell % 15,
      moveNumber: index + 1,
      player: (index % 2 + 1) as 1 | 2,
      row: Math.floor(cell / 15),
    })),
    practiceOrigin: ORIGIN,
    undoFloor: 8,
    variant: "freestyle" as const,
  };
}

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
    vi.restoreAllMocks();
    disposeLocalMatchSession();
    localProfileStore.setState(initialLocalProfileState, true);
    cloudAuthStore.setState(initialCloudAuthState, true);
  });

  it("keeps the active match store across repeated route entries", () => {
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));

    const first = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(first.getState().placeHumanMove(7, 7)).toBe(true);

    const second = ensureLocalMatchSession({ botRunner: noOpBotRunner });

    expect(second).toBe(first);
    expect(second.getState().moves).toHaveLength(1);
  });

  it("applies saved setup to the selected next game without mutating a game in progress", () => {
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));

    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(store.getState().placeHumanMove(7, 7)).toBe(true);

    localProfileStore.getState().updateSettings({
      botConfig: hard,
      gameConfig: {
        opening: "standard",
        ruleset: "renju",
      },
    });
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

  it("disposes the old game when the local identity is reset", () => {
    localProfileStore.setState(createLocalProfileTestState());
    const dispose = vi.fn();
    ensureLocalMatchSession({ botRunner: { ...noOpBotRunner, dispose } });
    localProfileStore.getState().resetLocalProfile();
    expect(dispose).toHaveBeenCalledOnce();
    expect(localMatchSessionStore.getState().matchStore).toBeNull();
  });

  it("does not reset an empty game when only hint preferences change", () => {
    localProfileStore.setState(createLocalProfileTestState());
    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    const initial = store.getState();
    localProfileStore.getState().updateSettings({
      boardHints: { ...defaultSettings.boardHints, immediate: "off" },
    });
    applySavedLocalMatchSetup();
    expect(store.getState()).toBe(initial);
  });

  it("resumes replay branches with the current bot config and replay rule", () => {
    const hard: BotConfig = { mode: "preset", preset: "hard", version: 1 };
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle", hard)));

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

  it("records a completed continuation as a regular game with its replay origin", () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const recordFinishedMatch = vi.spyOn(localProfileStore.getState(), "recordFinishedMatch");
    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner, resumeState: practiceBranch() });

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    const [saved] = localProfileStore.getState().matchHistory.replayMatches;
    expect(store.getState().status).toBe("black_won");
    expect(recordFinishedMatch).toHaveBeenCalledOnce();
    expect(localProfileStore.getState().matchHistory.replayMatches).toHaveLength(1);
    expect(saved).toMatchObject({
      undo_floor: 8,
      replay_origin: { source_fingerprint: ORIGIN.sourceFingerprint, inherited_ply: 8 },
      move_cells: [110, 0, 111, 1, 112, 2, 113, 3, 114],
      source: "local_history",
      trust: "local_only",
    });
    expect(portableReplayFromSavedMatch(saved).replay.practice).toEqual(ORIGIN);
    expect(localMatchSessionStore.getState().latestReplayId).toBe(saved?.id);
    expect(saved?.id).not.toMatch(/^portable-/);
  });

  it("clears the replay origin when a continuation starts a fresh round", async () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const botCells = [110, 111, 112, 113, 114];
    const botRunner = {
      chooseMove: async () => {
        const cell = botCells.shift();
        return cell === undefined ? null : { row: Math.floor(cell / 15), col: cell % 15 };
      },
      configure: () => undefined,
      dispose: () => undefined,
    };
    const store = ensureLocalMatchSession({ botRunner, resumeState: practiceBranch() });
    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    const continuationId = localMatchSessionStore.getState().latestReplayId;
    store.getState().startNextRound();
    await vi.waitFor(() => expect(store.getState()).toMatchObject({ pendingBotMove: false, moves: [{ row: 7, col: 5 }] }));
    for (let col = 0; col < 4; col += 1) {
      expect(store.getState().placeHumanMove(0, col)).toBe(true);
      await vi.waitFor(() => expect(store.getState().pendingBotMove).toBe(false));
    }
    await vi.waitFor(() => expect(store.getState().status).toBe("black_won"));
    const matches = localProfileStore.getState().matchHistory.replayMatches;
    expect(matches).toHaveLength(2);
    const fresh = matches.find((match) => match.id !== continuationId);
    expect(fresh?.undo_floor).toBe(0);
    expect(fresh?.replay_origin).toBeUndefined();
    expect(matches.find((match) => match.id === continuationId)?.replay_origin?.inherited_ply).toBe(8);
    expect(localMatchSessionStore.getState().latestReplayId).toBe(fresh?.id);
  });

  it("syncs a continuation through normal history under the account captured at finish", async () => {
    setSignedIn("account-A");
    const flush = vi.spyOn(cloudSync, "flushCloudProfileSync").mockResolvedValue(null);
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const store = ensureLocalMatchSession({ botRunner: noOpBotRunner, resumeState: practiceBranch() });
    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    setSignedIn("account-B");
    expect(flush).toHaveBeenCalledOnce();
    expect(flush).toHaveBeenCalledWith(expect.objectContaining({ uid: "account-A" }), {
      localMatchHistory: expect.objectContaining({ replayMatches: [expect.objectContaining({
        replay_origin: { source_fingerprint: ORIGIN.sourceFingerprint, inherited_ply: 8 },
      })] }),
    });
    expect(localMatchSessionStore.getState().latestReplayId).toBe(localProfileStore.getState().matchHistory.replayMatches[0]?.id);
  });
});
