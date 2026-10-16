import { describe, expect, it } from "vitest";

import { firebaseConfigFromEnv, firebaseEmulatorsFromEnv } from "./firebase";

const emulatorEnv = {
  VITE_FIREBASE_API_KEY: "fake-key", VITE_FIREBASE_APP_ID: "fake-app",
  VITE_FIREBASE_AUTH_DOMAIN: "127.0.0.1", VITE_FIREBASE_MESSAGING_SENDER_ID: "123",
  VITE_FIREBASE_PROJECT_ID: "demo-gomoku2d-acceptance", VITE_FIREBASE_STORAGE_BUCKET: "unused",
  VITE_FIREBASE_EMULATORS: "1", VITE_FIREBASE_AUTH_EMULATOR_PORT: "19099",
  VITE_FIREBASE_FIRESTORE_EMULATOR_PORT: "18081",
};

describe("Firebase emulator isolation", () => {
  it("leaves ordinary unconfigured and production builds outside emulator mode", () => {
    expect(firebaseEmulatorsFromEnv({})).toBeNull();
    expect(firebaseEmulatorsFromEnv({ VITE_FIREBASE_PROJECT_ID: "gomoku2d" })).toBeNull();
    expect(firebaseEmulatorsFromEnv(emulatorEnv)).toEqual({ authPort: 19099, firestorePort: 18081 });
  });
  it.each([
    { VITE_FIREBASE_EMULATORS: undefined }, { VITE_FIREBASE_EMULATORS: "0" },
    { VITE_FIREBASE_PROJECT_ID: "gomoku2d" }, { VITE_FIREBASE_AUTH_DOMAIN: "gomoku2d.firebaseapp.com" },
    { VITE_FIREBASE_AUTH_EMULATOR_PORT: "http://example.com" },
    { VITE_FIREBASE_AUTH_EMULATOR_PORT: "80" }, { VITE_FIREBASE_FIRESTORE_EMULATOR_PORT: "65536" },
    { VITE_FIREBASE_AUTH_EMULATOR_PORT: "18081" }, { VITE_FIREBASE_API_KEY: "" },
  ])("rejects incomplete or mixed emulator configuration: %j", (patch) => {
    expect(() => firebaseEmulatorsFromEnv({ ...emulatorEnv, ...patch })).toThrow(/emulator mode/);
  });
});

describe("firebaseConfigFromEnv", () => {
  it("returns null until every required Vite env value is present", () => {
    expect(firebaseConfigFromEnv({})).toBeNull();
    expect(
      firebaseConfigFromEnv({
        VITE_FIREBASE_API_KEY: "api-key",
        VITE_FIREBASE_PROJECT_ID: "gomoku2d",
      }),
    ).toBeNull();
  });

  it("builds Firebase config from trimmed Vite env values", () => {
    expect(
      firebaseConfigFromEnv({
        VITE_FIREBASE_API_KEY: " api-key ",
        VITE_FIREBASE_APP_ID: " app-id ",
        VITE_FIREBASE_AUTH_DOMAIN: " gomoku2d.firebaseapp.com ",
        VITE_FIREBASE_MESSAGING_SENDER_ID: " 892554744656 ",
        VITE_FIREBASE_PROJECT_ID: " gomoku2d ",
        VITE_FIREBASE_STORAGE_BUCKET: " gomoku2d.firebasestorage.app ",
      }),
    ).toEqual({
      apiKey: "api-key",
      appId: "app-id",
      authDomain: "gomoku2d.firebaseapp.com",
      messagingSenderId: "892554744656",
      projectId: "gomoku2d",
      storageBucket: "gomoku2d.firebasestorage.app",
    });
  });
});
