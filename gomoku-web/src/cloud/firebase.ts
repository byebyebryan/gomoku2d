import { initializeApp, type FirebaseApp, type FirebaseOptions } from "firebase/app";
import { getAuth, GithubAuthProvider, GoogleAuthProvider, type Auth } from "firebase/auth";
import { getFirestore, type Firestore } from "firebase/firestore";

export interface FirebaseConfigEnv {
  readonly VITE_FIREBASE_API_KEY?: string;
  readonly VITE_FIREBASE_APP_ID?: string;
  readonly VITE_FIREBASE_AUTH_DOMAIN?: string;
  readonly VITE_FIREBASE_MESSAGING_SENDER_ID?: string;
  readonly VITE_FIREBASE_PROJECT_ID?: string;
  readonly VITE_FIREBASE_STORAGE_BUCKET?: string;
}

export interface FirebaseClients {
  app: FirebaseApp;
  auth: Auth;
  firestore: Firestore;
  providers: {
    github: GithubAuthProvider;
    google: GoogleAuthProvider;
  };
}

const requiredEnvKeys: Array<keyof FirebaseConfigEnv> = [
  "VITE_FIREBASE_API_KEY",
  "VITE_FIREBASE_APP_ID",
  "VITE_FIREBASE_AUTH_DOMAIN",
  "VITE_FIREBASE_MESSAGING_SENDER_ID",
  "VITE_FIREBASE_PROJECT_ID",
  "VITE_FIREBASE_STORAGE_BUCKET",
];

let cachedClients: FirebaseClients | null = null;

function clean(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

export function firebaseConfigFromEnv(env: FirebaseConfigEnv): FirebaseOptions | null {
  const missing = requiredEnvKeys.some((key) => clean(env[key]) === undefined);
  if (missing) {
    return null;
  }

  return {
    apiKey: clean(env.VITE_FIREBASE_API_KEY),
    appId: clean(env.VITE_FIREBASE_APP_ID),
    authDomain: clean(env.VITE_FIREBASE_AUTH_DOMAIN),
    messagingSenderId: clean(env.VITE_FIREBASE_MESSAGING_SENDER_ID),
    projectId: clean(env.VITE_FIREBASE_PROJECT_ID),
    storageBucket: clean(env.VITE_FIREBASE_STORAGE_BUCKET),
  };
}

export function getFirebaseClients(): FirebaseClients | null {
  if (cachedClients) {
    return cachedClients;
  }

  const config = firebaseConfigFromEnv(import.meta.env);
  if (!config) {
    return null;
  }

  const app = initializeApp(config);
  cachedClients = {
    app,
    auth: getAuth(app),
    firestore: getFirestore(app),
    providers: {
      github: new GithubAuthProvider(),
      google: new GoogleAuthProvider(),
    },
  };

  return cachedClients;
}

export function isFirebaseConfigured(): boolean {
  return firebaseConfigFromEnv(import.meta.env) !== null;
}
