import { createStore, type StoreApi } from "zustand/vanilla";

import type { ProfileSettings } from "../profile/profile_settings";

import type { CloudAuthUser } from "./auth_store";
import { deleteCloudProfile, ensureCloudProfile, resetCloudProfile, type CloudProfile } from "./cloud_profile";

export type CloudProfileStatus = "idle" | "loading" | "ready" | "error";

export interface CloudProfileState {
  applyLocalPatch: (patch: Partial<CloudProfile>) => void;
  deleteForUser: (user: CloudAuthUser) => Promise<void>;
  errorMessage: string | null;
  loadForUser: (user: CloudAuthUser, settings: ProfileSettings) => Promise<void>;
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

  return createStore<CloudProfileState>((set) => ({
    applyLocalPatch: (patch) => {
      set((state) => ({
        profile: state.profile ? { ...state.profile, ...patch } : state.profile,
      }));
    },
    deleteForUser: async (user) => {
      const currentRequestId = requestId + 1;
      requestId = currentRequestId;

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
        if (requestId !== currentRequestId) {
          return;
        }

        const message = errorMessageFor(error);
        set({
          errorMessage: message,
          status: "error",
        });
        throw new Error(message);
      }
    },
    errorMessage: null,
    loadForUser: async (user, settings) => {
      const currentRequestId = requestId + 1;
      requestId = currentRequestId;

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await loadProfile(user, settings);
        if (requestId !== currentRequestId) {
          return;
        }

        set({
          errorMessage: null,
          profile,
          status: "ready",
        });
      } catch (error) {
        if (requestId !== currentRequestId) {
          return;
        }

        set({
          errorMessage: errorMessageFor(error),
          status: "error",
        });
      }
    },
    profile: null,
    reset: () => {
      requestId += 1;
      set({
        errorMessage: null,
        profile: null,
        status: "idle",
      });
    },
    resetForUser: async (user, settings) => {
      const currentRequestId = requestId + 1;
      requestId = currentRequestId;

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await resetProfile(user, settings);
        if (requestId !== currentRequestId) {
          return;
        }

        set({
          errorMessage: null,
          profile,
          status: "ready",
        });
      } catch (error) {
        if (requestId !== currentRequestId) {
          return;
        }

        const message = errorMessageFor(error);
        set({
          errorMessage: message,
          status: "error",
        });
        throw new Error(message);
      }
    },
    status: "idle",
  }));
}

export const cloudProfileStore = createCloudProfileStore();
