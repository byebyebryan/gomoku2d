import { createStore, type StoreApi } from "zustand/vanilla";

import {
  promoteGuestToCloud,
  promotionInputKey,
  type GuestPromotionInput,
  type GuestPromotionResult,
} from "./cloud_promotion";

export type CloudPromotionStatus = "idle" | "promoting" | "complete" | "error";

export interface CloudPromotionState {
  errorMessage: string | null;
  promote: (input: GuestPromotionInput) => Promise<void>;
  reset: () => void;
  result: GuestPromotionResult | null;
  status: CloudPromotionStatus;
}

export interface CloudPromotionStoreOptions {
  promoteGuest?: (input: GuestPromotionInput) => Promise<GuestPromotionResult>;
}

function errorMessageFor(error: unknown): string {
  return error instanceof Error ? error.message : "Cloud promotion failed.";
}

export function createCloudPromotionStore(
  options: CloudPromotionStoreOptions = {},
): StoreApi<CloudPromotionState> {
  const promoteGuest = options.promoteGuest ?? promoteGuestToCloud;
  let requestId = 0;
  let completedInputKey: string | null = null;
  let activeInputKey: string | null = null;
  let activePromise: Promise<void> | null = null;

  return createStore<CloudPromotionState>((set) => ({
    errorMessage: null,
    promote: async (input) => {
      const inputKey = promotionInputKey(input);
      if (completedInputKey === inputKey) {
        return;
      }
      if (activeInputKey === inputKey && activePromise) {
        return activePromise;
      }

      const currentRequestId = requestId + 1;
      requestId = currentRequestId;
      activeInputKey = inputKey;

      set({
        errorMessage: null,
        result: null,
        status: "promoting",
      });

      const promotionPromise = (async () => {
        try {
          const result = await promoteGuest(input);
          if (requestId !== currentRequestId) {
            return;
          }

          completedInputKey = inputKey;
          set({
            errorMessage: null,
            result,
            status: "complete",
          });
        } catch (error) {
          if (requestId !== currentRequestId) {
            return;
          }

          set({
            errorMessage: errorMessageFor(error),
            result: null,
            status: "error",
          });
        } finally {
          if (activeInputKey === inputKey && requestId === currentRequestId) {
            activeInputKey = null;
            activePromise = null;
          }
        }
      })();

      activePromise = promotionPromise;
      return promotionPromise;
    },
    reset: () => {
      requestId += 1;
      activeInputKey = null;
      activePromise = null;
      completedInputKey = null;
      set({
        errorMessage: null,
        result: null,
        status: "idle",
      });
    },
    result: null,
    status: "idle",
  }));
}

export const cloudPromotionStore = createCloudPromotionStore();
