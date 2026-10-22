import { createStore, type StoreApi } from "zustand/vanilla";

import { BOT_CONFIG_VERSION, BOT_ENGINE, BOT_VERSION } from "../match/saved_match";
import {
  currentReplayLibraryOwner,
  replayLibraryStore as defaultReplayLibraryStore,
  type ReplayLibraryOwner,
  type ReplayLibraryState,
} from "../replay/replay_library_store";
import type { PortableReplayParticipantV1, PortableReplayV1 } from "../replay/portable_replay";
import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { cloudMatchHistoryHasMatch } from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { flushCloudProfileSync } from "../cloud/cloud_sync";
import { localProfileStore } from "../profile/local_profile_store";

import {
  createLocalMatchStore,
  type LocalMatchResumeSeed,
  type LocalMatchState,
  type LocalMatchStoreOptions,
} from "./local_match_store";

export interface PendingPracticeSave {
  error: string | null;
  owner: ReplayLibraryOwner;
  replay: PortableReplayV1;
  saving: boolean;
}

export interface LocalMatchSessionState {
  latestReplayId: string | null;
  matchStore: StoreApi<LocalMatchState> | null;
  pendingPracticeSave: PendingPracticeSave | null;
}

type FinishedLocalMatch = Parameters<NonNullable<LocalMatchStoreOptions["onMatchFinished"]>>[0];

export interface LocalMatchSessionOptions
  extends Omit<
    LocalMatchStoreOptions,
    "humanDisplayName" | "onMatchFinished" | "botConfig" | "resumeState" | "variant"
  > {
  replayLibraryStore?: StoreApi<ReplayLibraryState>;
  resumeState?: LocalMatchResumeSeed;
}

export const localMatchSessionStore = createStore<LocalMatchSessionState>(() => ({
  latestReplayId: null,
  matchStore: null,
  pendingPracticeSave: null,
}));

interface PendingPracticeTask {
  libraryStore: StoreApi<ReplayLibraryState>;
  matchStore: StoreApi<LocalMatchState>;
  moves: LocalMatchState["moves"];
  originUndoFloor: number;
  owner: ReplayLibraryOwner;
  replay: PortableReplayV1;
  status: Exclude<LocalMatchState["status"], "playing">;
  saving: boolean;
}

let pendingPracticeTask: PendingPracticeTask | null = null;
let latestPracticeOwner: ReplayLibraryOwner | null = null;
let matchStateUnsubscribe: (() => void) | null = null;

function errorMessage(error: unknown): string {
  if (error instanceof Error && error.message.trim()) return error.message;
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) return message;
  }
  return "Could not save this practice replay. Try again.";
}

function isFinishedRoundState(task: PendingPracticeTask, state: LocalMatchState): boolean {
  return state.status === task.status
    && state.undoFloor === task.originUndoFloor
    && state.moves.length === task.moves.length
    && state.moves.every((move, index) => {
      const expected = task.moves[index];
      return expected !== undefined
        && move.col === expected.col
        && move.moveNumber === expected.moveNumber
        && move.player === expected.player
        && move.row === expected.row;
    });
}

function stillOnFinishedRound(task: PendingPracticeTask): boolean {
  return isFinishedRoundState(task, task.matchStore.getState());
}

function clearPendingPracticeTask(task: PendingPracticeTask): void {
  if (pendingPracticeTask !== task) return;
  pendingPracticeTask = null;
  if (localMatchSessionStore.getState().matchStore === task.matchStore) {
    localMatchSessionStore.setState({ pendingPracticeSave: null });
  }
}

async function savePendingPracticeTask(task: PendingPracticeTask): Promise<string | null> {
  if (pendingPracticeTask !== task || task.saving) return null;
  if (
    localMatchSessionStore.getState().matchStore !== task.matchStore
    || currentReplayLibraryOwner() !== task.owner
    || !stillOnFinishedRound(task)
  ) {
    clearPendingPracticeTask(task);
    return null;
  }

  task.saving = true;
  localMatchSessionStore.setState({
    pendingPracticeSave: {
      error: null,
      owner: task.owner,
      replay: task.replay,
      saving: true,
    },
  });

  try {
    const fingerprint = await task.libraryStore.getState().saveReplay(task.owner, task.replay, "practice");
    if (pendingPracticeTask !== task) return null;
    if (
      localMatchSessionStore.getState().matchStore !== task.matchStore
      || currentReplayLibraryOwner() !== task.owner
      || !stillOnFinishedRound(task)
    ) {
      clearPendingPracticeTask(task);
      return null;
    }

    pendingPracticeTask = null;
    latestPracticeOwner = task.owner;
    localMatchSessionStore.setState({
      latestReplayId: `portable-${fingerprint}`,
      pendingPracticeSave: null,
    });
    return fingerprint;
  } catch (error) {
    if (pendingPracticeTask === task) {
      task.saving = false;
      if (
        localMatchSessionStore.getState().matchStore === task.matchStore
        && currentReplayLibraryOwner() === task.owner
        && stillOnFinishedRound(task)
      ) {
        localMatchSessionStore.setState({
          pendingPracticeSave: {
            error: errorMessage(error),
            owner: task.owner,
            replay: task.replay,
            saving: false,
          },
        });
      } else {
        clearPendingPracticeTask(task);
      }
    }
    return null;
  }
}

