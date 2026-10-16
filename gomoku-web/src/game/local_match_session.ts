import { createStore, type StoreApi } from "zustand/vanilla";

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

interface LocalMatchSessionState {
  latestReplayId: string | null;
  matchStore: StoreApi<LocalMatchState> | null;
}

export interface LocalMatchSessionOptions
  extends Omit<
    LocalMatchStoreOptions,
    "humanDisplayName" | "onMatchFinished" | "botConfig" | "resumeState" | "variant"
  > {
  resumeState?: LocalMatchResumeSeed;
}

export const localMatchSessionStore = createStore<LocalMatchSessionState>(() => ({
  latestReplayId: null,
  matchStore: null,
}));

function createSessionMatchStore(options: LocalMatchSessionOptions = {}): StoreApi<LocalMatchState> {
  const localProfile = localProfileStore.getState().ensureLocalProfile();
  const settings = localProfileStore.getState().settings;

  return createLocalMatchStore({
    ...options,
    humanDisplayName: localProfile.displayName,
    onMatchFinished: (match) => {
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
}

export function ensureLocalMatchSession(
  options: LocalMatchSessionOptions = {},
): StoreApi<LocalMatchState> {
  const existingStore = localMatchSessionStore.getState().matchStore;
  if (existingStore && !options.resumeState) {
    return existingStore;
  }

  if (existingStore) {
    existingStore.getState().dispose();
  }

  const matchStore = createSessionMatchStore(options);
  localMatchSessionStore.setState({ latestReplayId: null, matchStore });
  return matchStore;
}

export function disposeLocalMatchSession(): void {
  localMatchSessionStore.getState().matchStore?.getState().dispose();
  localMatchSessionStore.setState({ latestReplayId: null, matchStore: null });
}

export function clearLocalMatchLatestReplay(): void {
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
