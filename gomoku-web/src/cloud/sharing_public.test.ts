import { beforeEach, describe, expect, it, vi } from "vitest";

import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import { portableReplayFromSavedMatch, validatePortableReplay } from "../replay/portable_replay";
import {
  decodePortableReplay,
  encodePortableReplay,
  portableReplayFingerprint,
} from "../replay/portable_replay_codec";
import {
  editPublishedReplay,
  type PublicCatalogEntry,
} from "./sharing_public";
import { SharingConflictError } from "./sharing_common";
import type { Firestore } from "firebase/firestore";

const firestoreApi = vi.hoisted(() => ({
  doc: vi.fn((_firestore: unknown, ...path: unknown[]) => path.map(String).join("/")),
  getDocFromServer: vi.fn(),
  runTransaction: vi.fn(),
  serverTimestamp: vi.fn(() => ({ serverTimestamp: true })),
}));

vi.mock("firebase/firestore", () => firestoreApi);

interface WriteRecord {
  path: string;
  value: Record<string, unknown>;
}

function transactionFor(docs: Map<string, Record<string, unknown>>) {
  const writes: WriteRecord[] = [];
  const transaction = {
    get: vi.fn(async (reference: unknown) => {
      const path = String(reference);
      const value = docs.get(path);
      return { exists: () => value !== undefined, data: () => value };
    }),
    set: vi.fn((reference: unknown, value: Record<string, unknown>) => {
      writes.push({ path: String(reference), value });
    }),
  };
  firestoreApi.runTransaction.mockImplementation(async (_firestore: unknown, callback: (tx: typeof transaction) => Promise<unknown>) => callback(transaction));
  return { transaction, writes };
}

function rootDocument(overrides: Record<string, unknown> = {}) {
  return {
    schema_version: 1,
    uid: "uid-1",
    display_name: "Public name",
    enabled: true,
    generation: 2,
    revision: 7,
    updated_at: new Date("2026-10-22T12:00:00.000Z"),
    shared_stats: {
      matches: 4,
      wins: 2,
      losses: 1,
      draws: 1,
      updated_at: new Date("2026-10-21T12:00:00.000Z"),
    },
    catalog_page_counts: [1, 0, 0, 0],
    ...overrides,
  };
}

function legacyRootDocument() {
  const root = { ...rootDocument() } as Record<string, unknown>;
  delete root.catalog_page_counts;
  return root;
}

function pageDocument(entries: PublicCatalogEntry[], overrides: Record<string, unknown> = {}) {
  return {
    schema_version: 1,
    generation: 2,
    revision: 4,
    updated_at: new Date("2026-10-22T12:00:00.000Z"),
    entries,
    ...overrides,
  };
}

