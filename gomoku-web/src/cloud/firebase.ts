import { initializeApp, type FirebaseApp, type FirebaseOptions } from "firebase/app";
import { connectAuthEmulator, getAuth, GithubAuthProvider, GoogleAuthProvider, type Auth } from "firebase/auth";
import { connectFirestoreEmulator, getFirestore, type Firestore } from "firebase/firestore";

export interface FirebaseConfigEnv {
  readonly VITE_FIREBASE_API_KEY?: string;
  readonly VITE_FIREBASE_APP_ID?: string;
  readonly VITE_FIREBASE_AUTH_DOMAIN?: string;
  readonly VITE_FIREBASE_MESSAGING_SENDER_ID?: string;
  readonly VITE_FIREBASE_PROJECT_ID?: string;
  readonly VITE_FIREBASE_STORAGE_BUCKET?: string;
  readonly VITE_FIREBASE_EMULATORS?: string;
  readonly VITE_FIREBASE_AUTH_EMULATOR_PORT?: string;
  readonly VITE_FIREBASE_FIRESTORE_EMULATOR_PORT?: string;
}

export interface FirebaseClients {
  app: FirebaseApp;
  auth: Auth;
  authDomain: string;
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

export function firebaseEmulatorsFromEnv(env: FirebaseConfigEnv): {
  authPort: number;
  firestorePort: number;
} | null {
  const values = [env.VITE_FIREBASE_EMULATORS, env.VITE_FIREBASE_AUTH_EMULATOR_PORT,
    env.VITE_FIREBASE_FIRESTORE_EMULATOR_PORT].map(clean);
  if (values.every((value) => value === undefined)) return null;
  const validPort = (value: string | undefined) => value !== undefined
    && /^\d+$/.test(value) && Number(value) >= 1024 && Number(value) <= 65535;
  if (values[0] !== "1" || !validPort(values[1]) || !validPort(values[2])
    || values[1] === values[2] || !/^demo-[a-z0-9-]+$/.test(clean(env.VITE_FIREBASE_PROJECT_ID) ?? "")
    || !["localhost", "127.0.0.1"].includes(clean(env.VITE_FIREBASE_AUTH_DOMAIN) ?? "")
    || firebaseConfigFromEnv(env) === null) {
    throw new Error("Firebase emulator mode requires complete loopback configuration and a demo- project.");
  }
  return { authPort: Number(values[1]), firestorePort: Number(values[2]) };
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

  const emulators = firebaseEmulatorsFromEnv(import.meta.env);
  if (emulators && !["localhost", "127.0.0.1", "[::1]"].includes(globalThis.location?.hostname)) {
    throw new Error("Firebase emulator builds can only run on loopback origins.");
  }
  const config = firebaseConfigFromEnv(import.meta.env);
  if (!config) {
    return null;
  }

  const app = initializeApp(config);
  const auth = getAuth(app);
  const firestore = getFirestore(app);
  if (emulators) {
    connectAuthEmulator(auth, `http://127.0.0.1:${emulators.authPort}`, { disableWarnings: true });
    connectFirestoreEmulator(firestore, "127.0.0.1", emulators.firestorePort);
  }
  cachedClients = {
    app,
    auth,
    authDomain: config.authDomain ?? "",
    firestore,
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
