import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import type { CloudAuthUser } from "../cloud/auth_store";
import { cloudAuthStore } from "../cloud/auth_store";
import { CloudSessionController } from "../cloud/CloudSessionController";
import { cloudHistoryStore } from "../cloud/cloud_history_store";
import { createCloudSavedMatch } from "../cloud/cloud_match";
import { emptyCloudMatchHistory, type CloudProfile } from "../cloud/cloud_profile";
import { cloudProfileStore } from "../cloud/cloud_profile_store";
import { cloudPromotionStore } from "../cloud/cloud_promotion_store";
import { createLocalSavedMatch } from "../match/saved_match";
import { emptyLocalMatchHistory, localProfileStore, type LocalProfileSavedMatch } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import {
  readReplayAnalysisCache,
  writeReplayAnalysisCache,
} from "../replay/replay_analysis_cache";
import { replayLibraryStore } from "../replay/replay_library_store";
import { replayLibrarySync } from "../replay/replay_library_sync";

import { ProfileRoute } from "./ProfileRoute";

const sharingMocks = vi.hoisted(() => ({
  clearSharingData: vi.fn(),
  loadPrivateLibrary: vi.fn(),
  savePrivateLibrary: vi.fn(),
}));

vi.mock("../cloud/sharing_lifecycle", () => ({
  clearSharingData: sharingMocks.clearSharingData,
}));

vi.mock("../cloud/sharing_private", () => ({
  loadPrivateLibrary: sharingMocks.loadPrivateLibrary,
  savePrivateLibrary: sharingMocks.savePrivateLibrary,
}));

const cloudUser: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const defaultSettings = createDefaultProfileSettings();

const cloudProfile: CloudProfile = {
  auth: {
    providers: [
      {
        avatarUrl: null,
        displayName: "Bryan",
        provider: "google.com",
      },
    ],
  },
  createdAt: null,
  displayName: "Bryan",
  matchHistory: emptyCloudMatchHistory(),
  resetAt: null,
  settings: defaultSettings,
  uid: "uid-1",
  updatedAt: null,
  username: null,
};

const initialCloudAuthState = cloudAuthStore.getState();
const initialCloudHistoryState = cloudHistoryStore.getState();
const initialCloudProfileState = cloudProfileStore.getState();
const initialCloudPromotionState = cloudPromotionStore.getState();
const initialLocalProfileState = localProfileStore.getState();

function renderProfileRoute() {
  render(
    <MemoryRouter>
      <CloudSessionController />
      <ProfileRoute />
    </MemoryRouter>,
  );
}

function localMatchHistoryWith(matches: LocalProfileSavedMatch[] = []) {
  return {
    ...emptyLocalMatchHistory(),
    replayMatches: matches,
  };
}

function localSavedMatch(id: string, localProfileId: string, minuteOffset = 0): LocalProfileSavedMatch {
  return createLocalSavedMatch({
    id,
    localProfileId,
    moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
    players: [
      { kind: "human", name: "Guest", stone: "black" },
      { kind: "bot", name: "Practice Bot", stone: "white" },
    ],
    savedAt: new Date(Date.UTC(2026, 3, 28, 1, minuteOffset, 0)).toISOString(),
    status: "draw",
    ruleset: "freestyle",
  });
}

const replayAnalysisOptions = { maxDepth: 4, maxScanPlies: 64 };

function cacheReplayAnalysis(match: LocalProfileSavedMatch): void {
  writeReplayAnalysisCache(match, replayAnalysisOptions, {
    annotationsByPly: {},
    step: {
      analysis: { schema_version: 1 },
      annotations: [],
      counters: { branch_roots: 0, prefixes_analyzed: 0, proof_nodes: 0 },
      current_ply: null,
      done: true,
      error: null,
      schema_version: 1,
      status: "resolved",
    },
  });
}

