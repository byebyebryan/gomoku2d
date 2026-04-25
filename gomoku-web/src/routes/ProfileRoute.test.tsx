import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import type { CloudAuthUser } from "../cloud/auth_store";
import { cloudAuthStore } from "../cloud/auth_store";
import type { CloudProfile } from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { guestProfileStore } from "../profile/guest_profile_store";

import { ProfileRoute } from "./ProfileRoute";

const cloudUser: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const cloudProfile: CloudProfile = {
  authProviders: ["google.com"],
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  preferredVariant: "freestyle",
  uid: "uid-1",
  username: null,
};

const initialCloudAuthState = cloudAuthStore.getState();
const initialCloudProfileState = cloudProfileStore.getState();
const initialGuestProfileState = guestProfileStore.getState();

function renderProfileRoute() {
  render(
    <MemoryRouter>
      <ProfileRoute />
    </MemoryRouter>,
  );
}

describe("ProfileRoute cloud state", () => {
  afterEach(() => {
    cleanup();
    cloudAuthStore.setState(initialCloudAuthState, true);
    cloudProfileStore.setState(initialCloudProfileState, true);
    guestProfileStore.setState(initialGuestProfileState, true);
  });

  beforeEach(() => {
    guestProfileStore.setState({
      history: [],
      profile: null,
      settings: { preferredVariant: "freestyle" },
    });
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: false,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
      start: vi.fn(),
      status: "unconfigured",
      stop: vi.fn(),
      user: null,
    });
    cloudProfileStore.setState({
      errorMessage: null,
      loadForUser: vi.fn(),
      profile: null,
      reset: vi.fn(),
      status: "idle",
    });
  });

  it("keeps local profile usable when cloud auth is not configured", () => {
    renderProfileRoute();

    expect(screen.getByRole("heading", { name: "Profile" })).toBeInTheDocument();
    expect(screen.getByText("Cloud sign-in is not configured for this build.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign in with Google" })).toBeDisabled();
    expect(screen.getByText("Default rule")).toBeInTheDocument();
    expect(screen.getByLabelText("Display name")).toHaveValue("Guest");
  });

  it("shows the linked cloud profile and allows sign-out", () => {
    const signOut = vi.fn().mockResolvedValue(undefined);
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut,
      start: vi.fn(),
      status: "signed_in",
      stop: vi.fn(),
      user: cloudUser,
    });
    cloudProfileStore.setState({
      errorMessage: null,
      loadForUser: vi.fn(),
      profile: cloudProfile,
      reset: vi.fn(),
      status: "ready",
    });

    renderProfileRoute();

    expect(screen.getByText("Signed in as Bryan")).toBeInTheDocument();
    expect(screen.getByText("uid-1")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Sign out" }));
    expect(signOut).toHaveBeenCalledTimes(1);
  });
});
