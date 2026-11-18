import { describe, expect, it } from "vitest";

import { privateLibraryJson } from "./sharing_common";
import {
  PRIVATE_LIBRARY_SCHEMA_VERSION,
  parsePrivateLibrarySnapshot,
  privateLibraryV2Json,
  validatePrivateLibraryEntries,
  type PrivateLibraryEntry,
} from "./sharing_private";

const entry: PrivateLibraryEntry = {
  fingerprint: "A".repeat(43),
  payload: "AQID",
  kind: "saved",
  saved_at: "2026-10-21T12:00:00.000Z",
  metadata: { title: "Fin ✨", note: "Reply here.\nThen review.", opening_ply: 0 },
};

function document(schemaVersion: number, entriesJson: string, extra: Record<string, unknown> = {}) {
  return {
    schema_version: schemaVersion,
    uid: "uid-1",
    revision: 4,
    updated_at: new Date("2026-10-21T12:00:00.000Z"),
    entries_json: entriesJson,
    ...extra,
  };
}

describe("private replay library v2 codec", () => {
  it("writes canonical base64url metadata while keeping entry and document shape exact", () => {
    const json = privateLibraryV2Json([entry]);
    expect(JSON.parse(json)).toEqual([{
      fingerprint: entry.fingerprint,
      payload: entry.payload,
      kind: entry.kind,
      saved_at: entry.saved_at,
      metadata: {
        title: "RmluIOKcqA",
        note: "UmVwbHkgaGVyZS4KVGhlbiByZXZpZXcu",
        opening_ply: 0,
      },
    }]);
    expect(json).toMatch(/^[\x20-\x7e]*$/);
    const snapshot = parsePrivateLibrarySnapshot(document(PRIVATE_LIBRARY_SCHEMA_VERSION, json), "uid-1");
    expect(snapshot.entries).toEqual([entry]);
  });

  it("reads v1 cloud entries as default details and supports v1 callers during the upgrade", () => {
    const legacy = {
      fingerprint: entry.fingerprint,
      payload: entry.payload,
      kind: entry.kind,
      saved_at: entry.saved_at,
    };
    const snapshot = parsePrivateLibrarySnapshot(document(1, privateLibraryJson([legacy])), "uid-1");
    expect(snapshot.entries).toEqual([{ ...legacy, metadata: { title: "", note: "", opening_ply: null } }]);
    expect(validatePrivateLibraryEntries([legacy])).toEqual(snapshot.entries);
    expect(privateLibraryV2Json([legacy])).toContain('"opening_ply":null');
  });

  it("rejects noncanonical base64url and unknown metadata fields", () => {
    const body = JSON.parse(privateLibraryV2Json([entry])) as Array<Record<string, unknown>>;
    (body[0]!.metadata as Record<string, unknown>).title = "AB";
    expect(() => parsePrivateLibrarySnapshot(document(2, JSON.stringify(body)), "uid-1")).toThrow(/trailing base64url/i);

    const withUnknown = JSON.parse(privateLibraryV2Json([entry])) as Array<Record<string, unknown>>;
    (withUnknown[0]!.metadata as Record<string, unknown>).extra = true;
    expect(() => parsePrivateLibrarySnapshot(document(2, JSON.stringify(withUnknown)), "uid-1")).toThrow(/metadata has an invalid shape/i);
  });

  it("rejects malformed cloud document and entry fields without accepting future versions", () => {
    const json = privateLibraryV2Json([entry]);
    expect(() => parsePrivateLibrarySnapshot(document(3, json), "uid-1")).toThrow(/unsupported schema/i);
    expect(() => parsePrivateLibrarySnapshot(document(2, json, { extra: true }), "uid-1")).toThrow(/document shape/i);
    const body = JSON.parse(json) as Array<Record<string, unknown>>;
    body[0]!.unexpected = true;
    expect(() => parsePrivateLibrarySnapshot(document(2, JSON.stringify(body)), "uid-1")).toThrow(/invalid shape/i);
  });

  it("normalizes new details and rejects metadata that exceeds bounds or contains forbidden text", () => {
    expect(validatePrivateLibraryEntries([{
      ...entry,
      metadata: { title: "  Study  ", note: "  first\r\nsecond  ", opening_ply: 2 },
    }])[0]?.metadata).toEqual({ title: "Study", note: "first\nsecond", opening_ply: 2 });
    expect(() => validatePrivateLibraryEntries([{
      ...entry,
      metadata: { title: "x".repeat(81), note: "", opening_ply: null },
    }])).toThrow(/title/i);
    expect(() => validatePrivateLibraryEntries([{
      ...entry,
      metadata: { title: "", note: "bad\u2028break", opening_ply: null },
    }])).toThrow(/control character/i);
    expect(() => validatePrivateLibraryEntries([{
      ...entry,
      metadata: { title: "", note: "", opening_ply: 1 },
    }])).not.toThrow();
  });
});