describe("ProfileRoute cloud state", () => {
  it.each(["reset", "delete"] as const)("does not let a delayed %s affect a later account", async (action) => {
    let finish!: () => void;
    const pending = new Promise<void>((resolve) => { finish = resolve; });
    const operation = vi.fn(() => pending);
    const signOut = vi.fn().mockResolvedValue(undefined);
    localProfileStore.getState().ensureLocalProfile();
    localProfileStore.getState().renameDisplayName("Local player");
    cloudAuthStore.setState({ status: "signed_in", isConfigured: true, user: cloudUser, signOut });
    cloudProfileStore.setState({
      profile: cloudProfile,
      status: "ready",
      ...(action === "reset" ? { resetForUser: operation } : { deleteForUser: operation }),
    });
    renderProfileRoute();
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    if (action === "delete") fireEvent.click(screen.getByRole("button", { name: "Delete Cloud" }));
    fireEvent.click(screen.getByRole("button", { name: action === "reset" ? "Reset" : "Delete" }));
    await waitFor(() => expect(operation).toHaveBeenCalledOnce());

    const nextUser = { ...cloudUser, uid: "uid-2" };
    let nextLocalId: string | undefined;
    await act(async () => {
      cloudAuthStore.setState({ user: nextUser });
      cloudProfileStore.setState({ profile: { ...cloudProfile, uid: nextUser.uid }, status: "ready" });
      localProfileStore.getState().resetLocalProfile();
      nextLocalId = localProfileStore.getState().ensureLocalProfile().id;
      localProfileStore.getState().renameDisplayName("Next player");
      finish();
      await pending;
    });
    await waitFor(() => expect(screen.getByRole("button", { name: "Reset Profile" })).toBeEnabled());
    expect(screen.getByRole("alert")).toHaveTextContent(/account changed before cleanup finished/i);
    expect(signOut).not.toHaveBeenCalled();
    expect(localProfileStore.getState().profile?.id).toBe(nextLocalId);
    expect(localProfileStore.getState().profile?.displayName).toBe("Next player");
    expect(cloudAuthStore.getState().user).toBe(nextUser);
    expect(cloudPromotionStore.getState().reset).not.toHaveBeenCalled();
  });

  afterEach(() => {
    cleanup();
    cloudAuthStore.setState(initialCloudAuthState, true);
    cloudHistoryStore.setState(initialCloudHistoryState, true);
    cloudProfileStore.setState(initialCloudProfileState, true);
    cloudPromotionStore.setState(initialCloudPromotionState, true);
    localProfileStore.setState(initialLocalProfileState, true);
    replayLibraryStore.setState({ owners: {}, storageError: null });
    replayLibrarySync.store.setState({ users: {} }, true);
  });

  beforeEach(() => {
    localStorage.clear();
    replayLibraryStore.setState({ owners: {}, storageError: null });
    replayLibrarySync.store.setState({ users: {} }, true);
    sharingMocks.clearSharingData.mockReset();
    sharingMocks.clearSharingData.mockImplementation(async (uid: string) => ({
      publicProfile: {
        uid,
        displayName: "",
        enabled: false,
        generation: 2,
        revision: 2,
        updatedAt: "2026-04-28T01:00:00.000Z",
      },
      pages: [],
      privateLibrary: { uid, revision: 3, entries: [], updatedAt: "2026-04-28T01:00:00.000Z" },
    }));
    sharingMocks.loadPrivateLibrary.mockReset();
    sharingMocks.loadPrivateLibrary.mockResolvedValue(null);
    sharingMocks.savePrivateLibrary.mockReset();
    sharingMocks.savePrivateLibrary.mockImplementation(async (
      uid: string,
      entries: unknown,
      expectedRevision: number | null,
    ) => ({
      uid,
      revision: (expectedRevision ?? 0) + 1,
      entries: Array.isArray(entries) ? entries : [],
      updatedAt: "2026-04-28T01:00:00.000Z",
    }));
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: null,
      settings: defaultSettings,
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
      deleteForUser: vi.fn(),
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

  it("keeps game setup controls out of the profile page", () => {
    renderProfileRoute();

    expect(screen.getByRole("heading", { name: /^profile$/i })).toBeInTheDocument();
    expect(screen.queryByText(/default rule/i)).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /freestyle/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /renju/i })).not.toBeInTheDocument();
  });

  it("keeps local profile usable when cloud auth is not configured", () => {
    renderProfileRoute();

    expect(screen.getByRole("heading", { name: "Profile" })).toBeInTheDocument();
    expect(screen.getByText("Local profile")).toBeInTheDocument();
    expect(screen.getByText("Cloud sync unavailable.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sign in" })).toBeDisabled();
    expect(screen.getByRole("link", { name: "Settings" })).toHaveAttribute("href", "/settings");
    expect(screen.getByText("Matches")).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toHaveValue("Guest");
    expect(screen.getByText("No matches yet")).toBeInTheDocument();
    expect(screen.getByText("Finish a game to inspect it here.")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Play a game" })).toHaveAttribute(
      "href",
      "/match/local",
    );
  });

  it("renders canonical local saved-match history", () => {
    localProfileStore.getState().ensureLocalProfile();
    localProfileStore.getState().recordFinishedMatch({
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
    expect(screen.getByText("vs Normal Bot")).toBeInTheDocument();
    expect(screen.getByText("Black")).toBeInTheDocument();
    expect(screen.getByText("Moves 9")).toBeInTheDocument();
  });

  it("confirms before clearing local profile data", async () => {
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    localProfileStore.getState().renameDisplayName("ByeByeBryan");
    const match = localSavedMatch("match-1", localProfile.id);
    localProfileStore.setState({ matchHistory: localMatchHistoryWith([match]) });
    cacheReplayAnalysis(match);

    renderProfileRoute();

    expect(screen.getByLabelText("Name")).toHaveValue("ByeByeBryan");
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    expect(
      screen.getByText("Reset this profile? This clears settings, games, saved replays, practice and replay analyses on this device."),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByLabelText("Name")).toHaveValue("ByeByeBryan");
    expect(readReplayAnalysisCache(match, replayAnalysisOptions)).not.toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    await waitFor(() => expect(screen.getByLabelText("Name")).toHaveValue("Guest"));
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([]);
    expect(readReplayAnalysisCache(match, replayAnalysisOptions)).toBeNull();
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

    expect(screen.getByText("Cloud profile")).toBeInTheDocument();
    expect(screen.queryByText("uid-1")).not.toBeInTheDocument();
    expect(screen.getByText("Cloud sync is on.")).toBeInTheDocument();
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
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: localProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      ruleset: "freestyle",
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

  it("shows queued when cloud history has pending matches", () => {
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = localSavedMatch("match-1", localProfile.id);

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
          sync: {},
        },
      },
    });

    renderProfileRoute();

    expect(screen.getByText("Queued")).toBeInTheDocument();
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
      expect(loadForUser).toHaveBeenCalled();
    });
    const initialLoadCount = loadForUser.mock.calls.length;

    window.dispatchEvent(new Event("online"));

    await waitFor(() => {
      expect(loadForUser.mock.calls.length).toBeGreaterThan(initialLoadCount);
    });
    expect(syncPendingForUser).toHaveBeenCalled();
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

    expect(screen.getByText("Cloud profile")).toBeInTheDocument();
    expect(screen.getByText("Loading cloud profile...")).toBeInTheDocument();
    expect(screen.queryByText("Sign in to sync this profile and its history."))
      .not
      .toBeInTheDocument();
  });

  it("shows merged cloud and local history without duplicate direct cloud saves", () => {
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: localProfile.id,
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
      ruleset: "freestyle",
    });
    const cloudMatch = createCloudSavedMatch(cloudUser, localMatch);

    localProfileStore.setState({ matchHistory: localMatchHistoryWith([localMatch]) });
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
    expect(screen.getAllByRole("button", { name: "Inspect" })).toHaveLength(1);
  });

  it("reveals replay history in batches", () => {
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const matches = Array.from({ length: 20 }, (_, index) =>
      localSavedMatch(`match-${index}`, localProfile.id, index)
    );

    localProfileStore.setState({ matchHistory: localMatchHistoryWith(matches) });

    renderProfileRoute();

    expect(screen.getAllByRole("button", { name: "Inspect" })).toHaveLength(16);
    fireEvent.click(screen.getByRole("button", { name: "Show more" }));
    expect(screen.getAllByRole("button", { name: "Inspect" })).toHaveLength(20);
    expect(screen.queryByRole("button", { name: "Show more" })).not.toBeInTheDocument();
  });

  it("resets cloud profile scope and local cache when signed in", async () => {
    const resetForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockResolvedValue(undefined);
    const resetUserCache = vi.fn();
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: localProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      ruleset: "freestyle",
    });

    localProfileStore.setState({ matchHistory: localMatchHistoryWith([localMatch]) });
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
      screen.getByText(
        "Reset private settings, games, saved replays and practice here and in the cloud? Local replay analyses are cleared. Your public profile and publications stay unchanged; empty cloud revision markers remain.",
      ),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    await waitFor(() => {
      expect(resetForUser).toHaveBeenCalledWith(cloudUser, defaultSettings);
    });
    expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    expect(resetUserCache).toHaveBeenCalledWith("uid-1");
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([]);
  });

  it("clears the account replay library on reset without changing public sharing", async () => {
    localProfileStore.getState().ensureLocalProfile();
    const localCopy = {
      fingerprint: "A".repeat(43),
      payload: "AQ",
      kind: "saved" as const,
      saved_at: "2026-04-28T01:00:00.000Z",
    };
    replayLibraryStore.setState({
      owners: { "user:uid-1": { dirty: true, entries: [localCopy], error: null } },
    });
    sharingMocks.loadPrivateLibrary.mockResolvedValue({
      uid: "uid-1",
      revision: 6,
      entries: [localCopy],
      updatedAt: "2026-04-28T01:00:00.000Z",
    });
    sharingMocks.savePrivateLibrary.mockResolvedValue({
      uid: "uid-1",
      revision: 7,
      entries: [],
      updatedAt: "2026-04-28T01:01:00.000Z",
    });
    const resetForUser = vi.fn().mockResolvedValue(undefined);
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

    renderProfileRoute();
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    await waitFor(() => expect(resetForUser).toHaveBeenCalledOnce());
    expect(sharingMocks.loadPrivateLibrary).toHaveBeenCalledWith("uid-1");
    expect(sharingMocks.savePrivateLibrary).toHaveBeenCalledWith("uid-1", [], 6);
    expect(sharingMocks.clearSharingData).not.toHaveBeenCalled();
    expect(replayLibraryStore.getState().owners["user:uid-1"]?.entries).toEqual([]);
    expect(replayLibrarySync.store.getState().users["uid-1"]).toMatchObject({
      loaded: true,
      revision: 7,
      status: "synced",
      hasIncoming: false,
    });
  });

  it("deletes the online profile and signs out without clearing local history", async () => {
    const deleteForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockResolvedValue(undefined);
    const resetUserCache = vi.fn();
    const signOut = vi.fn().mockResolvedValue(undefined);
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = localSavedMatch("match-1", localProfile.id);
    const accountCopy = {
      fingerprint: "B".repeat(43),
      payload: "AQ",
      kind: "saved" as const,
      saved_at: "2026-04-28T01:00:00.000Z",
    };

    localProfileStore.setState({ matchHistory: localMatchHistoryWith([localMatch]) });
    replayLibraryStore.setState({
      owners: { "user:uid-1": { dirty: false, entries: [accountCopy], error: null } },
    });
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
      deleteForUser,
      errorMessage: null,
      loadForUser: vi.fn(),
      profile: cloudProfile,
      reset: vi.fn(),
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({
      clearForUser,
      resetUserCache,
    });

    renderProfileRoute();

    expect(screen.queryByRole("button", { name: "Delete Cloud" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Cloud" }));
    expect(
      screen.getByText(
        "Delete your cloud profile, history, saved replays, public name and publications, then sign out? Local copies stay on this device. Empty revision markers remain to block stale writes. Copied replay links still work.",
      ),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(deleteForUser).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Cloud" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(deleteForUser).toHaveBeenCalledWith(cloudUser);
    });
    expect(sharingMocks.clearSharingData).toHaveBeenCalledWith("uid-1");
    expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    expect(resetUserCache).toHaveBeenCalledWith("uid-1");
    expect(signOut).toHaveBeenCalledTimes(1);
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([localMatch]);
    expect(replayLibraryStore.getState().owners["user:uid-1"]).toMatchObject({
      dirty: true,
      entries: [accountCopy],
    });
    expect(replayLibrarySync.store.getState().users["uid-1"]).toMatchObject({
      status: "conflict",
      hasIncoming: true,
      incoming: { entries: [] },
    });
    expect(sharingMocks.clearSharingData.mock.invocationCallOrder[0]).toBeLessThan(
      clearForUser.mock.invocationCallOrder[0],
    );
    expect(clearForUser.mock.invocationCallOrder[0]).toBeLessThan(deleteForUser.mock.invocationCallOrder[0]);
    expect(deleteForUser.mock.invocationCallOrder[0]).toBeLessThan(signOut.mock.invocationCallOrder[0]);
  });

  it("keeps Delete Cloud retryable when sharing clear fails and skips profile deletion/sign-out", async () => {
    const deleteForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockResolvedValue(undefined);
    const signOut = vi.fn().mockResolvedValue(undefined);
    sharingMocks.clearSharingData.mockRejectedValueOnce(new Error("Sharing cleanup failed."));
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
      deleteForUser,
      errorMessage: null,
      loadForUser: vi.fn(),
      profile: cloudProfile,
      reset: vi.fn(),
      resetForUser: vi.fn(),
      status: "ready",
    });
    cloudHistoryStore.setState({ clearForUser });

    renderProfileRoute();
    fireEvent.click(screen.getByRole("button", { name: "Reset Profile" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete Cloud" }));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Sharing cleanup failed.");
    expect(screen.getByRole("button", { name: "Delete" })).toBeInTheDocument();
    expect(clearForUser).not.toHaveBeenCalled();
    expect(deleteForUser).not.toHaveBeenCalled();
    expect(signOut).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(deleteForUser).toHaveBeenCalledWith(cloudUser));
    expect(sharingMocks.clearSharingData).toHaveBeenCalledTimes(2);
    expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    expect(signOut).toHaveBeenCalledOnce();
  });

  it("keeps signed-in reset confirmation open and local history intact when cloud clear fails", async () => {
    const resetForUser = vi.fn().mockResolvedValue(undefined);
    const clearForUser = vi.fn().mockRejectedValue(new Error("permission denied"));
    const resetUserCache = vi.fn();
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    const localMatch = createLocalSavedMatch({
      id: "match-1",
      localProfileId: localProfile.id,
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-28T01:00:00.000Z",
      status: "draw",
      ruleset: "freestyle",
    });

    localProfileStore.setState({ matchHistory: localMatchHistoryWith([localMatch]) });
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
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));

    await waitFor(() => {
      expect(clearForUser).toHaveBeenCalledWith(cloudUser);
    });
    expect(screen.getByRole("button", { name: "Reset" })).toBeInTheDocument();
    expect(resetUserCache).not.toHaveBeenCalled();
    expect(localProfileStore.getState().matchHistory.replayMatches).toEqual([localMatch]);
  });

  it("starts background local profile promotion after cloud profile loads", async () => {
    const promote = vi.fn().mockResolvedValue(undefined);
    const localProfile = localProfileStore.getState().ensureLocalProfile();
    localProfileStore.getState().renameDisplayName("ByeByeBryan");
    const matchHistory = localProfileStore.getState().matchHistory;
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
        sourceProfile: cloudProfile,
        cloudDisplayName: cloudProfile.displayName,
        cloudMatchHistory: cloudProfile.matchHistory,
        cloudSettings: cloudProfile.settings,
        localMatchHistory: matchHistory,
        localProfile: expect.objectContaining({
          displayName: "ByeByeBryan",
          id: localProfile.id,
        }),
        resetAt: null,
        settings: defaultSettings,
        user: cloudUser,
      });
    }, { timeout: 3_000 });
  });

  it("syncs when local profile fields change after cloud profile loads", async () => {
    const promote = vi.fn().mockResolvedValue(undefined);
    const localProfile = localProfileStore.getState().ensureLocalProfile();
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
      expect(screen.getByLabelText("Name")).toHaveValue("Bryan");
    });

    fireEvent.change(screen.getByLabelText("Name"), {
      target: { value: "Later Name" },
    });
    await waitFor(() => {
      expect(screen.getByLabelText("Name")).toHaveValue("Later Name");
    });
    localProfileStore.getState().updateSettings({
      gameConfig: {
        opening: "standard",
        ruleset: "renju",
      },
    });
    await Promise.resolve();

    await waitFor(() => {
      expect(promote).toHaveBeenCalledTimes(2);
    });
    expect(promote).toHaveBeenLastCalledWith(expect.objectContaining({
      localProfile: expect.objectContaining({
        displayName: "Later Name",
        id: localProfile.id,
      }),
      settings: expect.objectContaining({
        gameConfig: expect.objectContaining({
          ruleset: "renju",
        }),
      }),
    }));
  });

  it("restores the account setup before history promotion and retains later local edits", async () => {
    const local = localProfileStore.getState().ensureLocalProfile();
    localProfileStore.setState({ matchHistory: localMatchHistoryWith([localSavedMatch("pending-local", local.id)]) });
    const savedSettings = {
      ...defaultSettings,
      botConfig: { mode: "preset", preset: "hard", version: 1 } as const,
      gameConfig: { opening: "standard", ruleset: "renju" } as const,
    };
    const promote = vi.fn().mockResolvedValue(undefined);
    cloudAuthStore.setState({ status: "signed_in", isConfigured: true, user: cloudUser });
    cloudProfileStore.setState({ profile: { ...cloudProfile, settings: savedSettings }, status: "ready" });
    cloudPromotionStore.setState({ promote });
    renderProfileRoute();
    await waitFor(() => expect(promote).toHaveBeenCalled());
    expect(localProfileStore.getState().settings).toEqual(savedSettings);
    expect(promote).toHaveBeenLastCalledWith(expect.objectContaining({ settings: savedSettings }));
    await act(async () => {
      localProfileStore.getState().updateSettings({ gameConfig: { opening: "standard", ruleset: "freestyle" } });
      cloudProfileStore.setState({ profile: { ...cloudProfile, settings: savedSettings } });
    });
    await waitFor(() => expect(promote).toHaveBeenLastCalledWith(expect.objectContaining({
      settings: expect.objectContaining({ gameConfig: { opening: "standard", ruleset: "freestyle" } }),
    })));
    expect(localProfileStore.getState().settings.gameConfig.ruleset).toBe("freestyle");
  });

  it("adopts the cloud display name without promoting a default local name", async () => {
    const promote = vi.fn().mockResolvedValue(undefined);
    localProfileStore.getState().ensureLocalProfile();
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
      expect(screen.getByLabelText("Name")).toHaveValue("Bryan");
    });
    expect(promote).not.toHaveBeenCalled();
  });
});