/** Retries a failed practice save for the still-active finished round. */
export function retryPracticeSave(): Promise<string | null> {
  const task = pendingPracticeTask;
  if (!task) return Promise.resolve(null);
  return savePendingPracticeTask(task);
}

function replayParticipant(
  player: FinishedLocalMatch["players"][number],
  botConfig: FinishedLocalMatch["botConfig"],
  profileUid: string | null,
): PortableReplayParticipantV1 {
  if (player.kind === "human") {
    return {
      kind: "human",
      name: player.name,
      ...(profileUid && /^[A-Za-z0-9_-]{1,128}$/.test(profileUid) ? { profileUid } : {}),
    };
  }

  return {
    bot: {
      config: botConfig,
      configVersion: BOT_CONFIG_VERSION,
      engine: BOT_ENGINE,
      version: BOT_VERSION,
    },
    kind: "bot",
    name: player.name,
  };
}

function portablePracticeReplay(
  match: FinishedLocalMatch,
  sourceFingerprint: string,
  inheritedPly: number,
  profileUid: string | null,
): PortableReplayV1 {
  const black = match.players.find((player) => player.stone === "black");
  const white = match.players.find((player) => player.stone === "white");
  if (!black || !white) {
    throw new Error("Finished match is missing a player for one of the stones.");
  }

  return {
    black: replayParticipant(black, match.botConfig, profileUid),
    moves: match.moves.map((move) => move.row * 15 + move.col),
    practice: { sourceFingerprint, inheritedPly },
    ruleset: match.variant,
    version: 1,
    white: replayParticipant(white, match.botConfig, profileUid),
  };
}

function createSessionMatchStore(options: LocalMatchSessionOptions = {}): StoreApi<LocalMatchState> {
  const { replayLibraryStore: replayLibraryStoreOption, ...matchOptions } = options;
  const localProfile = localProfileStore.getState().ensureLocalProfile();
  const settings = localProfileStore.getState().settings;
  const replayLibraryStore = replayLibraryStoreOption ?? defaultReplayLibraryStore;
  let sessionMatchStore: StoreApi<LocalMatchState> | null = null;

  const matchStore = createLocalMatchStore({
    ...matchOptions,
    humanDisplayName: localProfile.displayName,
    onMatchFinished: (match) => {
      const practiceOrigin = options.resumeState?.practiceOrigin;
      if (match.undoFloor > 0 && practiceOrigin) {
        // Snapshot the owner and optional UID before saveReplay starts its async
        // fingerprint, so a later sign-in or account switch cannot redirect it.
        const replayOwner = currentReplayLibraryOwner();
        const authAtFinish = cloudAuthStore.getState();
        const humanProfileUid = authAtFinish.status === "signed_in"
          ? authAtFinish.user?.uid ?? null
          : null;
        const replay = portablePracticeReplay(
          match,
          practiceOrigin.sourceFingerprint,
          practiceOrigin.inheritedPly,
          humanProfileUid,
        );
        if (!sessionMatchStore) return;
        const task: PendingPracticeTask = {
          libraryStore: replayLibraryStore,
          matchStore: sessionMatchStore,
          moves: match.moves.map((move) => ({ ...move })),
          originUndoFloor: match.undoFloor,
          owner: replayOwner,
          replay,
          status: match.status,
          saving: false,
        };
        pendingPracticeTask = task;
        latestPracticeOwner = null;
        localMatchSessionStore.setState({ latestReplayId: null });
        localMatchSessionStore.setState({
          pendingPracticeSave: {
            error: null,
            owner: task.owner,
            replay: task.replay,
            saving: false,
          },
        });
        void savePendingPracticeTask(task);
        return;
      }

      if (pendingPracticeTask?.matchStore === sessionMatchStore) {
        pendingPracticeTask = null;
        localMatchSessionStore.setState({ pendingPracticeSave: null });
      }
      latestPracticeOwner = null;

      const replayId = localProfileStore.getState().recordFinishedMatch(match);
      const localMatchHistory = localProfileStore.getState().matchHistory;
      const savedMatch = localMatchHistory.replayMatches.find((entry) => entry.id === replayId);
      const cloudAuth = cloudAuthStore.getState();

      if (savedMatch && cloudAuth.status === "signed_in" && cloudAuth.user) {
        const signedInUser = cloudAuth.user;
        void flushCloudProfileSync(signedInUser, { localMatchHistory }).then((cloudProfile) => {
          if (cloudAuthStore.getState().user !== signedInUser) {
            return;
          }
          if (!cloudProfile) {
            // A cancelled/reset flush must not upload a pre-reset match. Local
            // history remains available for the next current-session sync.
            return;
          }

          if (cloudProfileStore.getState().profile !== cloudProfile) {
            return;
          }

          cloudHistoryStore.getState().loadFromProfile(signedInUser, cloudProfile);
          if (!cloudMatchHistoryHasMatch(cloudProfile.matchHistory, savedMatch.id)) {
            void cloudHistoryStore.getState().syncMatchForUser(
              signedInUser,
              savedMatch,
              cloudProfile.resetAt,
            );
          }
        }).catch(() => {
          // Cloud stores retain the failure; a local replay is still available.
        });
      }

      localMatchSessionStore.setState({ latestReplayId: replayId });
    },
    botConfig: settings.botConfig,
    resumeState: options.resumeState,
    variant: options.resumeState?.variant ?? settings.gameConfig.ruleset,
  });

  sessionMatchStore = matchStore;
  return matchStore;
}

