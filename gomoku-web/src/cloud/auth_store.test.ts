import { describe, expect, it, vi } from "vitest";

import type { CloudAuthBackend, CloudAuthUser } from "./auth_store";
import { createCloudAuthStore } from "./auth_store";

const bryanUser: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

function createFakeBackend(): CloudAuthBackend & {
  emitError: (error: Error) => void;
  emitUser: (user: CloudAuthUser | null) => void;
  unsubscribe: ReturnType<typeof vi.fn>;
} {
  let onUser: ((user: CloudAuthUser | null) => void) | null = null;
  let onError: ((error: Error) => void) | null = null;
  const unsubscribe = vi.fn();

  return {
    emitError: (error) => {
      onError?.(error);
    },
    emitUser: (user) => {
      onUser?.(user);
    },
    onAuthStateChanged: (nextUser, nextError) => {
      onUser = nextUser;
      onError = nextError;
      return unsubscribe;
    },
    signInWithGoogle: vi.fn().mockResolvedValue(undefined),
    signOut: vi.fn().mockResolvedValue(undefined),
    unsubscribe,
  };
}

describe("createCloudAuthStore", () => {
  it("stays unconfigured when no Firebase backend is available", async () => {
    const store = createCloudAuthStore({ createBackend: () => null });

    store.getState().start();
    expect(store.getState()).toMatchObject({
      isConfigured: false,
      status: "unconfigured",
      user: null,
    });

    await store.getState().signInWithGoogle();
    expect(store.getState()).toMatchObject({
      errorMessage: "Cloud sign-in is not configured for this build.",
      status: "unconfigured",
    });
  });

  it("subscribes to auth state and signs in/out through the backend", async () => {
    const backend = createFakeBackend();
    const store = createCloudAuthStore({ backend });

    store.getState().start();
    expect(store.getState()).toMatchObject({
      isConfigured: true,
      status: "loading",
    });

    backend.emitUser(null);
    expect(store.getState()).toMatchObject({
      status: "signed_out",
      user: null,
    });

    backend.emitUser(bryanUser);
    expect(store.getState()).toMatchObject({
      status: "signed_in",
      user: bryanUser,
    });

    await store.getState().signInWithGoogle();
    await store.getState().signOut();
    expect(backend.signInWithGoogle).toHaveBeenCalledTimes(1);
    expect(backend.signOut).toHaveBeenCalledTimes(1);

    store.getState().stop();
    expect(backend.unsubscribe).toHaveBeenCalledTimes(1);
  });

  it("surfaces backend auth errors", () => {
    const backend = createFakeBackend();
    const store = createCloudAuthStore({ backend });

    store.getState().start();
    backend.emitError(new Error("popup blocked"));

    expect(store.getState()).toMatchObject({
      errorMessage: "popup blocked",
      status: "error",
      user: null,
    });
  });
});
