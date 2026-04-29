import { createStore, type StoreApi } from "zustand/vanilla";

import type { GameVariant } from "../core/bot_protocol";

import type { CloudAuthUser } from "./auth_store";
import { ensureCloudProfile, resetCloudProfile, type CloudProfile } from "./cloud_profile";

export type CloudProfileStatus = "idle" | "loading" | "ready" | "error";

export interface CloudProfileState {
  applyLocalPatch: (patch: Partial<CloudProfile>) => void;
  errorMessage: string | null;
  loadForUser: (user: CloudAuthUser, preferredVariant: GameVariant) => Promise<void>;
  profile: CloudProfile | null;
  reset: () => void;
  resetForUser: (user: CloudAuthUser, preferredVariant: GameVariant) => Promise<void>;
  status: CloudProfileStatus;
}

export interface CloudProfileStoreOptions {
  loadProfile?: (user: CloudAuthUser, preferredVariant: GameVariant) => Promise<CloudProfile>;
  resetProfile?: (user: CloudAuthUser, preferredVariant: GameVariant) => Promise<CloudProfile>;
}

function errorMessageFor(error: unknown): string {
  return error instanceof Error ? error.message : "Cloud profile failed to load.";
}

export function createCloudProfileStore(
  options: CloudProfileStoreOptions = {},
): StoreApi<CloudProfileState> {
  const loadProfile = options.loadProfile ?? ensureCloudProfile;
  const resetProfile = options.resetProfile ?? resetCloudProfile;
  let requestId = 0;

  return createStore<CloudProfileState>((set) => ({
    applyLocalPatch: (patch) => {
      set((state) => ({
        profile: state.profile ? { ...state.profile, ...patch } : state.profile,
      }));
    },
    errorMessage: null,
    loadForUser: async (user, preferredVariant) => {
      const currentRequestId = requestId + 1;
      requestId = currentRequestId;

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await loadProfile(user, preferredVariant);
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
    resetForUser: async (user, preferredVariant) => {
      const currentRequestId = requestId + 1;
      requestId = currentRequestId;

      set({
        errorMessage: null,
        status: "loading",
      });

      try {
        const profile = await resetProfile(user, preferredVariant);
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
