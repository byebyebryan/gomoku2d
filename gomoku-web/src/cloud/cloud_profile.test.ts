import { describe, expect, it } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_PROFILE_SCHEMA_VERSION,
  cloudProfileFromDocument,
  existingCloudProfileUpdate,
  newCloudProfileWrite,
  resetCloudProfileUpdate,
} from "./cloud_profile";

const authUser: CloudAuthUser = {
  avatarUrl: "https://example.com/avatar.png",
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

describe("cloudProfileFromDocument", () => {
  it("maps existing Firestore profile data and preserves app-owned fields", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        auth_providers: ["google.com", "github.com"],
        avatar_url: "https://example.com/cloud.png",
        display_name: "ByeByeBryan",
        email: "cloud@example.com",
        preferred_variant: "renju",
        username: "byebyebryan",
      }),
    ).toEqual({
      authProviders: ["google.com", "github.com"],
      avatarUrl: "https://example.com/cloud.png",
      displayName: "ByeByeBryan",
      email: "cloud@example.com",
      historyResetAt: null,
      preferredVariant: "renju",
      uid: "uid-1",
      username: "byebyebryan",
    });
  });

  it("falls back to auth user data for missing or invalid fields", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        auth_providers: [null, "google.com"],
        display_name: "",
        preferred_variant: "unknown",
      }),
    ).toMatchObject({
      authProviders: ["google.com"],
      avatarUrl: authUser.avatarUrl,
      displayName: authUser.displayName,
      email: authUser.email,
      historyResetAt: null,
      preferredVariant: "freestyle",
      username: null,
    });
  });

  it("maps Firestore reset timestamps to stable ISO strings", () => {
    expect(
      cloudProfileFromDocument(authUser, "freestyle", {
        history_reset_at: {
          nanoseconds: 123_000_000,
          seconds: 1_777_363_200,
        },
      }),
    ).toMatchObject({
      historyResetAt: "2026-04-28T08:00:00.123Z",
    });
  });
});

describe("cloud profile writes", () => {
  it("creates a complete profile document for first sign-in", () => {
    expect(newCloudProfileWrite(authUser, "renju")).toMatchObject({
      auth_providers: ["google.com"],
      avatar_url: authUser.avatarUrl,
      display_name: "Bryan",
      email: "bryan@example.com",
      history_reset_at: null,
      preferred_variant: "renju",
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      uid: "uid-1",
      username: null,
    });
  });

  it("updates provider-owned fields without overwriting app-owned display name", () => {
    expect(existingCloudProfileUpdate(authUser, "freestyle")).toMatchObject({
      auth_providers: ["google.com"],
      avatar_url: authUser.avatarUrl,
      email: "bryan@example.com",
      preferred_variant: "freestyle",
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      uid: "uid-1",
    });
    expect(existingCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("display_name");
    expect(existingCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("history_reset_at");
    expect(existingCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("username");
  });

  it("resets profile-owned fields and writes a history reset barrier", () => {
    expect(resetCloudProfileUpdate(authUser, "freestyle")).toMatchObject({
      auth_providers: ["google.com"],
      avatar_url: authUser.avatarUrl,
      display_name: authUser.displayName,
      email: authUser.email,
      preferred_variant: "freestyle",
      schema_version: CLOUD_PROFILE_SCHEMA_VERSION,
      uid: "uid-1",
    });
    expect(resetCloudProfileUpdate(authUser, "freestyle")).toHaveProperty("history_reset_at");
    expect(resetCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("username");
  });

  it("returns the refreshed provider fields after updating an existing profile", () => {
    const existing = {
      auth_providers: ["github.com"],
      avatar_url: "https://example.com/old.png",
      display_name: "ByeByeBryan",
      email: "old@example.com",
      preferred_variant: "freestyle",
      username: "byebyebryan",
    };
    const update = existingCloudProfileUpdate(authUser, "renju");

    expect(cloudProfileFromDocument(authUser, "freestyle", { ...existing, ...update })).toMatchObject({
      authProviders: ["google.com"],
      avatarUrl: authUser.avatarUrl,
      displayName: "ByeByeBryan",
      email: authUser.email,
      historyResetAt: null,
      preferredVariant: "renju",
      username: "byebyebryan",
    });
  });
});
