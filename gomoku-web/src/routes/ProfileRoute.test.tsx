import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import type { CloudAuthUser } from "../cloud/auth_store";
import { cloudAuthStore } from "../cloud/auth_store";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { createCloudDirectSavedMatch } from "../cloud/cloud_match";
import type { CloudProfile } from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import { createLocalSavedMatch } from "../match/saved_match";
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
  historyResetAt: null,
  preferredVariant: "freestyle",
  uid: "uid-1",
  username: null,
};

const initialCloudAuthState = cloudAuthStore.getState();
const initialCloudHistoryState = cloudHistoryStore.getState();
const initialCloudProfileState = cloudProfileStore.getState();
const initialCloudPromotionState = cloudPromotionStore.getState();
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
    cloudHistoryStore.setState(initialCloudHistoryState, true);
    cloudProfileStore.setState(initialCloudProfileState, true);
    cloudPromotionStore.setState(initialCloudPromotionState, true);
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
      resetForUser: vi.fn(),
      status: "idle",
    });
    cloudHistoryStore.setState({
      clearForUser: vi.fn().mockResolvedValue(undefined),
      errorMessage: null,
      loadForUser: vi.fn().mockResolvedValue(undefined),
      loadStatus: "idle",
      resetUserCache: vi.fn(),
      syncMatchForUser: vi.fn().mockResolvedValue(undefined),
      syncPendingForUser: vi.fn().mockResolvedValue(undefined),
      syncStatus: "idle",
      users: {},
    });
    cloudPromotionStore.setState({
      errorMessage: null,
      promote: vi.fn(),
      reset: vi.fn(),
      result: null,
      status: "idle",
    });
  });

  it("keeps local profile usable when cloud auth is not configured", () => {
    renderProfileRoute();

    expect(screen.getByRole("heading", { name: "Profile" })).toBeInTheDocument();
    expect(screen.getByText("Local profile")).toBeInTheDocument();
    expect(screen.getByText("Online features disabled.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign in" })).toBeDisabled();
    expect(screen.getByText("Default rule")).toBeInTheDocument();
    expect(screen.getByLabelText("Display name")).toHaveValue("Guest");
  });

  it("renders canonical local saved-match history", () => {
    guestProfileStore.getState().ensureGuestProfile();
    guestProfileStore.getState().recordFinishedMatch({
      mode: "bot",
      moves: [
        { col: 5, moveNumber: 1, player: 1, row: 7 },
        { col: 0, moveNumber: 2, player: 2, row: 0 },
        { col: 6, moveNumber: 3, player: 1, row: 7 },
        { col: 1, moveNumber: 4, player: 2, row: 0 },
        { col: 7, moveNumber: 5, player: 1, row: 7 },
        { col: 2, moveNumber: 6, player: 2, row: 0 },
        { col: 8, moveNumber: 7, player: 1, row: 7 },
        { col: 3, moveNumber: 8, player: 2, row: 0 },
        { col: 9, moveNumber: 9, player: 1, row: 7 },
      ],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      status: "black_won",
      variant: "freestyle",
    });

    renderProfileRoute();

    expect(screen.getByText("Win")).toBeInTheDocument();
    expect(screen.getByText("vs Practice Bot")).toBeInTheDocument();
    expect(screen.getByText("Black")).toBeInTheDocument();
    expect(screen.getByText("Moves 9")).toBeInTheDocument();
  });

  it("confirms before resetting the local-only profile", () => {
    guestProfileStore.getState().ensureGuestProfile();
    guestProfileStore.getState().renameDisplayName("ByeByeBryan");

    renderProfileRoute();

    expect(screen.getByLabelText("Display name")).toHaveValue("ByeByeBryan");
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    expect(screen.getByText("This clears the local profile and match history on this device.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByLabelText("Display name")).toHaveValue("ByeByeBryan");

    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm reset" }));

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
      resetForUser: vi.fn(),
      status: "ready",
    });

    renderProfileRoute();

    expect(screen.getByText("Signed in as Bryan")).toBeInTheDocument();
    expect(screen.queryByText("uid-1")).not.toBeInTheDocument();
    expect(screen.getByText("Cloud history enabled.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Sign out" }));
    expect(signOut).toHaveBeenCalledTimes(1);
  });

  it("shows compact cloud history status when signed-in history has loaded", () => {
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      loadStatus: "ready",
      users: {
        [cloudUser.uid]: {
          cachedMatches: [],
          loadedAt: "2026-04-28T01:01:00.000Z",
          pendingMatches: {},
          sync: {},
        },
      },
    });

    renderProfileRoute();

    expect(screen.getByText("Synced")).toBeInTheDocument();
    expect(screen.queryByText("Finished matches are saved here.")).not.toBeInTheDocument();
  });

  it("shows retrying when pending cloud history sync has failed", () => {
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: guestProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      variant: "freestyle",
    });

    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      loadStatus: "ready",
      users: {
        [cloudUser.uid]: {
          cachedMatches: [],
          loadedAt: "2026-04-28T01:01:00.000Z",
          pendingMatches: { [localMatch.id]: localMatch },
          sync: {
            [localMatch.id]: {
              errorMessage: "network failed",
              matchId: localMatch.id,
              status: "error",
              updatedAt: "2026-04-28T01:02:00.000Z",
            },
          },
        },
      },
    });

    renderProfileRoute();

    expect(screen.getByText("Retrying")).toBeInTheDocument();
  });

  it("does not show history retrying for a non-history sync error with no pending matches", () => {
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      errorMessage: "permission denied",
      loadStatus: "ready",
      syncStatus: "error",
      users: {
        [cloudUser.uid]: {
          cachedMatches: [],
          loadedAt: "2026-04-28T01:01:00.000Z",
          pendingMatches: {},
          sync: {},
        },
      },
    });

    renderProfileRoute();

    expect(screen.getByText("Synced")).toBeInTheDocument();
    expect(screen.queryByText("Retrying")).not.toBeInTheDocument();
  });

  it("shows cloud history loading and unavailable badge states", () => {
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      loadStatus: "loading",
    });

    renderProfileRoute();

    expect(screen.getByText("Loading")).toBeInTheDocument();
    expect(screen.queryByText("Finished matches are saved here.")).not.toBeInTheDocument();

    cleanup();
    cloudHistoryStore.setState({
      errorMessage: "network failed",
      loadStatus: "error",
      users: {},
    });

    renderProfileRoute();

    expect(screen.getByText("Retrying")).toBeInTheDocument();
    expect(screen.queryByText("Finished matches are saved here.")).not.toBeInTheDocument();
  });

  it("retries cloud history load and pending sync when the browser comes online", async () => {
    const loadForUser = vi.fn().mockResolvedValue(undefined);
    const syncPendingForUser = vi.fn().mockResolvedValue(undefined);
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      loadForUser,
      syncPendingForUser,
    });

    renderProfileRoute();

    await waitFor(() => {
      expect(loadForUser).toHaveBeenCalledTimes(1);
    });

    window.dispatchEvent(new Event("online"));

    await waitFor(() => {
      expect(loadForUser).toHaveBeenCalledTimes(2);
    });
    expect(syncPendingForUser).toHaveBeenCalledTimes(2);
  });

  it("uses signed-in loading copy while the cloud profile is loading", () => {
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
      start: vi.fn(),
      status: "signed_in",
      stop: vi.fn(),
      user: cloudUser,
    });
    cloudProfileStore.setState({
      errorMessage: null,
      loadForUser: vi.fn(),
      profile: null,
      reset: vi.fn(),
      resetForUser: vi.fn(),
      status: "loading",
    });

    renderProfileRoute();

    expect(screen.getByText("Signed in")).toBeInTheDocument();
    expect(screen.getByText("Loading cloud profile...")).toBeInTheDocument();
    expect(screen.queryByText("Sign in for cloud history."))
      .not
      .toBeInTheDocument();
  });

  it("shows merged cloud and local history without duplicate direct cloud saves", () => {
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: guestProfile.id,
      moves: [
        { col: 5, moveNumber: 1, player: 1, row: 7 },
        { col: 0, moveNumber: 2, player: 2, row: 0 },
        { col: 6, moveNumber: 3, player: 1, row: 7 },
        { col: 1, moveNumber: 4, player: 2, row: 0 },
        { col: 7, moveNumber: 5, player: 1, row: 7 },
      ],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "black_won",
      variant: "freestyle",
    });
    const cloudMatch = createCloudDirectSavedMatch(cloudUser, localMatch);

    guestProfileStore.setState({ history: [localMatch] });
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      users: {
        [cloudUser.uid]: {
          cachedMatches: [cloudMatch],
          loadedAt: "2026-04-28T01:01:00.000Z",
          pendingMatches: {},
          sync: {},
        },
      },
    });

    renderProfileRoute();

    expect(screen.getByText("Win")).toBeInTheDocument();
    expect(screen.getByText("Moves 5")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Replay" })).toHaveLength(1);
  });

  it("resets cloud profile scope and local cache when signed in", async () => {
    const resetForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockResolvedValue(undefined);
    const resetUserCache = vi.fn();
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: guestProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      variant: "freestyle",
    });

    guestProfileStore.setState({ history: [localMatch] });
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser,
      status: "ready",
    });
    cloudHistoryStore.setState({
      clearForUser,
      resetUserCache,
    });

    renderProfileRoute();

    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    expect(
      screen.getByText("This clears cloud history, resets cloud profile values, and clears this device's local cache."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Confirm reset" }));

    await waitFor(() => {
      expect(resetForUser).toHaveBeenCalledWith(cloudUser, "freestyle");
    });
    expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    expect(resetUserCache).toHaveBeenCalledWith("uid-1");
    expect(guestProfileStore.getState().history).toEqual([]);
  });

  it("keeps signed-in reset confirmation open and local history intact when cloud clear fails", async () => {
    const resetForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockRejectedValue(new Error("permission denied"));
    const resetUserCache = vi.fn();
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: guestProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      variant: "freestyle",
    });

    guestProfileStore.setState({ history: [localMatch] });
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser,
      status: "ready",
    });
    cloudHistoryStore.setState({
      clearForUser,
      resetUserCache,
    });

    renderProfileRoute();

    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm reset" }));

    await waitFor(() => {
      expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    });
    expect(screen.getByRole("button", { name: "Confirm reset" })).toBeInTheDocument();
    expect(resetUserCache).not.toHaveBeenCalled();
    expect(guestProfileStore.getState().history).toEqual([localMatch]);
  });

  it("starts background guest promotion after cloud profile loads", async () => {
    const promote = vi.fn().mockResolvedValue(undefined);
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    guestProfileStore.getState().renameDisplayName("ByeByeBryan");
    const history = guestProfileStore.getState().history;
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudPromotionStore.setState({
      errorMessage: null,
      promote,
      reset: vi.fn(),
      result: null,
      status: "idle",
    });

    renderProfileRoute();

    await waitFor(() => {
      expect(promote).toHaveBeenCalledWith({
        cloudDisplayName: cloudProfile.displayName,
        guestHistory: history,
        guestProfile: expect.objectContaining({
          displayName: "ByeByeBryan",
          id: guestProfile.id,
        }),
        historyResetAt: null,
        settings: { preferredVariant: "freestyle" },
        user: cloudUser,
      });
    });
  });

  it("adopts the cloud display name before promoting a default local guest name", async () => {
    const promote = vi.fn().mockResolvedValue(undefined);
    const guestProfile = guestProfileStore.getState().ensureGuestProfile();
    cloudAuthStore.setState({
      errorMessage: null,
      isConfigured: true,
      signInWithGoogle: vi.fn(),
      signOut: vi.fn(),
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
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudPromotionStore.setState({
      errorMessage: null,
      promote,
      reset: vi.fn(),
      result: null,
      status: "idle",
    });

    renderProfileRoute();

    await waitFor(() => {
      expect(screen.getByLabelText("Display name")).toHaveValue("Bryan");
    });
    await waitFor(() => {
      expect(promote).toHaveBeenCalledWith({
        cloudDisplayName: cloudProfile.displayName,
        guestHistory: [],
        guestProfile: expect.objectContaining({
          displayName: "Bryan",
          id: guestProfile.id,
        }),
        historyResetAt: null,
        settings: { preferredVariant: "freestyle" },
        user: cloudUser,
      });
    });
    expect(promote).toHaveBeenCalledTimes(1);
  });
});
