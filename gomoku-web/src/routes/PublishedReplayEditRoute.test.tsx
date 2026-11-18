import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes, useLocation } from "react-router-dom";

import { cloudAuthStore, type CloudAuthUser } from "../cloud/auth_store";
import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { portableReplayFromSavedMatch, validatePortableReplay } from "../replay/portable_replay";
import { encodePortableReplay, portableReplayFingerprint } from "../replay/portable_replay_codec";
import type { PublicCatalogEntry, PublicCatalogPageSnapshot, PublicProfileSnapshot } from "../cloud/sharing_public";
import { PublishedReplayEditRoute } from "./PublishedReplayEditRoute";

const sharing = vi.hoisted(() => ({
  editPublishedReplay: vi.fn(),
  loadCatalogPage: vi.fn(),
  loadPublicProfile: vi.fn(),
  SharingConflictError: class SharingConflictError extends Error {},
}));

vi.mock("../cloud/sharing_public", () => sharing);

interface PublicDetails {
  kind: "public" | "private";
  value: { title: string; note: string; opening_ply: number | null };
  onSave: (value: PublicDetails["value"], previous: PublicDetails["value"]) => Promise<string | void>;
  onCancel?: () => void;
}

vi.mock("./ReplayRoute", async () => {
  const React = await import("react");
  return {
    ReplayViewer: ({ details }: { details?: PublicDetails }) => {
      const [draft, setDraft] = React.useState(details?.value ?? { title: "", note: "", opening_ply: null });
      const [failure, setFailure] = React.useState("");
      const save = () => {
        if (!details) return;
        void details.onSave(draft, details.value).catch((error: unknown) => {
          setFailure(error instanceof Error ? error.message : "Save failed.");
        });
      };
      return React.createElement("main", null,
        React.createElement("output", { "aria-label": "Publication draft" }, JSON.stringify(draft)),
        React.createElement("button", { type: "button", onClick: () => setDraft({ ...draft, title: "Edited title" }) }, "Change title"),
        React.createElement("button", { type: "button", onClick: save }, "Save publication"),
        React.createElement("button", { type: "button", onClick: () => details?.onCancel?.() }, "Cancel publication edit"),
        failure ? React.createElement("p", { role: "alert" }, failure) : null,
      );
    },
  };
});

const owner: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Owner",
  email: "owner@example.com",
  providerIds: ["google.com"],
  uid: "owner-uid",
};
const initialAuthState = cloudAuthStore.getState();

function setSignedIn(user = owner): void {
  cloudAuthStore.setState({
    ...initialAuthState,
    isConfigured: true,
    status: "signed_in",
    user,
  });
}

function ReturnProbe() {
  const location = useLocation();
  return <output aria-label="Return destination">{`${location.pathname}${location.search} ${JSON.stringify(location.state)}`}</output>;
}

function profile(overrides: Partial<PublicProfileSnapshot> = {}): PublicProfileSnapshot {
  return {
    uid: owner.uid,
    displayName: "Public name",
    enabled: true,
    generation: 3,
    revision: 9,
    updatedAt: "2026-10-22T12:00:00.000Z",
    sharedStats: null,
    catalogPageCounts: [0, 1, 0, 0],
    ...overrides,
  };
}

function page(entries: PublicCatalogEntry[], overrides: Partial<PublicCatalogPageSnapshot> = {}): PublicCatalogPageSnapshot {
  return {
    uid: owner.uid,
    pageId: 1,
    generation: 3,
    revision: 5,
    entries,
    updatedAt: "2026-10-22T12:00:00.000Z",
    ...overrides,
  };
}

async function publishedEntry(): Promise<PublicCatalogEntry> {
  const match = createLocalSavedMatch({
    id: "public-route-edit-test",
    localProfileId: "local-owner",
    moves: movesFromMoveCells([110, 0, 111, 1, 112, 2, 113, 3, 114]),
    players: [
      { kind: "human", name: "Publisher", stone: "black" },
      { kind: "bot", name: "Normal Bot", stone: "white" },
    ],
    ruleset: "freestyle",
    savedAt: "2026-10-21T00:00:00.000Z",
    status: "black_won",
  });
  const replay = portableReplayFromSavedMatch(match).replay;
  if (!replay || replay.black.kind !== "human") throw new Error("Public route fixture must have a human publisher.");
  const validated = validatePortableReplay({
    ...replay,
    viewPly: 4,
    black: { ...replay.black, profileUid: owner.uid },
  });
  return {
    fingerprint: await portableReplayFingerprint(validated),
    payload: encodePortableReplay(validated.replay),
    title: "Published replay",
    view_ply: 4,
  };
}

