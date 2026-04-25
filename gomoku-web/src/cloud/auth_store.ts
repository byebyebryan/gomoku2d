import { createStore, type StoreApi } from "zustand/vanilla";
import {
  onAuthStateChanged,
  signInWithPopup,
  signOut as firebaseSignOut,
  type Unsubscribe,
  type User,
} from "firebase/auth";

import { getFirebaseClients, type FirebaseClients } from "./firebase";

export interface CloudAuthUser {
  avatarUrl: string | null;
  displayName: string;
  email: string | null;
  providerIds: string[];
  uid: string;
}

export type CloudAuthStatus = "unconfigured" | "loading" | "signed_out" | "signed_in" | "error";

export interface CloudAuthBackend {
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
    providerIds: user.providerData.map((provider) => provider.providerId),
    uid: user.uid,
  };
}

function createFirebaseAuthBackend(clients: FirebaseClients): CloudAuthBackend {
  return {
    onAuthStateChanged: (onUser, onError) =>
      onAuthStateChanged(
        clients.auth,
        (user) => {
          onUser(user ? cloudAuthUserFromFirebaseUser(user) : null);
        },
        onError,
      ),
    signInWithGoogle: async () => {
      await signInWithPopup(clients.auth, clients.providers.google);
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
