import { describe, expect, it } from "vitest";

import { firebaseConfigFromEnv } from "./firebase";

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
