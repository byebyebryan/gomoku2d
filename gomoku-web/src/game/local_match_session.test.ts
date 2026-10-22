import { afterEach, describe, expect, it, vi } from "vitest";

import { DEFAULT_BOT_CONFIG, type BotConfig } from "../core/bot_config";
import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import { decodePortableReplay } from "../replay/portable_replay_codec";
import { createReplayLibraryStore, REPLAY_LIBRARY_LIMIT } from "../replay/replay_library_store";
import type { PortableReplayV1 } from "../replay/portable_replay";
import {
  createLocalProfileTestState,
  noOpBotRunner,
} from "../test/local_match_fixtures";

import {
  applySavedLocalMatchSetup,
  disposeLocalMatchSession,
  ensureLocalMatchSession,
  localMatchSessionStore,
  retryPracticeSave,
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

function createMemoryReplayLibrary() {
  const backing = new Map<string, string>();
  return createReplayLibraryStore({
    storage: {
      getItem: (name) => backing.get(name) ?? null,
      removeItem: (name) => { backing.delete(name); },
      setItem: (name, value) => { backing.set(name, value); },
    },
  });
}

function libraryReplay(name: string): PortableReplayV1 {
  return {
    black: { kind: "human", name },
    moves: [110, 0, 111, 1, 112, 2, 113, 3, 114],
    ruleset: "freestyle",
    version: 1,
    white: {
      bot: {
        config: DEFAULT_BOT_CONFIG,
        configVersion: 1,
        engine: "search_bot",
        version: 1,
      },
      kind: "bot",
      name: "Normal Bot",
    },
  };
}

function deferReplayLibrarySave(store: ReturnType<typeof createMemoryReplayLibrary>) {
  let resolveSave!: (fingerprint: string) => void;
  const pending = new Promise<string>((resolve) => { resolveSave = resolve; });
  const settled = pending.then(() => undefined);
  store.setState({ saveReplay: () => pending });
  return { resolveSave, settled };
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

  it("saves a finished branch as practice and leaves ordinary match history untouched", async () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const replayLibrary = createMemoryReplayLibrary();
    const recordFinishedMatch = vi.spyOn(localProfileStore.getState(), "recordFinishedMatch");
    const store = ensureLocalMatchSession({
      botRunner: noOpBotRunner,
      replayLibraryStore: replayLibrary,
      resumeState: practiceBranch(),
    });

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    await vi.waitFor(() => expect(replayLibrary.getState().owners.guest?.entries).toHaveLength(1));

    expect(store.getState().status).toBe("black_won");
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([]);
    expect(recordFinishedMatch).not.toHaveBeenCalled();
    expect(replayLibrary.getState().owners.guest?.entries[0]?.kind).toBe("practice");
    expect(decodePortableReplay(replayLibrary.getState().owners.guest!.entries[0]!.payload).replay)
      .toMatchObject({ moves: [110, 0, 111, 1, 112, 2, 113, 3, 114], practice: ORIGIN });
    expect(localMatchSessionStore.getState().latestReplayId)
      .toBe(`portable-${replayLibrary.getState().owners.guest!.entries[0]!.fingerprint}`);
  });

  it("returns to ordinary match history when a practice branch starts a new round", async () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const replayLibrary = createMemoryReplayLibrary();
    const botCells = [110, 111, 112, 113, 114];
    const botRunner = {
      chooseMove: async () => {
        const cell = botCells.shift();
        return cell === undefined ? null : { row: Math.floor(cell / 15), col: cell % 15 };
      },
      configure: () => undefined,
      dispose: () => undefined,
    };
    const store = ensureLocalMatchSession({
      botRunner,
      replayLibraryStore: replayLibrary,
      resumeState: practiceBranch(),
    });

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    await vi.waitFor(() => expect(replayLibrary.getState().owners.guest?.entries).toHaveLength(1));
    store.getState().startNextRound();
    await vi.waitFor(() => expect(store.getState()).toMatchObject({ pendingBotMove: false, moves: [{ row: 7, col: 5 }] }));

    for (let col = 0; col < 4; col += 1) {
      expect(store.getState().placeHumanMove(0, col)).toBe(true);
      await vi.waitFor(() => expect(store.getState().pendingBotMove).toBe(false));
    }
    await vi.waitFor(() => expect(store.getState().status).toBe("black_won"));

    const ordinaryReplay = localProfileStore.getState().matchHistory.replayMatches[0];
    expect(localProfileStore.getState().matchHistory.replayMatches).toHaveLength(1);
    expect(ordinaryReplay?.undo_floor).toBe(0);
    expect(JSON.stringify(ordinaryReplay)).not.toContain(ORIGIN.sourceFingerprint);
    expect(replayLibrary.getState().owners.guest?.entries).toHaveLength(1);
    expect(localMatchSessionStore.getState().latestReplayId).not.toMatch(/^portable-/);
  });

  it("keeps a branch under the account captured at finish if auth changes during hashing", async () => {
    setSignedIn("account-A");
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const replayLibrary = createMemoryReplayLibrary();
    const store = ensureLocalMatchSession({
      botRunner: noOpBotRunner,
      replayLibraryStore: replayLibrary,
      resumeState: practiceBranch(),
    });

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    setSignedIn("account-B");
    await vi.waitFor(() => expect(replayLibrary.getState().owners["user:account-A"]?.entries).toHaveLength(1));

    const entry = replayLibrary.getState().owners["user:account-A"]!.entries[0]!;
    expect(entry.kind).toBe("practice");
    expect(replayLibrary.getState().owners["user:account-B"]).toBeUndefined();
    expect(localMatchSessionStore.getState().latestReplayId).toBeNull();
    expect(localMatchSessionStore.getState().pendingPracticeSave).toBeNull();
    expect(decodePortableReplay(entry.payload).replay.black).toMatchObject({ profileUid: "account-A" });
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([]);
  });

  it("keeps a full-library completion pending and retries it after a replay is removed", async () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const replayLibrary = createMemoryReplayLibrary();
    const fingerprints: string[] = [];
    for (let index = 0; index < REPLAY_LIBRARY_LIMIT; index += 1) {
      fingerprints.push(await replayLibrary.getState().saveReplay("guest", libraryReplay(`Existing ${index}`), "saved"));
    }

    const store = ensureLocalMatchSession({
      botRunner: noOpBotRunner,
      replayLibraryStore: replayLibrary,
      resumeState: practiceBranch(),
    });
    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    await vi.waitFor(() => expect(localMatchSessionStore.getState().pendingPracticeSave?.error).toContain("full"));
    expect(localMatchSessionStore.getState().pendingPracticeSave?.replay.practice).toEqual(ORIGIN);
    expect(localMatchSessionStore.getState().latestReplayId).toBeNull();
    expect(store.getState().status).toBe("black_won");

    await replayLibrary.getState().removeReplay("guest", fingerprints[0]!);
    const fingerprint = await retryPracticeSave();

    expect(fingerprint).toBeTruthy();
    expect(localMatchSessionStore.getState().latestReplayId).toBe(`portable-${fingerprint}`);
    expect(localMatchSessionStore.getState().pendingPracticeSave).toBeNull();
    expect(replayLibrary.getState().owners.guest?.entries).toHaveLength(REPLAY_LIBRARY_LIMIT);
    expect(replayLibrary.getState().owners.guest?.entries[0]?.kind).toBe("practice");
  });

  it.each(["replaced session", "new round"] as const)(
    "does not attach a late practice save to a %s",
    async (change) => {
      setSignedOut();
      localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
      const replayLibrary = createMemoryReplayLibrary();
      const delayed = deferReplayLibrarySave(replayLibrary);
      const store = ensureLocalMatchSession({
        botRunner: noOpBotRunner,
        replayLibraryStore: replayLibrary,
        resumeState: practiceBranch(),
      });
      expect(store.getState().placeHumanMove(7, 9)).toBe(true);
      expect(localMatchSessionStore.getState().pendingPracticeSave?.saving).toBe(true);

      let activeStore = store;
      if (change === "replaced session") {
        disposeLocalMatchSession();
        activeStore = ensureLocalMatchSession({ botRunner: noOpBotRunner });
      } else {
        store.getState().startNextRound();
        expect(localMatchSessionStore.getState().pendingPracticeSave).toBeNull();
      }
      delayed.resolveSave("D".repeat(43));
      await delayed.settled;

      expect(localMatchSessionStore.getState().matchStore).toBe(activeStore);
      expect(localMatchSessionStore.getState().latestReplayId).toBeNull();
      expect(localMatchSessionStore.getState().pendingPracticeSave).toBeNull();
    },
  );

  it("keeps a finished branch inspectable when browser storage rejects its save", async () => {
    setSignedOut();
    localProfileStore.setState(createLocalProfileTestState(settingsWith("freestyle")));
    const replayLibrary = createReplayLibraryStore({
      storage: {
        getItem: () => null,
        removeItem: () => undefined,
        setItem: () => { throw new DOMException("Storage full", "QuotaExceededError"); },
      },
    });
    const store = ensureLocalMatchSession({
      botRunner: noOpBotRunner,
      replayLibraryStore: replayLibrary,
      resumeState: practiceBranch(),
    });

    expect(store.getState().placeHumanMove(7, 9)).toBe(true);
    await vi.waitFor(() => expect(localMatchSessionStore.getState().latestReplayId).toMatch(/^portable-/));

    expect(store.getState().status).toBe("black_won");
    expect(store.getState().moves).toHaveLength(9);
    expect(replayLibrary.getState().owners.guest?.entries).toHaveLength(1);
    expect(replayLibrary.getState().owners.guest?.error).toContain("Storage full");
  });
});
