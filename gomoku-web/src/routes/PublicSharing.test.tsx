import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes, useNavigate } from "react-router-dom";

import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { localProfileStore } from "../profile/local_profile_store";
import type { PublicCatalogEntry, PublicCatalogPageSnapshot, PublicProfileSnapshot } from "../cloud/sharing_public";

import { PublicPlayerRoute } from "./PublicPlayerRoute";
import { PublicProfileManager } from "./PublicProfileManager";
import { PublishReplayPanel } from "./PublishReplayPanel";

const sharing = vi.hoisted(() => ({
  loadCatalogPage: vi.fn(),
  loadPublicProfile: vi.fn(),
  saveCatalogPage: vi.fn(),
  setPublicProfile: vi.fn(),
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
    ...overrides,
  };
}

function entry(index: number): PublicCatalogEntry {
  return {
    fingerprint: String(index).padStart(43, "x"),
    payload: "YWJj",
    title: `Public game ${index}`,
    view_ply: 9,
  };
}

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
  sharing.loadCatalogPage.mockResolvedValue(null);
  sharing.loadPublicProfile.mockResolvedValue(null);
  sharing.saveCatalogPage.mockImplementation(async (uid: string, pageId: number, entries: PublicCatalogEntry[], generation: number, revision: number) => page(
    pageId as 0 | 1 | 2 | 3,
    entries,
    { uid, generation, revision: revision + 1 },
  ));
  sharing.setPublicProfile.mockImplementation(async (uid: string, displayName: string, enabled: boolean, revision: number | null) => publicProfile({
    uid,
    displayName: enabled ? displayName : "",
    enabled,
    generation: enabled ? 2 : 3,
    revision: (revision ?? 0) + 1,
  }));
});

describe("public profile routes and controls", () => {
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
    sharing.loadPublicProfile
      .mockResolvedValueOnce(null)
      .mockResolvedValue(publicProfile({ uid: "uid-1", displayName: "Available player" }));
    let nextPageAttempts = 0;
    sharing.loadCatalogPage.mockImplementation(async (uid: string, pageId: number) => {
      if (pageId === 1 && nextPageAttempts++ === 0) {
        throw Object.assign(new Error("The network request timed out."), { code: "unavailable" });
      }
      return page(
        pageId as 0 | 1 | 2 | 3,
        pageId === 0 ? [entry(1)] : [],
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
    expect(screen.getByRole("link", { name: "View replay" })).toHaveAttribute("href", expect.stringContaining("#g2r1=YWJj"));

    fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    await waitFor(() => expect(sharing.loadCatalogPage).toHaveBeenLastCalledWith("uid-1", 1));
    expect(await screen.findByRole("button", { name: "Retry page" })).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("network request timed out");
    fireEvent.click(screen.getByRole("button", { name: "Retry page" }));
    expect(await screen.findByText("Page 2 of 4")).toBeInTheDocument();
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(3);

    fireEvent.click(screen.getByRole("button", { name: "Previous page" }));
    expect(screen.getByText("Page 1 of 4")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Next page" }));
    expect(screen.getByText("Page 2 of 4")).toBeInTheDocument();
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

    render(<MemoryRouter><PublicProfileManager /></MemoryRouter>);
    fireEvent.click(screen.getByRole("button", { name: "Manage public profile" }));
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

  it("removes a publication only after the guarded save succeeds and explains disable behavior", async () => {
    sharing.loadPublicProfile.mockResolvedValue(publicProfile());
    sharing.loadCatalogPage.mockImplementation(async (_uid: string, pageId: number) => page(
      pageId as 0 | 1 | 2 | 3,
      pageId === 0 ? [entry(7)] : [],
    ));
    sharing.saveCatalogPage.mockImplementation(async (uid: string, pageId: number, entries: PublicCatalogEntry[], generation: number, revision: number) => page(
      pageId as 0 | 1 | 2 | 3,
      entries,
      { uid, generation, revision: revision + 1 },
    ));
    sharing.setPublicProfile
      .mockResolvedValueOnce(publicProfile({ displayName: "", enabled: false, generation: 3, revision: 5 }))
      .mockResolvedValueOnce(publicProfile({ displayName: "Public name", enabled: true, generation: 4, revision: 6 }));

    render(<MemoryRouter><PublicProfileManager /></MemoryRouter>);
    fireEvent.click(screen.getByRole("button", { name: "Manage public profile" }));
    expect(await screen.findByText("Public game 7")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Unpublish" }));
    await waitFor(() => expect(sharing.saveCatalogPage).toHaveBeenCalledWith(
      signedInUser.uid,
      0,
      [],
      2,
      1,
    ));
    expect(await screen.findByRole("status")).toHaveTextContent("Publication removed");

    fireEvent.click(screen.getByRole("button", { name: "Disable public profile" }));
    expect(screen.getByText(/Re-enabling starts with an empty catalog/)).toHaveTextContent("Portable replay copies already made by others cannot be recalled");
    fireEvent.click(screen.getByRole("button", { name: "Confirm disable" }));
    await waitFor(() => expect(sharing.setPublicProfile).toHaveBeenCalledWith(
      signedInUser.uid,
      "",
      false,
      4,
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
      5,
    ));
    expect(sharing.loadPublicProfile).toHaveBeenCalledTimes(1);
    expect(await screen.findByRole("status")).toHaveTextContent("Public profile enabled with an empty catalog");
  });

  it("reports a full public catalog and leaves the catalog untouched", async () => {
    const fullPages = [0, 1, 2, 3].map((pageId) => page(
      pageId as 0 | 1 | 2 | 3,
      Array.from({ length: 8 }, (_, index) => entry(pageId * 8 + index + 1)),
    ));
    sharing.loadPublicProfile.mockResolvedValue(publicProfile());
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
      3,
      1,
    ));
    expect(sharing.loadCatalogPage).toHaveBeenCalledTimes(4);
  });
});