async function publication(viewPly?: number) {
  const match = createLocalSavedMatch({
    id: "public-replay-edit-test",
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
  if (!replay || replay.black.kind !== "human") throw new Error("Public replay fixture must have a human publisher.");
  const validated = validatePortableReplay({
    ...replay,
    black: { ...replay.black, profileUid: "uid-1" },
    ...(viewPly === undefined ? {} : { viewPly }),
  });
  const fingerprint = await portableReplayFingerprint(validated);
  const payload = encodePortableReplay(validated.replay);
  return {
    fingerprint,
    payload,
    entry: { fingerprint, payload, title: "Original title", view_ply: viewPly ?? 0 },
  };
}

function documentStore(entry: PublicCatalogEntry, root = rootDocument()) {
  return new Map<string, Record<string, unknown>>([
    ["public_profiles/uid-1", root],
    ["public_profiles/uid-1/pages/0", pageDocument([entry])],
  ]);
}

beforeEach(() => {
  vi.clearAllMocks();
});

describe("revision-bound public replay edits", () => {
  it("keeps legacy title-only edits byte-stable and leaves the legacy root manifest absent", async () => {
    const existing = await publication();
    const docs = new Map<string, Record<string, unknown>>([
      ["public_profiles/uid-1", legacyRootDocument()],
      ["public_profiles/uid-1/pages/0", pageDocument([existing.entry])],
    ]);
    const legacyHarness = transactionFor(docs);

    const result = await editPublishedReplay(
      "uid-1", 0, existing.fingerprint, { title: " Renamed legacy game ", opening_ply: null },
      7, 2, 4, { firestore: {} as Firestore },
    );

    expect(result.page.entries[0]).toMatchObject({
      fingerprint: existing.fingerprint,
      payload: existing.payload,
      title: "Renamed legacy game",
      view_ply: 0,
    });
    expect(legacyHarness.transaction.get).toHaveBeenCalledTimes(2);
    expect(legacyHarness.writes).toHaveLength(2);
    const savedRoot = legacyHarness.writes.find((write) => write.path === "public_profiles/uid-1")?.value;
    expect(savedRoot).not.toHaveProperty("catalog_page_counts");
    expect(savedRoot?.shared_stats).toEqual(expect.objectContaining({ matches: 4, wins: 2 }));
  });

  it("writes a matching canonical at and view_ply for move zero without changing the fingerprint", async () => {
    const existing = await publication(4);
    const harness = transactionFor(documentStore(existing.entry));
    const result = await editPublishedReplay(
      "uid-1", 0, existing.fingerprint, { title: "Original title", opening_ply: 0 },
      7, 2, 4, { firestore: {} as Firestore },
    );

    const edited = result.page.entries[0]!;
    const decoded = decodePortableReplay(edited.payload);
    expect(edited).toMatchObject({ fingerprint: existing.fingerprint, title: "Original title", view_ply: 0 });
    expect(edited.payload).not.toBe(existing.payload);
    expect(decoded.replay.viewPly).toBe(0);
    expect(await portableReplayFingerprint(decoded)).toBe(existing.fingerprint);
    expect(harness.transaction.get).toHaveBeenCalledTimes(2);
    expect(harness.writes).toHaveLength(2);
  });

  it("does not write for an unchanged title and opening after checking revisions and fingerprint", async () => {
    const existing = await publication(4);
    const harness = transactionFor(documentStore(existing.entry));
    const result = await editPublishedReplay(
      "uid-1", 0, existing.fingerprint, { title: existing.entry.title, opening_ply: 4 },
      7, 2, 4, { firestore: {} as Firestore },
    );

    expect(harness.transaction.get).toHaveBeenCalledTimes(2);
    expect(harness.writes).toHaveLength(0);
    expect(result.profile.revision).toBe(7);
    expect(result.page.revision).toBe(4);
    expect(result.page.entries[0]?.payload).toBe(existing.payload);
  });

  it("rejects a stale root revision without writing or restoring a publication", async () => {
    const existing = await publication(4);
    const harness = transactionFor(documentStore(existing.entry));

    await expect(editPublishedReplay(
      "uid-1", 0, existing.fingerprint, { title: "Stale edit", opening_ply: 0 },
      6, 2, 4, { firestore: {} as Firestore },
    )).rejects.toBeInstanceOf(SharingConflictError);

    expect(harness.writes).toHaveLength(0);
  });

  it("cannot recreate a publication removed from the current page", async () => {
    const existing = await publication(4);
    const docs = new Map<string, Record<string, unknown>>([
      ["public_profiles/uid-1", rootDocument({ catalog_page_counts: [0, 0, 0, 0] })],
      ["public_profiles/uid-1/pages/0", pageDocument([])],
    ]);
    const harness = transactionFor(docs);

    await expect(editPublishedReplay(
      "uid-1", 0, existing.fingerprint, { title: "Restore it", opening_ply: 0 },
      7, 2, 4, { firestore: {} as Firestore },
    )).rejects.toThrow(/removed/i);

    expect(harness.writes).toHaveLength(0);
  });
});
