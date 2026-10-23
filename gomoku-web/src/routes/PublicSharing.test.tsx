import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes, useNavigate } from "react-router-dom";

import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import type { PublicCatalogEntry, PublicCatalogPageSnapshot, PublicProfileSnapshot } from "../cloud/sharing_public";
import { portableReplayFromSavedMatch } from "../replay/portable_replay";
import { encodePortableReplay } from "../replay/portable_replay_codec";

import { PublicPlayerRoute } from "./PublicPlayerRoute";
import { PublicProfileManager } from "./PublicProfileManager";
import { PublishReplayPanel } from "./PublishReplayPanel";

const sharing = vi.hoisted(() => ({
  loadCatalogPage: vi.fn(),
  loadPublicProfile: vi.fn(),
  saveCatalogPage: vi.fn(),
  setPublicProfile: vi.fn(),
  setPublicProfileStats: vi.fn(),
}));

vi.mock("../cloud/sharing_public", () => sharing);

const signedInUser: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Provider name must not be published",
  email: "owner@example.com",
  providerIds: ["google.com"],
  uid: "owner_uid-1",
};

const initialAuthState = cloudAuthStore.getState();
const initialLocalState = localProfileStore.getState();

function publicProfile(overrides: Partial<PublicProfileSnapshot> = {}): PublicProfileSnapshot {
  return {
    uid: signedInUser.uid,
    displayName: "Public name",
    enabled: true,
    generation: 2,
    revision: 4,
    updatedAt: "2026-10-21T00:00:00.000Z",
    sharedStats: null,
    catalogPageCounts: [0, 0, 0, 0],
    ...overrides,
  };
}

function entry(index: number, payload = "YWJj"): PublicCatalogEntry {
  return {
    fingerprint: String(index).padStart(43, "x"),
    payload,
    title: `Public game ${index}`,
    view_ply: 9,
  };
}

function replayPayload(uid: string): string {
  const replay = portableReplayFromSavedMatch(winningMatch()).replay!;
  if (replay.black.kind !== "human") throw new Error("Fixture publisher must be human.");
  return encodePortableReplay({ ...replay, black: { ...replay.black, profileUid: uid } });
}

const ownerStats = { matches: 1, wins: 1, losses: 0, draws: 0 };

function page(pageId: 0 | 1 | 2 | 3, entries: PublicCatalogEntry[] = [], overrides: Partial<PublicCatalogPageSnapshot> = {}): PublicCatalogPageSnapshot {
  return {
    uid: signedInUser.uid,
    pageId,
    generation: 2,
    revision: 1,
    entries,
    updatedAt: "2026-10-21T00:00:00.000Z",
    ...overrides,
  };
}

function setSignedIn(uid = signedInUser.uid): void {
  cloudAuthStore.setState({
    ...initialAuthState,
    isConfigured: true,
    status: "signed_in",
    user: { ...signedInUser, uid },
  });
}

function renderManager() {
  render(
    <MemoryRouter>
      <PublicProfileManager stats={ownerStats} statsReady statsBlocker="History is still loading." />
    </MemoryRouter>,
  );
}

function SwitchProfileButton() {
  const navigate = useNavigate();
  return <button type="button" onClick={() => navigate("/player/?id=uid-2")}>Switch public player</button>;
}

function winningMatch() {
  const moves = movesFromMoveCells([110, 0, 111, 1, 112, 2, 113, 3, 114]);
  return createLocalSavedMatch({
    id: "private-match-id",
    localProfileId: "local-owner",
    moves,
    players: [
      { kind: "human", name: "Local label", stone: "black" },
      { kind: "bot", name: "Hard Bot", stone: "white" },
    ],
    ruleset: "freestyle",
    savedAt: "2026-10-21T00:00:00.000Z",
    status: "black_won",
  });
}

afterEach(() => {
  cleanup();
  cloudAuthStore.setState(initialAuthState);
  localProfileStore.setState(initialLocalState);
  vi.clearAllMocks();
});

