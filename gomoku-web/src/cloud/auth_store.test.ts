import { describe, expect, it, vi } from "vitest";

import type { CloudAuthBackend, CloudAuthUser, FirebaseAuthBackendOptions } from "./auth_store";
import {
  createCloudAuthStore,
  createFirebaseAuthBackend,
  popupErrorShouldFallbackToRedirect,
  shouldPreferRedirectSignIn,
} from "./auth_store";
import type { FirebaseClients } from "./firebase";

const bryanUser: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

function createFakeBackend(): CloudAuthBackend & {
  completeRedirectSignIn: ReturnType<typeof vi.fn>;
  emitError: (error: Error) => void;
  emitUser: (user: CloudAuthUser | null) => void;
  unsubscribe: ReturnType<typeof vi.fn>;
} {
  let onUser: ((user: CloudAuthUser | null) => void) | null = null;
  let onError: ((error: Error) => void) | null = null;
  const unsubscribe = vi.fn();

  return {
    completeRedirectSignIn: vi.fn().mockResolvedValue(undefined),
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

function fakeFirebaseClients(): FirebaseClients {
  return {
    app: {},
    auth: {},
    firestore: {},
    providers: {
      github: {},
      google: {},
    },
  } as FirebaseClients;
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
    expect(backend.completeRedirectSignIn).toHaveBeenCalledTimes(1);

    store.getState().stop();
    expect(backend.unsubscribe).toHaveBeenCalledTimes(1);
  });

  it("surfaces redirect sign-in completion errors", async () => {
    const backend = createFakeBackend();
    backend.completeRedirectSignIn.mockRejectedValueOnce(new Error("redirect failed"));
    const store = createCloudAuthStore({ backend });

    store.getState().start();
    await Promise.resolve();

    expect(store.getState()).toMatchObject({
      errorMessage: "redirect failed",
      status: "error",
      user: null,
    });
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

describe("Firebase auth sign-in strategy", () => {
  it("prefers redirect sign-in for embedded, mobile, or coarse touch environments", () => {
    expect(shouldPreferRedirectSignIn({
      embedded: true,
      maxTouchPoints: 0,
      pointerCoarse: false,
      userAgentMobile: false,
    })).toBe(true);
    expect(shouldPreferRedirectSignIn({
      embedded: false,
      maxTouchPoints: 0,
      pointerCoarse: false,
      userAgentMobile: true,
    })).toBe(true);
    expect(shouldPreferRedirectSignIn({
      embedded: false,
      maxTouchPoints: 2,
      pointerCoarse: true,
      userAgentMobile: false,
    })).toBe(true);
    expect(shouldPreferRedirectSignIn({
      embedded: false,
      maxTouchPoints: 0,
      pointerCoarse: false,
      userAgentMobile: false,
    })).toBe(false);
  });

  it("falls back to redirect only for popup-specific failures", () => {
    expect(popupErrorShouldFallbackToRedirect({ code: "auth/popup-blocked" })).toBe(true);
    expect(popupErrorShouldFallbackToRedirect({
      code: "auth/operation-not-supported-in-this-environment",
    })).toBe(true);
    expect(popupErrorShouldFallbackToRedirect({ code: "auth/cancelled-popup-request" })).toBe(false);
    expect(popupErrorShouldFallbackToRedirect({ code: "auth/popup-closed-by-user" })).toBe(false);
    expect(popupErrorShouldFallbackToRedirect(new Error("network failed"))).toBe(false);
  });

  it("uses redirect directly when the environment prefers redirect", async () => {
    const signInWithPopup = vi.fn().mockResolvedValue(undefined);
    const signInWithRedirect = vi.fn().mockResolvedValue(undefined);
    const backend = createFirebaseAuthBackend(fakeFirebaseClients(), {
      getRedirectResult: vi.fn().mockResolvedValue(null) as FirebaseAuthBackendOptions["getRedirectResult"],
      prefersRedirectSignIn: () => true,
      signInWithPopup: signInWithPopup as FirebaseAuthBackendOptions["signInWithPopup"],
      signInWithRedirect: signInWithRedirect as FirebaseAuthBackendOptions["signInWithRedirect"],
    });

    await backend.signInWithGoogle();

    expect(signInWithPopup).not.toHaveBeenCalled();
    expect(signInWithRedirect).toHaveBeenCalledTimes(1);
  });

  it("falls back to redirect when popup is blocked", async () => {
    const signInWithPopup = vi.fn().mockRejectedValue({ code: "auth/popup-blocked" });
    const signInWithRedirect = vi.fn().mockResolvedValue(undefined);
    const backend = createFirebaseAuthBackend(fakeFirebaseClients(), {
      getRedirectResult: vi.fn().mockResolvedValue(null) as FirebaseAuthBackendOptions["getRedirectResult"],
      prefersRedirectSignIn: () => false,
      signInWithPopup: signInWithPopup as FirebaseAuthBackendOptions["signInWithPopup"],
      signInWithRedirect: signInWithRedirect as FirebaseAuthBackendOptions["signInWithRedirect"],
    });

    await backend.signInWithGoogle();

    expect(signInWithPopup).toHaveBeenCalledTimes(1);
    expect(signInWithRedirect).toHaveBeenCalledTimes(1);
  });

  it("does not redirect after an intentional popup close", async () => {
    const popupError = { code: "auth/popup-closed-by-user" };
    const signInWithPopup = vi.fn().mockRejectedValue(popupError);
    const signInWithRedirect = vi.fn().mockResolvedValue(undefined);
    const backend = createFirebaseAuthBackend(fakeFirebaseClients(), {
      getRedirectResult: vi.fn().mockResolvedValue(null) as FirebaseAuthBackendOptions["getRedirectResult"],
      prefersRedirectSignIn: () => false,
      signInWithPopup: signInWithPopup as FirebaseAuthBackendOptions["signInWithPopup"],
      signInWithRedirect: signInWithRedirect as FirebaseAuthBackendOptions["signInWithRedirect"],
    });

    await expect(backend.signInWithGoogle()).rejects.toBe(popupError);

    expect(signInWithPopup).toHaveBeenCalledTimes(1);
    expect(signInWithRedirect).not.toHaveBeenCalled();
  });
});