function renderEditor(fingerprint: string) {
  const origin = { fingerprint, pageId: 1, scrollTop: 432 };
  render(<MemoryRouter initialEntries={[{
    pathname: `/replay/published-edit-${fingerprint}`,
    search: "?page=1",
    state: { publishedReplayEditReturn: origin },
  }]}>
    <Routes>
      <Route path="/replay/:matchId" element={<PublishedReplayEditRoute fingerprint={fingerprint} />} />
      <Route path="/profile" element={<ReturnProbe />} />
    </Routes>
  </MemoryRouter>);
}

beforeEach(() => {
  setSignedIn();
  sharing.editPublishedReplay.mockReset();
  sharing.loadCatalogPage.mockReset();
  sharing.loadPublicProfile.mockReset();
});

afterEach(() => {
  cleanup();
  cloudAuthStore.setState(initialAuthState);
});

describe("published replay edit route", () => {
  it("returns an unchanged editor draft without starting a transaction", async () => {
    const entry = await publishedEntry();
    sharing.loadPublicProfile.mockResolvedValue(profile());
    sharing.loadCatalogPage.mockResolvedValue(page([entry]));
    renderEditor(entry.fingerprint);

    expect(await screen.findByRole("button", { name: "Save publication" })).toBeInTheDocument();
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Save publication" })));

    expect(await screen.findByLabelText("Return destination")).toHaveTextContent("/profile?tab=public");
    expect(screen.getByLabelText("Return destination")).toHaveTextContent('"fingerprint"');
    expect(screen.getByLabelText("Return destination")).toHaveTextContent('"scrollTop":432');
    expect(sharing.editPublishedReplay).not.toHaveBeenCalled();
  });

  it("keeps the draft visible on a stale transaction and returns only after save succeeds", async () => {
    const entry = await publishedEntry();
    sharing.loadPublicProfile.mockResolvedValue(profile());
    sharing.loadCatalogPage.mockResolvedValue(page([entry]));
    sharing.editPublishedReplay.mockRejectedValueOnce(new sharing.SharingConflictError("Public catalog changed on another device."));
    renderEditor(entry.fingerprint);

    await screen.findByRole("button", { name: "Save publication" });
    fireEvent.click(screen.getByRole("button", { name: "Change title" }));
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Save publication" })));

    expect(await screen.findByRole("alert")).toHaveTextContent("Public catalog changed on another device.");
    expect(screen.getByLabelText("Publication draft")).toHaveTextContent("Edited title");
    expect(screen.queryByLabelText("Return destination")).not.toBeInTheDocument();
    expect(sharing.editPublishedReplay).toHaveBeenCalledWith(
      owner.uid,
      1,
      entry.fingerprint,
      { title: "Edited title", opening_ply: 4 },
      9,
      3,
      5,
      {},
      expect.any(Function),
    );
    expect(sharing.editPublishedReplay.mock.calls[0]?.[8]()).toBe(true);

    sharing.editPublishedReplay.mockResolvedValueOnce({ profile: profile({ revision: 10 }), page: page([entry], { revision: 6 }) });
    await act(async () => fireEvent.click(screen.getByRole("button", { name: "Save publication" })));
    await waitFor(() => expect(screen.getByLabelText("Return destination")).toBeInTheDocument());
  });

  it("closes the editor when the active auth session changes, even for the same UID", async () => {
    const entry = await publishedEntry();
    sharing.loadPublicProfile.mockResolvedValue(profile());
    sharing.loadCatalogPage.mockResolvedValue(page([entry]));
    renderEditor(entry.fingerprint);
    await screen.findByRole("button", { name: "Save publication" });

    setSignedIn({ ...owner });

    expect(await screen.findByLabelText("Return destination")).toHaveTextContent("/profile?tab=public");
    expect(sharing.editPublishedReplay).not.toHaveBeenCalled();
  });
});