beforeEach(() => {
  setSignedIn();
  localProfileStore.setState({
    ...initialLocalState,
    profile: {
      avatarUrl: null,
      createdAt: "2026-10-20T00:00:00.000Z",
      displayName: "Local suggested name",
      id: "local-owner",
      kind: "local",
      updatedAt: "2026-10-20T00:00:00.000Z",
      username: null,
    },
  });
  sharing.loadPublicProfile.mockReset();
  sharing.loadCatalogPage.mockReset();
  sharing.saveCatalogPage.mockReset();
  sharing.setPublicProfile.mockReset();
  sharing.setPublicProfileStats.mockReset();
  sharing.loadCatalogPage.mockResolvedValue(null);
  sharing.loadPublicProfile.mockResolvedValue(null);
  sharing.saveCatalogPage.mockImplementation(async (
    uid: string,
    pageId: number,
    entries: PublicCatalogEntry[],
    expectedRootRevision: number,
    generation: number,
    pageRevision: number,
  ) => ({
    profile: publicProfile({ uid, revision: expectedRootRevision + 1, catalogPageCounts: [entries.length, 0, 0, 0] }),
    page: page(pageId as 0 | 1 | 2 | 3, entries, { uid, generation, revision: pageRevision + 1 }),
  }));
  sharing.setPublicProfile.mockImplementation(async (uid: string, displayName: string, enabled: boolean, revision: number | null) => publicProfile({
    uid,
    displayName: enabled ? displayName : "",
    enabled,
    generation: enabled ? 2 : 3,
    revision: (revision ?? 0) + 1,
  }));
  sharing.setPublicProfileStats.mockImplementation(async (uid: string, stats: typeof ownerStats | null, revision: number) => publicProfile({
    uid,
    revision: revision + 1,
    sharedStats: stats ? { ...stats, updatedAt: "2026-10-22T00:00:00.000Z" } : null,
  }));
});

