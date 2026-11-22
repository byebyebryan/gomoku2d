import { createStore, type StoreApi } from "zustand/vanilla";

import type { ProfileSettings } from "../profile/profile_settings";

import type { CloudAuthUser } from "./auth_store";
import { deleteCloudProfile, ensureCloudProfile, resetCloudProfile, type CloudProfile } from "./cloud_profile";

export type CloudProfileStatus = "idle" | "loading" | "ready" | "error";

export interface CloudProfileState {
  applyLocalPatch: (source: CloudProfile, patch: Partial<Omit<CloudProfile, "uid">>) => boolean;
  deleteForUser: (user: CloudAuthUser) => Promise<void>;
  errorMessage: string | null;
  loadForUser: (user: CloudAuthUser, settings: ProfileSettings) => Promise<CloudProfile | null>;
  profile: CloudProfile | null;
  reset: () => void;
  resetForUser: (user: CloudAuthUser, settings: ProfileSettings) => Promise<void>;
  status: CloudProfileStatus;
}

export interface CloudProfileStoreOptions {
  deleteProfile?: (user: CloudAuthUser) => Promise<void>;
  loadProfile?: (user: CloudAuthUser, settings: ProfileSettings) => Promise<CloudProfile>;
  resetProfile?: (user: CloudAuthUser, settings: ProfileSettings) => Promise<CloudProfile>;
}

function errorMessageFor(error: unknown): string {
  return error instanceof Error ? error.message : "Cloud profile failed to load.";
}

export function createCloudProfileStore(
  options: CloudProfileStoreOptions = {},
): StoreApi<CloudProfileState> {
  const deleteProfile = options.deleteProfile ?? deleteCloudProfile;
  const loadProfile = options.loadProfile ?? ensureCloudProfile;
  const resetProfile = options.resetProfile ?? resetCloudProfile;
  let requestId = 0;
  let requestEpoch = 0;
  let requestUser: CloudAuthUser | null = null;
  let requestKind: "delete" | "load" | "reset" | null = null;
  const startRequest = (user: CloudAuthUser, kind: "delete" | "load" | "reset") => {
    if (requestUser !== user || kind === "delete" || kind === "reset") requestEpoch += 1;
    requestUser = user;
    requestKind = kind;
    requestId += 1;
    return { id: requestId, epoch: requestEpoch };
  };

  return createStore<CloudProfileState>((set, get) => ({
    applyLocalPatch: (source, patch) => {
      if (get().profile !== source || get().status !== "ready") {
        return false;
      }
      set({ profile: { ...source, ...patch } });
      return true;
    },
    deleteForUser: async (user) => {
      const { id: currentRequestId } = startRequest(user, "delete");

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        await deleteProfile(user);
        if (requestId !== currentRequestId) {
          return;
        }

        set({
          errorMessage: null,
          profile: null,
          status: "idle",
        });
      } catch (error) {
        if (requestId === currentRequestId) {
          set({
            errorMessage: errorMessageFor(error),
            status: "error",
          });
        }
        throw error;
      }
    },
    errorMessage: null,
    loadForUser: async (user, settings) => {
      const { id: currentRequestId } = startRequest(user, "load");

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await loadProfile(user, settings);
        if (requestId !== currentRequestId) {
          return null;
        }

        set({
          errorMessage: null,
          profile,
          status: "ready",
        });
        return profile;
      } catch (error) {
        if (requestId !== currentRequestId) {
          return null;
        }

        set({
          errorMessage: errorMessageFor(error),
          status: "error",
        });
        return null;
      }
    },
    profile: null,
    reset: () => {
      requestId += 1;
      requestEpoch += 1;
      requestUser = null;
      requestKind = null;
      set({
        errorMessage: null,
        profile: null,
        status: "idle",
      });
    },
    resetForUser: async (user, settings) => {
      const { id: currentRequestId, epoch: currentRequestEpoch } = startRequest(user, "reset");

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await resetProfile(user, settings);
        const supersedesSameUserLoad = requestId !== currentRequestId
          && requestEpoch === currentRequestEpoch
          && requestUser === user
          && requestKind === "load";
        if (requestId !== currentRequestId && !supersedesSameUserLoad) {
          return;
        }

        const currentProfile = get().profile;
        const currentResetAt = currentProfile?.uid === user.uid ? currentProfile.resetAt : null;
        const resultResetAt = profile.uid === user.uid ? profile.resetAt : null;
        const hasNewerResetBarrier = currentProfile?.uid === user.uid
          && currentResetAt !== null
          && (resultResetAt === null || Date.parse(currentResetAt) > Date.parse(resultResetAt));

        if (supersedesSameUserLoad) {
          requestId += 1;
          requestKind = "reset";
        }

        if (hasNewerResetBarrier) {
          set({ errorMessage: null, status: "ready" });
          return;
        }

        set({
          errorMessage: null,
          profile,
          status: "ready",
        });
      } catch (error) {
        if (requestId === currentRequestId) {
          set({
            errorMessage: errorMessageFor(error),
            status: "error",
          });
        }
        throw error;
      }
    },
    status: "idle",
  }));
}

export const cloudProfileStore = createCloudProfileStore();
