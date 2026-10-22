import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { PortableReplayV1 } from "../replay/portable_replay";

const sharing = vi.hoisted(() => ({
  loadPrivateLibrary: vi.fn(),
  savePrivateLibrary: vi.fn(),
}));

vi.mock("../cloud/sharing_private", () => sharing);

const STORAGE_KEY = "gomoku2d.replay-library.v1";
const REPLAY: PortableReplayV1 = {
  black: { kind: "human", name: "Black" },
  white: { kind: "human", name: "White" },
  moves: [110, 0, 111, 1, 112, 2, 113, 3, 114],
  ruleset: "freestyle",
  version: 1,
};

async function setup(signedIn = false) {
  const { cloudAuthStore } = await import("../cloud/auth_store");
  cloudAuthStore.setState({
    status: signedIn ? "signed_in" : "signed_out",
    user: signedIn ? { uid: "library-reader", displayName: "Reader", avatarUrl: null, email: null, providerIds: [] } : null,
  });
  const { replayLibraryStore, createReplayLibraryStore } = await import("../replay/replay_library_store");
  const { replayLibrarySync } = await import("../replay/replay_library_sync");
  const { validatePortableReplay } = await import("../replay/portable_replay");
  const { portableReplayFingerprint } = await import("../replay/portable_replay_codec");
  const { SaveReplayButton, ReplayLibraryPanel } = await import("./ReplayLibraryPanel");
  const fingerprint = await portableReplayFingerprint(validatePortableReplay(REPLAY));
  render(<MemoryRouter><SaveReplayButton replay={REPLAY} fingerprint={fingerprint} /></MemoryRouter>);
  return { replayLibraryStore, replayLibrarySync, createReplayLibraryStore, ReplayLibraryPanel, fingerprint };
}

beforeEach(() => {
  // Each test needs a fresh best-effort store: storage failure is permanent for
  // that store's session, even if a later browser write would succeed.
  vi.resetModules();
  localStorage.clear();
  sharing.loadPrivateLibrary.mockReset().mockResolvedValue(null);
  sharing.savePrivateLibrary.mockReset();
});

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("replay save persistence messages", () => {
  it("confirms a device save only when the replay can be restored from storage", async () => {
    const { createReplayLibraryStore, fingerprint } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Save replay" }));

    expect(await screen.findByText("Saved on this device")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Saved replay" })).toBeDisabled();
    expect(createReplayLibraryStore().getState().owners.guest?.entries[0]?.fingerprint).toBe(fingerprint);
    expect(sharing.savePrivateLibrary).not.toHaveBeenCalled();
  });

  it("reports session-only saving when a write exceeds storage quota", async () => {
    const { replayLibraryStore, createReplayLibraryStore, fingerprint } = await setup();
    vi.spyOn(localStorage, "setItem").mockImplementation(() => {
      throw new DOMException("Storage full", "QuotaExceededError");
    });
    fireEvent.click(screen.getByRole("button", { name: "Save replay" }));

    expect(await screen.findByRole("button", { name: "Saved for this session" })).toBeDisabled();
    expect(screen.getByText(/lost when you reload or close this tab/i)).toBeInTheDocument();
    expect(screen.getByText(/keep the replay link/i)).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent("Storage full");
    expect(screen.queryByText(/saved on this device/i)).not.toBeInTheDocument();
    expect(replayLibraryStore.getState().owners.guest?.entries[0]?.fingerprint).toBe(fingerprint);
    expect(createReplayLibraryStore().getState().owners.guest).toBeUndefined();
  });

  it.each([
    ["malformed", "{bad-json"],
    ["newer", JSON.stringify({ schema_version: 99, owners: {} })],
  ])("discloses temporary storage before saving over an unusable %s cache", async (_label, raw) => {
    localStorage.setItem(STORAGE_KEY, raw);
    await setup();
    expect(screen.getByText(/saving keeps a copy for this session only/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Save replay" }));

    expect(await screen.findByRole("button", { name: "Saved for this session" })).toBeDisabled();
    expect(screen.getByText(/lost when you reload or close this tab/i)).toBeInTheDocument();
    expect(localStorage.getItem(STORAGE_KEY)).toBe(raw);
  });

  it("warns about session-only guest copies in the library after a storage failure", async () => {
    const { replayLibraryStore, ReplayLibraryPanel } = await setup();
    vi.spyOn(localStorage, "setItem").mockImplementation(() => { throw new Error("Writes blocked"); });
    await act(() => replayLibraryStore.getState().saveReplay("guest", REPLAY, "saved"));
    cleanup();
    render(<MemoryRouter><ReplayLibraryPanel /></MemoryRouter>);

    expect(screen.getByText(/guest copies are kept for this session only/i)).toBeInTheDocument();
    expect(screen.getByText(/lost when you reload or close this tab/i)).toBeInTheDocument();
    expect(screen.queryByText(/guest copies stay on this device/i)).not.toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Inspect replay" })).toBeInTheDocument();
  });

  it("keeps an unconfirmed account save session-only when local storage and cloud saving fail", async () => {
    const { replayLibraryStore, replayLibrarySync } = await setup(true);
    vi.spyOn(localStorage, "setItem").mockImplementation(() => { throw new Error("Writes blocked"); });
    sharing.savePrivateLibrary.mockRejectedValue(new Error("Offline"));
    fireEvent.click(screen.getByRole("button", { name: "Save replay" }));

    expect(await screen.findByRole("button", { name: "Saved for this session" })).toBeDisabled();
    expect(await screen.findByText(/account sync has not been confirmed/i)).toBeInTheDocument();
    expect(screen.getByText(/lost when you reload or close this tab/i)).toBeInTheDocument();
    expect(screen.queryByText(/saved in your account library/i)).not.toBeInTheDocument();
    await waitFor(() => expect(replayLibrarySync.store.getState().users["library-reader"]?.status).toBe("error"));
    expect(replayLibraryStore.getState().owners["user:library-reader"]?.dirty).toBe(true);
  });

  it("confirms a successful cloud save even when the device cache is unavailable", async () => {
    await setup(true);
    vi.spyOn(localStorage, "setItem").mockImplementation(() => { throw new Error("Writes blocked"); });
    sharing.savePrivateLibrary.mockImplementation(async (uid, entries) => ({
      uid, entries, revision: 1, updatedAt: "2026-10-21T00:00:00.000Z",
    }));
    fireEvent.click(screen.getByRole("button", { name: "Save replay" }));

    expect(await screen.findByText(/saved in your account library/i)).toBeInTheDocument();
    expect(screen.getByText(/could not be stored on this device/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Saved replay" })).toBeDisabled();
    expect(screen.queryByText(/lost when you reload/i)).not.toBeInTheDocument();
    expect(sharing.savePrivateLibrary).toHaveBeenCalledOnce();
  });
});