export function ensureLocalMatchSession(
  options: LocalMatchSessionOptions = {},
): StoreApi<LocalMatchState> {
  const existingStore = localMatchSessionStore.getState().matchStore;
  if (existingStore && !options.resumeState) {
    return existingStore;
  }

  if (existingStore) {
    matchStateUnsubscribe?.();
    matchStateUnsubscribe = null;
    existingStore.getState().dispose();
  }

  pendingPracticeTask = null;
  latestPracticeOwner = null;
  const matchStore = createSessionMatchStore(options);
  localMatchSessionStore.setState({ latestReplayId: null, matchStore, pendingPracticeSave: null });
  matchStateUnsubscribe = matchStore.subscribe((state) => {
    const task = pendingPracticeTask;
    if (task?.matchStore === matchStore && !isFinishedRoundState(task, state)) {
      clearPendingPracticeTask(task);
    }
  });
  return matchStore;
}

export function disposeLocalMatchSession(): void {
  matchStateUnsubscribe?.();
  matchStateUnsubscribe = null;
  localMatchSessionStore.getState().matchStore?.getState().dispose();
  pendingPracticeTask = null;
  latestPracticeOwner = null;
  localMatchSessionStore.setState({ latestReplayId: null, matchStore: null, pendingPracticeSave: null });
}

export function clearLocalMatchLatestReplay(): void {
  latestPracticeOwner = null;
  localMatchSessionStore.setState({ latestReplayId: null });
}

export function applySavedLocalMatchSetup(): void {
  const matchStore = localMatchSessionStore.getState().matchStore;
  if (!matchStore) {
    return;
  }

  const settings = localProfileStore.getState().settings;
  if (matchStore.getState().selectedVariant !== settings.gameConfig.ruleset) {
    matchStore.getState().selectVariant(settings.gameConfig.ruleset);
  }
  if (JSON.stringify(matchStore.getState().selectedBotConfig) !== JSON.stringify(settings.botConfig)) {
    matchStore.getState().selectBotConfig(settings.botConfig);
  }
}

export function startLocalMatchWithSavedSetup(): StoreApi<LocalMatchState> {
  const matchStore = ensureLocalMatchSession();
  applySavedLocalMatchSetup();
  matchStore.getState().startNewMatch();
  localMatchSessionStore.setState({ latestReplayId: null });
  return matchStore;
}

cloudAuthStore.subscribe(() => {
  const owner = currentReplayLibraryOwner();
  const pending = pendingPracticeTask;
  if (pending && pending.owner !== owner) {
    pendingPracticeTask = null;
    if (localMatchSessionStore.getState().matchStore === pending.matchStore) {
      localMatchSessionStore.setState({ pendingPracticeSave: null });
    }
  }
  if (latestPracticeOwner && latestPracticeOwner !== owner) {
    latestPracticeOwner = null;
    localMatchSessionStore.setState({ latestReplayId: null });
  }
});

// The session owns this binding so optional cloud restoration does not eagerly
// load the board/Wasm on public routes. Replay branches keep their own rules
// until the player actually changes the saved setup.
localProfileStore.subscribe((state, previous) => {
  if (state.profile?.id !== previous.profile?.id) {
    disposeLocalMatchSession();
  } else if (state.settings !== previous.settings) {
    applySavedLocalMatchSetup();
  }
});
