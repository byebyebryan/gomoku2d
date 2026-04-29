import { createStore, type StoreApi } from "zustand/vanilla";
import {
  getRedirectResult as firebaseGetRedirectResult,
  onAuthStateChanged,
  signInWithPopup as firebaseSignInWithPopup,
  signInWithRedirect as firebaseSignInWithRedirect,
  signOut as firebaseSignOut,
  type Unsubscribe,
  type User,
} from "firebase/auth";

import { getFirebaseClients, type FirebaseClients } from "./firebase";

export interface CloudAuthUserProvider {
  avatarUrl: string | null;
  displayName: string | null;
  provider: string;
}

export interface CloudAuthUser {
  avatarUrl: string | null;
  displayName: string;
  email: string | null;
  providers?: CloudAuthUserProvider[];
  providerIds: string[];
  uid: string;
}

export type CloudAuthStatus = "unconfigured" | "loading" | "signed_out" | "signed_in" | "error";

export interface CloudAuthBackend {
  completeRedirectSignIn?: () => Promise<void>;
  onAuthStateChanged: (
    onUser: (user: CloudAuthUser | null) => void,
    onError: (error: Error) => void,
  ) => Unsubscribe;
  signInWithGoogle: () => Promise<void>;
  signOut: () => Promise<void>;
}

export interface CloudAuthState {
  errorMessage: string | null;
  isConfigured: boolean;
  signInWithGoogle: () => Promise<void>;
  signOut: () => Promise<void>;
  start: () => void;
  status: CloudAuthStatus;
  stop: () => void;
  user: CloudAuthUser | null;
}

export interface CloudAuthStoreOptions {
  backend?: CloudAuthBackend | null;
  createBackend?: () => CloudAuthBackend | null;
}

export interface CloudAuthSignInEnvironment {
  embedded: boolean;
  maxTouchPoints: number;
  pointerCoarse: boolean;
  userAgentMobile: boolean;
}

export interface FirebaseAuthBackendOptions {
  getRedirectResult?: typeof firebaseGetRedirectResult;
  prefersRedirectSignIn?: () => boolean;
  popupErrorShouldFallbackToRedirect?: (error: unknown) => boolean;
  signInWithPopup?: typeof firebaseSignInWithPopup;
  signInWithRedirect?: typeof firebaseSignInWithRedirect;
}

function displayNameForUser(user: Pick<CloudAuthUser, "displayName" | "email">): string {
  return user.displayName.trim() || user.email?.split("@")[0]?.trim() || "Player";
}

export function cloudAuthUserFromFirebaseUser(user: User): CloudAuthUser {
  return {
    avatarUrl: user.photoURL,
    displayName: displayNameForUser({
      displayName: user.displayName ?? "",
      email: user.email,
    }),
    email: user.email,
    providers: user.providerData.map((provider) => ({
      avatarUrl: provider.photoURL,
      displayName: provider.displayName,
      provider: provider.providerId,
    })),
    providerIds: user.providerData.map((provider) => provider.providerId),
    uid: user.uid,
  };
}

function embeddedWindow(): boolean {
  if (typeof window === "undefined") {
    return false;
  }

  try {
    return window.self !== window.top;
  } catch {
    return true;
  }
}

export function currentSignInEnvironment(): CloudAuthSignInEnvironment {
  const navigatorLike = globalThis.navigator as (Navigator & {
    userAgentData?: { mobile?: boolean };
  }) | undefined;

  return {
    embedded: embeddedWindow(),
    maxTouchPoints: navigatorLike?.maxTouchPoints ?? 0,
    pointerCoarse: typeof window !== "undefined"
      && typeof window.matchMedia === "function"
      && window.matchMedia("(pointer: coarse)").matches,
    userAgentMobile: navigatorLike?.userAgentData?.mobile === true,
  };
}

export function shouldPreferRedirectSignIn(
  environment: CloudAuthSignInEnvironment = currentSignInEnvironment(),
): boolean {
  return environment.embedded
    || environment.userAgentMobile
    || (environment.maxTouchPoints > 0 && environment.pointerCoarse);
}

function authErrorCode(error: unknown): string | null {
  if (typeof error !== "object" || error === null || !("code" in error)) {
    return null;
  }

  const code = (error as { code?: unknown }).code;
  return typeof code === "string" ? code : null;
}

export function popupErrorShouldFallbackToRedirect(error: unknown): boolean {
  return [
    "auth/operation-not-supported-in-this-environment",
    "auth/popup-blocked",
  ].includes(authErrorCode(error) ?? "");
}

