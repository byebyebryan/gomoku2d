import { describe, expect, it } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import {
  cloudProfileFromDocument,
  existingCloudProfileUpdate,
  newCloudProfileWrite,
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
      preferredVariant: "freestyle",
      username: null,
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
      preferred_variant: "renju",
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
      uid: "uid-1",
    });
    expect(existingCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("display_name");
    expect(existingCloudProfileUpdate(authUser, "freestyle")).not.toHaveProperty("username");
  });
});