describe("public profile routes and controls", () => {
  it("renders an opted-in snapshot and an explicit zero-rate state without empty page controls", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({
      uid: "uid-1",
      catalogPageCounts: [0, 0, 0, 0],
      sharedStats: { matches: 0, wins: 0, losses: 0, draws: 0, updatedAt: "2026-10-22T00:00:00.000Z" },
    }));
    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByRole("heading", { name: "Public name" })).toBeInTheDocument();
    expect(screen.getByText("Win rate").parentElement).toHaveTextContent("—");
    expect(screen.getByText(/Updated /)).toBeInTheDocument();
    expect(screen.getByText("No games have been published.")).toBeInTheDocument();
    expect(screen.queryByText(/Page 1 of/)).not.toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();
  });

  it("fills sparse physical pages into eight-row public pages and reuses only the needed page", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ uid: "uid-1", catalogPageCounts: [0, 3, 0, 8] }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      pageId === 1
        ? [entry(11), entry(12), entry(13)]
        : pageId === 3
          ? Array.from({ length: 8 }, (_, index) => entry(31 + index))
          : [],
      { uid },
    ));
    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );

    expect(await screen.findByText("Page 1 of 2")).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(8);
    expect(sharing.loadCatalogPage.mock.calls.map((call) => call[1])).toEqual([1, 3]);
    fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    expect(await screen.findByText("Page 2 of 2")).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(3);
    expect(screen.getByText("Public game 36")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(2);
  });

  it("loads every physical page once for a legacy root and derives its logical count without a write", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ uid: "uid-1", catalogPageCounts: null }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      pageId === 2 ? [entry(21)] : [],
      { uid },
    ));
    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByText("Public game 21")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(4);
    expect(screen.queryByText(/Page 1 of/)).not.toBeInTheDocument();
    expect(sharing.saveCatalogPage).not.toHaveBeenCalled();
  });

  it("shows a reloadable error when a modern manifest disagrees with a page", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ uid: "uid-1", catalogPageCounts: [1, 0, 0, 0] }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      [],
      { uid },
    ));
    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByRole("heading", { name: "Public profile unavailable" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("public catalog changed");
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(sharing.loadPublicProfile).toHaveBeenCalledTimes(2);
  });

  it("ignores a late public profile response after the visitor changes UID", async () => {
    let resolveFirst!: (value: PublicProfileSnapshot) => void;
    sharing.loadPublicProfile.mockImplementation((uid: string) => {
      if (uid === "uid-1") return new Promise<PublicProfileSnapshot>((resolve) => { resolveFirst = resolve; });
      return Promise.resolve(publicProfile({ uid: "uid-2", displayName: "Second public player" }));
    });
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      [],
      { uid },
    ));

    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <SwitchProfileButton />
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    // Render the navigator control inside the router with normal JSX casing.
    fireEvent.click(screen.getByRole("button", { name: "Switch public player" }));
    expect(await screen.findByRole("heading", { name: "Second public player" })).toBeInTheDocument();

    await act(async () => {
      resolveFirst(publicProfile({ uid: "uid-1", displayName: "Old public player" }));
    });
    expect(screen.queryByRole("heading", { name: "Old public player" })).not.toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalledWith("uid-1", 0);
  });

  it("shows unavailable profiles, retries, and fetches each catalog page only after a request", async () => {
    const payload = replayPayload("uid-1");
    const firstEntries = Array.from({ length: 8 }, (_, index) => entry(index + 1, payload));
    sharing.loadPublicProfile
      .mockResolvedValueOnce(null)
      .mockResolvedValue(publicProfile({ uid: "uid-1", displayName: "Available player", catalogPageCounts: [8, 1, 0, 0] }));
    let nextPageAttempts = 0;
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => {
      if (pageId === 1 && nextPageAttempts++ === 0) {
        throw Object.assign(new Error("The network request timed out."), { code: "unavailable" });
      }
      return page(
        pageId as 0 | 1 | 2 | 3,
        pageId === 0 ? firstEntries : pageId === 1 ? [entry(9, payload)] : [],
        { uid },
      );
    });

    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByRole("heading", { name: "Public profile unavailable" })).toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("heading", { name: "Available player" })).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(1);
    expect(sharing.loadCatalogPage).toHaveBeenLastCalledWith("uid-1", 0);
    expect(screen.getAllByRole("link", { name: "Replay" })).toHaveLength(8);
    expect(screen.getAllByText("Win", { exact: true })).toHaveLength(8);
    expect(screen.getAllByText("vs Normal Bot", { exact: true })).toHaveLength(8);
    expect(screen.getAllByText("Freestyle", { exact: true })).toHaveLength(8);
    expect(screen.getAllByText("9 moves", { exact: true })).toHaveLength(8);

    fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await waitFor(() => expect(sharing.loadCatalogPage).toHaveBeenLastCalledWith("uid-1", 1));
    expect(await screen.findByRole("button", { name: "Retry page" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("network request timed out");
    fireEvent.click(screen.getByRole("button", { name: "Retry page" }));
    expect(await screen.findByText("Page 2 of 2")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(3);

    fireEvent.click(screen.getByRole("button", { name: "Previous page" }));
    expect(await screen.findByText("Page 1 of 2")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    expect(await screen.findByText("Page 2 of 2")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(3);
  });

  it("hides Firebase authorization details for unavailable public profiles", async () => {
    sharing.loadPublicProfile.mockRejectedValueOnce(Object.assign(
      new Error("private Firestore details"),
      { code: "firestore/permission-denied" },
    ));
    render(
      <MemoryRouter initialEntries={["/player/?id=uid-1"]}>
        <Routes><Route path="/player/" element={<PublicPlayerRoute />} /></Routes>
      </MemoryRouter>,
    );
    expect(await screen.findByText("This public player is unavailable.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Try again" })).toBeInTheDocument();
    expect(screen.queryByText("private Firestore details")).not.toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();
  });

  it("keeps public profile opt-in behind the explicit confirmation", async () => {
    sharing.loadCatalogPage.mockResolvedValue(null);
    sharing.setPublicProfile.mockResolvedValue(publicProfile({ uid: signedInUser.uid, displayName: "Local suggested name", revision: 1 }));

    renderManager();
    await waitFor(() => expect(sharing.loadPublicProfile).toHaveBeenCalledWith(signedInUser.uid));
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(4);
    expect(screen.getByDisplayValue("Local suggested name")).toBeInTheDocument();
    expect(screen.getByText(/Anyone with your profile link will see/)).toHaveTextContent("Local suggested name");
    expect(screen.queryByText(signedInUser.displayName)).not.toBeInTheDocument();
    expect(sharing.setPublicProfile).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("checkbox", { name: /I want to enable this public profile/ }));
    fireEvent.click(screen.getByRole("button", { name: "Enable public profile" }));
    await waitFor(() => expect(sharing.setPublicProfile).toHaveBeenCalledWith(
      signedInUser.uid,
      "Local suggested name",
      true,
      null,
    ));
    expect(screen.getByRole("status")).toHaveTextContent("Public profile enabled");
  });

  it("keeps statistics off by default, requires ready history to opt in or refresh, and allows removal without it", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ catalogPageCounts: [0, 0, 0, 0] }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      [],
      { uid },
    ));
    const rendered = render(
      <MemoryRouter>
        <PublicProfileManager stats={ownerStats} statsReady={false} statsBlocker="Wait for private history." />
      </MemoryRouter>,
    );
    const toggle = await screen.findByRole("checkbox", { name: "Share My games statistics publicly" });
    expect(toggle).not.toBeChecked();
    expect(toggle).toBeDisabled();
    expect(sharing.setPublicProfileStats).not.toHaveBeenCalled();

    rendered.rerender(
      <MemoryRouter>
        <PublicProfileManager stats={ownerStats} statsReady statsBlocker="Wait for private history." />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Share My games statistics publicly" }));
    await waitFor(() => expect(sharing.setPublicProfileStats).toHaveBeenCalledWith(signedInUser.uid, ownerStats, 4));
    expect(await screen.findByText("Published October 21, 2026")).toBeInTheDocument();
    expect(screen.getByText("100.0%")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Update public stats" }));
    await waitFor(() => expect(sharing.setPublicProfileStats).toHaveBeenNthCalledWith(2, signedInUser.uid, ownerStats, 5));
    rendered.rerender(
      <MemoryRouter>
        <PublicProfileManager stats={ownerStats} statsReady={false} statsBlocker="Wait for private history." />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByRole("checkbox", { name: "Share My games statistics publicly" }));
    await waitFor(() => expect(sharing.setPublicProfileStats).toHaveBeenNthCalledWith(3, signedInUser.uid, null, 6));
    expect(await screen.findByRole("status")).toHaveTextContent("Public statistics removed");
  });

  it("ignores a delayed statistics save after the owner switches account", async () => {
    sharing.loadPublicProfile.mockImplementation(async (uid: string) => publicProfile({
      uid, displayName: uid, catalogPageCounts: [0, 0, 0, 0],
    }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3, [], { uid },
    ));
    let resolveSave!: (value: PublicProfileSnapshot) => void;
    sharing.setPublicProfileStats.mockImplementation(() => new Promise<PublicProfileSnapshot>((resolve) => { resolveSave = resolve; }));
    renderManager();
    fireEvent.click(await screen.findByRole("checkbox", { name: "Share My games statistics publicly" }));
    await waitFor(() => expect(sharing.setPublicProfileStats).toHaveBeenCalledTimes(1));
    await act(async () => { setSignedIn("second-owner"); });
    await screen.findByDisplayValue("second-owner");
    await act(async () => {
      resolveSave(publicProfile({ sharedStats: { ...ownerStats, updatedAt: "2026-10-22T00:00:00.000Z" } }));
    });
    expect(screen.getByRole("checkbox", { name: "Share My games statistics publicly" })).not.toBeChecked();
    expect(screen.queryByText("Your current My games summary is now public.")).not.toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Public name" })).toHaveValue("second-owner");
  });

  it("allows a fully loaded zero-game record as an explicit snapshot", async () => {
    const zeroStats = { matches: 0, wins: 0, losses: 0, draws: 0 };
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ catalogPageCounts: [0, 0, 0, 0] }));
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      [],
      { uid },
    ));
    render(
      <MemoryRouter>
        <PublicProfileManager stats={zeroStats} statsReady statsBlocker="History is still loading." />
      </MemoryRouter>,
    );
    fireEvent.click(await screen.findByRole("checkbox", { name: "Share My games statistics publicly" }));
    await waitFor(() => expect(sharing.setPublicProfileStats).toHaveBeenCalledWith(signedInUser.uid, zeroStats, 4));
    expect(await screen.findByText("—")).toBeInTheDocument();
  });

  it("removes a publication only after the guarded save succeeds and explains disable behavior", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ catalogPageCounts: [1, 0, 0, 0] }));
    sharing.loadCatalogPage.mockImplementation(async (_uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      pageId === 0 ? [entry(7)] : [],
    ));
    sharing.saveCatalogPage.mockImplementation(async (
      uid: string,
      pageId: number,
      entries: PublicCatalogEntry[],
      expectedRootRevision: number,
      generation: number,
      pageRevision: number,
    ) => ({
      profile: publicProfile({ uid, revision: expectedRootRevision + 1, catalogPageCounts: [0, 0, 0, 0] }),
      page: page(pageId as 0 | 1 | 2 | 3, entries, { uid, generation, revision: pageRevision + 1 }),
    }));
    sharing.setPublicProfile
      .mockResolvedValueOnce(publicProfile({ displayName: "", enabled: false, generation: 3, revision: 6 }))
      .mockResolvedValueOnce(publicProfile({ displayName: "Public name", enabled: true, generation: 4, revision: 6 }));

    renderManager();
    expect(await screen.findByText("Public game 7")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Unpublish" }));
    await waitFor(() => expect(sharing.saveCatalogPage).toHaveBeenCalledWith(
      signedInUser.uid,
      0,
      [],
      4,
      2,
      1,
    ));
    expect(await screen.findByRole("status")).toHaveTextContent("Publication removed");

    fireEvent.click(screen.getByRole("button", { name: "Disable public profile" }));
    expect(screen.getByText(/Re-enabling starts empty with statistics off/)).toHaveTextContent("Portable replay copies already made by others cannot be recalled");
    fireEvent.click(screen.getByRole("button", { name: "Confirm disable" }));
    await waitFor(() => expect(sharing.setPublicProfile).toHaveBeenCalledWith(
      signedInUser.uid,
      "",
      false,
      5,
    ));
    expect(await screen.findByRole("status")).toHaveTextContent("Public profile disabled");
    expect(screen.getByRole("checkbox", { name: /I want to enable this public profile/ })).toBeInTheDocument();
    expect(sharing.loadPublicProfile).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("checkbox", { name: /I want to enable this public profile/ }));
    fireEvent.click(screen.getByRole("button", { name: "Enable public profile" }));
    await waitFor(() => expect(sharing.setPublicProfile).toHaveBeenNthCalledWith(
      2,
      signedInUser.uid,
      "Public name",
      true,
      6,
    ));
    expect(sharing.loadPublicProfile).toHaveBeenCalledTimes(1);
    expect(await screen.findByRole("status")).toHaveTextContent("Public profile enabled with an empty catalog");
  });

  it("reports a full public catalog and leaves the catalog untouched", async () => {
    const fullPages = [0, 1, 2, 3].map((pageId) => page(
      pageId as 0 | 1 | 2 | 3,
      Array.from({ length: 8 }, (_, index) => entry(pageId * 8 + index + 1)),
    ));
    sharing.loadPublicProfile.mockResolvedValue(publicProfile({ catalogPageCounts: [8, 8, 8, 8] }));
    sharing.loadCatalogPage.mockImplementation(async (_uid: string, pageId: number) => fullPages[pageId]!);

    render(
      <MemoryRouter>
        <PublishReplayPanel match={winningMatch()} eligible moveIndex={8} />
      </MemoryRouter>,
    );
    expect(sharing.loadPublicProfile).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Publish replay" }));
    expect(await screen.findByText("Public name")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();

    fireEvent.change(screen.getByRole("textbox", { name: "Public replay title" }), { target: { value: "Winning game" } });
    fireEvent.click(screen.getByRole("button", { name: "Publish this replay" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("catalog is full");
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(4);
    expect(sharing.saveCatalogPage).not.toHaveBeenCalled();
  });

  it("requires an explicit preview reload when the public profile revision changes", async () => {
    const current = publicProfile({ uid: signedInUser.uid, generation: 3, revision: 1, displayName: "Updated public name" });
    sharing.loadPublicProfile
      .mockResolvedValueOnce(publicProfile())
      .mockResolvedValueOnce(publicProfile({ generation: 3, revision: 1, displayName: "Updated public name" }))
      .mockResolvedValueOnce(current)
      .mockResolvedValueOnce(current);
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      [],
      { uid, generation: 3, revision: 1 },
    ));

    render(
      <MemoryRouter>
        <PublishReplayPanel match={winningMatch()} eligible moveIndex={8} />
      </MemoryRouter>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Publish replay" }));
    expect(await screen.findByText("Public name")).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Public replay title" }), { target: { value: "Winning game" } });
    fireEvent.click(screen.getByRole("button", { name: "Publish this replay" }));

    expect(await screen.findByRole("button", { name: "Reload preview" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("changed since this preview loaded");
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();
    expect(sharing.saveCatalogPage).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Reload preview" }));
    expect(await screen.findByText("Updated public name")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Publish this replay" }));
    await waitFor(() => expect(sharing.saveCatalogPage).toHaveBeenCalledWith(
      signedInUser.uid,
      0,
      expect.arrayContaining([expect.objectContaining({ title: "Winning game", view_ply: 8 })]),
      1,
      3,
      1,
    ));
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(4);
  });
});
