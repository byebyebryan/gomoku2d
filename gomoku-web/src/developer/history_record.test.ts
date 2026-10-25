import { describe, expect, it } from "vitest";
import { isHistoryCatalog, type HistoryCatalog } from "./history_record";

function fixture(): HistoryCatalog {
  const commitA = "a".repeat(40),
    commitB = "b".repeat(40),
    hash = "c".repeat(64);
  return {
    schema: 1,
    summary: "Source-backed screen history.",
    releases: [
      { version: "0.2.1", date: "2026-04-22", commit: commitA },
      { version: "0.2.2", date: "2026-04-22", commit: commitB },
    ],
    screens: [
      {
        id: "record",
        title: "Record",
        description: "A record screen.",
        states: [{ id: "default", title: "Record" }],
      },
    ],
    snapshots: [
      {
        id: "record-021",
        screen: "record",
        state: "default",
        version: "0.2.1",
        capturedAt: "2026-10-25T00:00:00Z",
        captureType: "retrospective",
        conditions: "Synthetic guest, matched CSS viewport.",
        manifest: { sourcePath: "docs/capture.json", sha256: hash },
        media: {
          desktop: {
            file: "record-021.png",
            sourcePath: "docs/record.png",
            sha256: hash,
            width: 1280,
            height: 900,
            label: "v0.2.1",
            alt: "The actual record screen",
            kind: "application capture",
            revision: commitA,
          },
        },
        unavailable: {
          portrait: "Only the original desktop capture is available.",
        },
      },
    ],
    milestones: [
      {
        version: "0.2.1",
        title: "Introduce the record",
        summary: "Record is added.",
        changes: [
          {
            id: "record",
            screen: "record",
            kind: "added",
            summary: "Record is added.",
            states: [
              {
                id: "default",
                summary: "The original record.",
                after: "record-021",
              },
            ],
          },
        ],
      },
      {
        version: "0.2.2",
        title: "Retire the record",
        summary: "Record is removed.",
        changes: [
          {
            id: "record",
            screen: "record",
            kind: "removed",
            summary: "Record is removed.",
            states: [
              {
                id: "default",
                summary: "Last existing view; removed in 0.2.2.",
                before: "record-021",
              },
            ],
          },
        ],
      },
    ],
  };
}

describe("source-backed UI history", () => {
  it("represents introduction and retirement with one shared original capture", () => {
    expect(isHistoryCatalog(fixture())).toBe(true);
  });
  it("keeps absence distinct from unrecorded layout coverage", () => {
    const value = fixture();
    delete value.snapshots[0].unavailable;
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("requires added/removed lifecycle sides to agree", () => {
    const value = fixture();
    value.milestones[0].changes[0].states[0].before = "record-021";
    expect(isHistoryCatalog(value)).toBe(false);
    delete value.milestones[0].changes[0].states[0].before;
    value.milestones[1].changes[0].states[0].after = "record-021";
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("rejects source versions and screen states that do not match their transition", () => {
    const value = fixture();
    value.snapshots[0].version = "0.2.2";
    expect(isHistoryCatalog(value)).toBe(false);
    value.snapshots[0].version = "0.2.1";
    value.snapshots[0].state = "account";
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("requires a real before and after for an update across its states", () => {
    const value = fixture();
    value.milestones[1].changes[0].kind = "updated";
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("rejects future baselines, unknown snapshots and foreign source revisions", () => {
    const value = fixture();
    value.milestones[1].changes[0].states[0].before = "missing";
    expect(isHistoryCatalog(value)).toBe(false);
    value.milestones[1].changes[0].states[0].before = "record-021";
    value.snapshots[0].media.desktop!.revision = "b".repeat(40);
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it.each([
    "../secret.png",
    "/tmp/private.png",
    "docs/../private.png",
    "docs\\private.png",
  ])(
    "rejects evidence outside the explicit repository allowlist: %s",
    (path) => {
      const value = fixture();
      value.snapshots[0].media.desktop!.sourcePath = path;
      expect(isHistoryCatalog(value)).toBe(false);
    },
  );
  it("validates replacement relationships without forcing a second image", () => {
    const value = fixture();
    value.screens.push({
      id: "lab",
      title: "Lab",
      description: "A replacement.",
      states: [{ id: "default", title: "Ranking" }],
    });
    value.milestones[1].changes[0].related = [
      { screen: "lab", relation: "merged-into" },
    ];
    expect(isHistoryCatalog(value)).toBe(true);
    value.milestones[1].changes[0].related![0].screen = "missing";
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("rejects duplicate/unsorted sources, unreferenced media and nonpublic evidence links", () => {
    const value = fixture();
    value.releases.reverse();
    expect(isHistoryCatalog(value)).toBe(false);
    value.releases.reverse();
    value.snapshots.push({ ...value.snapshots[0], id: "unused" });
    expect(isHistoryCatalog(value)).toBe(false);
    value.snapshots.pop();
    value.milestones[0].changes[0].sources = [
      { label: "Private host", url: "http://starship:8787/private" },
    ];
    expect(isHistoryCatalog(value)).toBe(false);
  });
  it("pins evidence links to a known release commit instead of a mutable branch", () => {
    const value = fixture();
    const source = {
      label: "Record source",
      url: `https://github.com/byebyebryan/gomoku2d/blob/${"a".repeat(40)}/gomoku-web/src/Record.tsx`,
    };
    value.milestones[0].changes[0].sources = [source];
    expect(isHistoryCatalog(value)).toBe(true);
    source.url = source.url.replace("a".repeat(40), "main");
    expect(isHistoryCatalog(value)).toBe(false);
    source.url = source.url.replace("main", "d".repeat(40));
    expect(isHistoryCatalog(value)).toBe(false);
  });
});