export function createFirebaseAuthBackend(
  clients: FirebaseClients,
  options: FirebaseAuthBackendOptions = {},
): CloudAuthBackend {
  const getRedirectResult = options.getRedirectResult ?? firebaseGetRedirectResult;
  const prefersRedirectSignIn = options.prefersRedirectSignIn ?? shouldPreferRedirectSignIn;
  const popupShouldFallbackToRedirect = options.popupErrorShouldFallbackToRedirect ?? popupErrorShouldFallbackToRedirect;
  const signInWithPopup = options.signInWithPopup ?? firebaseSignInWithPopup;
  const signInWithRedirect = options.signInWithRedirect ?? firebaseSignInWithRedirect;

  return {
    completeRedirectSignIn: async () => {
      await getRedirectResult(clients.auth);
    },
    onAuthStateChanged: (onUser, onError) =>
      onAuthStateChanged(
        clients.auth,
        (user) => {
          onUser(user ? cloudAuthUserFromFirebaseUser(user) : null);
        },
        onError,
      ),
    signInWithGoogle: async () => {
      if (prefersRedirectSignIn()) {
        await signInWithRedirect(clients.auth, clients.providers.google);
        return;
      }

      try {
        await signInWithPopup(clients.auth, clients.providers.google);
      } catch (error) {
        if (!popupShouldFallbackToRedirect(error)) {
          throw error;
        }

        await signInWithRedirect(clients.auth, clients.providers.google);
      }
    },
    signOut: async () => {
      await firebaseSignOut(clients.auth);
    },
  };
}

function defaultBackendFactory(): CloudAuthBackend | null {
  const clients = getFirebaseClients();
  return clients ? createFirebaseAuthBackend(clients) : null;
}

function errorMessageFor(error: unknown): string {
  return error instanceof Error ? error.message : "Cloud sign-in failed.";
}

export function createCloudAuthStore(
  options: CloudAuthStoreOptions = {},
): StoreApi<CloudAuthState> {
  let backend = options.backend ?? null;
  let unsubscribe: Unsubscribe | null = null;

  const resolveBackend = () => {
    if (backend) {
      return backend;
    }

    backend = options.createBackend?.() ?? null;
    return backend;
  };

  const store = createStore<CloudAuthState>((set, get) => ({
    errorMessage: null,
    isConfigured: false,
    signInWithGoogle: async () => {
      const activeBackend = resolveBackend();
      if (!activeBackend) {
        set({
          errorMessage: "Cloud sign-in is not configured for this build.",
          isConfigured: false,
          status: "unconfigured",
          user: null,
        });
        return;
      }

      set({ errorMessage: null, isConfigured: true });

      try {
        await activeBackend.signInWithGoogle();
      } catch (error) {
        set({
          errorMessage: errorMessageFor(error),
          status: "error",
        });
      }
    },
    signOut: async () => {
      const activeBackend = resolveBackend();
      if (!activeBackend) {
        return;
      }

      set({ errorMessage: null });

      try {
        await activeBackend.signOut();
      } catch (error) {
        set({
          errorMessage: errorMessageFor(error),
          status: "error",
        });
      }
    },
    start: () => {
      if (unsubscribe) {
        return;
      }

      const activeBackend = resolveBackend();
      if (!activeBackend) {
        set({
          errorMessage: null,
          isConfigured: false,
          status: "unconfigured",
          user: null,
        });
        return;
      }

      set({
        errorMessage: null,
        isConfigured: true,
        status: "loading",
        user: get().user,
      });

      unsubscribe = activeBackend.onAuthStateChanged(
        (user) => {
          set({
            errorMessage: null,
            isConfigured: true,
            status: user ? "signed_in" : "signed_out",
            user,
          });
        },
        (error) => {
          set({
            errorMessage: errorMessageFor(error),
            isConfigured: true,
            status: "error",
            user: null,
          });
        },
      );

      void activeBackend.completeRedirectSignIn?.().catch((error: unknown) => {
        set({
          errorMessage: errorMessageFor(error),
          isConfigured: true,
          status: "error",
          user: null,
        });
      });
    },
    status: "loading",
    stop: () => {
      unsubscribe?.();
      unsubscribe = null;
    },
    user: null,
  }));

  return store;
}

export const cloudAuthStore = createCloudAuthStore({
  createBackend: defaultBackendFactory,
});
