import type { ReviewMedia } from "./review_record.js";

export type HistoryLayout = "desktop" | "portrait";
export interface HistoryRelease {
  version: string;
  date: string;
  commit: string;
}
export interface HistoryScreen {
  id: string;
  title: string;
  description: string;
  states: { id: string; title: string }[];
}
export interface HistorySnapshot {
  id: string;
  screen: string;
  state: string;
  version: string;
  conditions: string;
  capturedAt: string;
  captureType: "retrospective" | "historical";
  manifest: { sourcePath: string; sha256: string };
  media: Partial<Record<HistoryLayout, ReviewMedia>>;
  unavailable?: Partial<Record<HistoryLayout, string>>;
}
export interface HistoryStateChange {
  id: string;
  summary: string;
  before?: string;
  after?: string;
}
export interface HistoryChange {
  id: string;
  screen: string;
  kind: "added" | "updated" | "removed";
  summary: string;
  states: HistoryStateChange[];
  related?: {
    screen: string;
    relation:
      "replaces" | "replaced-by" | "merged-into" | "split-into" | "split-from";
  }[];
  sources?: { label: string; url: string }[];
}
export interface HistoryMilestone {
  version: string;
  title: string;
  summary: string;
  changes: HistoryChange[];
}
export interface HistoryCatalog {
  schema: 1;
  summary: string;
  releases: HistoryRelease[];
  screens: HistoryScreen[];
  snapshots: HistorySnapshot[];
  milestones: HistoryMilestone[];
}

const object = (v: unknown): v is Record<string, unknown> =>
  !!v && typeof v === "object" && !Array.isArray(v);
const text = (v: unknown): v is string =>
  typeof v === "string" && v.trim().length > 0;
const name = (v: unknown): v is string =>
  text(v) && /^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(v) && !v.includes("..");
const sourcePath = (v: unknown): v is string =>
  text(v) &&
  !v.startsWith("/") &&
  !v.includes("\\") &&
  v
    .split("/")
    .every((part) => part !== ".." && part !== "." && part.length > 0);
const digest = (v: unknown): v is string =>
  typeof v === "string" && /^[a-f0-9]{64}$/.test(v);
const version = (v: unknown): v is string =>
  typeof v === "string" && /^0\.[2-6]\.\d+$/.test(v);
const date = (v: unknown): v is string =>
  typeof v === "string" &&
  /^\d{4}-\d{2}-\d{2}(?:T.*)?$/.test(v) &&
  Number.isFinite(Date.parse(v));
const unique = <T>(values: T[]) => new Set(values).size === values.length;
const orderedVersions = (values: string[]) =>
  values.every((value, index) => {
    if (!index) return true;
    const a = values[index - 1].split(".").map(Number),
      b = value.split(".").map(Number);
    return a[1] < b[1] || (a[1] === b[1] && a[2] < b[2]);
  });
function pinnedSourceCommit(value: string): string | undefined {
  try {
    const url = new URL(value);
    if (url.origin !== "https://github.com" || url.username || url.password)
      return;
    return /^\/byebyebryan\/gomoku2d\/(?:blob|tree)\/([a-f0-9]{40})\/.+/.exec(
      url.pathname,
    )?.[1];
  } catch {
    return;
  }
}
function isMedia(v: unknown): v is ReviewMedia {
  return (
    object(v) &&
    name(v.file) &&
    v.file.endsWith(".png") &&
    sourcePath(v.sourcePath) &&
    digest(v.sha256) &&
    Number.isInteger(v.width) &&
    Number(v.width) > 0 &&
    Number.isInteger(v.height) &&
    Number(v.height) > 0 &&
    [v.label, v.alt, v.revision].every(text) &&
    v.kind === "application capture"
  );
}
function isRelease(v: unknown): v is HistoryRelease {
  return (
    object(v) &&
    version(v.version) &&
    date(v.date) &&
    typeof v.commit === "string" &&
    /^[a-f0-9]{40}$/.test(v.commit)
  );
}
function isScreen(v: unknown): v is HistoryScreen {
  return (
    object(v) &&
    name(v.id) &&
    text(v.title) &&
    text(v.description) &&
    Array.isArray(v.states) &&
    v.states.length > 0 &&
    v.states.every((s) => object(s) && name(s.id) && text(s.title)) &&
    unique(v.states.map((s) => (s as { id: string }).id))
  );
}
function isSnapshot(v: unknown): v is HistorySnapshot {
  if (
    !object(v) ||
    ![v.id, v.screen, v.state].every(name) ||
    !version(v.version) ||
    !text(v.conditions) ||
    !date(v.capturedAt) ||
    !["retrospective", "historical"].includes(String(v.captureType)) ||
    !object(v.manifest) ||
    !sourcePath(v.manifest.sourcePath) ||
    !digest(v.manifest.sha256) ||
    !object(v.media) ||
    !Object.keys(v.media).length ||
    !Object.entries(v.media).every(
      ([layout, media]) =>
        ["desktop", "portrait"].includes(layout) && isMedia(media),
    )
  )
    return false;
  if (
    v.unavailable !== undefined &&
    (!object(v.unavailable) ||
      !Object.entries(v.unavailable).every(
        ([layout, reason]) =>
          ["desktop", "portrait"].includes(layout) &&
          text(reason) &&
          !(layout in (v.media as Record<string, unknown>)),
      ))
  )
    return false;
  return ["desktop", "portrait"].every(
    (layout) =>
      layout in (v.media as Record<string, unknown>) ||
      (object(v.unavailable) && layout in v.unavailable),
  );
}
function isChange(v: unknown): v is HistoryChange {
  if (
    !object(v) ||
    !name(v.id) ||
    !name(v.screen) ||
    !text(v.summary) ||
    !["added", "updated", "removed"].includes(String(v.kind)) ||
    !Array.isArray(v.states) ||
    !v.states.length ||
    !v.states.every(
      (s) =>
        object(s) &&
        name(s.id) &&
        text(s.summary) &&
        (s.before === undefined || name(s.before)) &&
        (s.after === undefined || name(s.after)) &&
        (s.before !== undefined || s.after !== undefined) &&
        (v.kind !== "added" ||
          (s.before === undefined && s.after !== undefined)) &&
        (v.kind !== "removed" ||
          (s.before !== undefined && s.after === undefined)),
    )
  )
    return false;
  if (!unique(v.states.map((s) => (s as { id: string }).id))) return false;
  if (
    v.kind === "updated" &&
    (!v.states.some(
      (s) => (s as Record<string, unknown>).before !== undefined,
    ) ||
      !v.states.some((s) => (s as Record<string, unknown>).after !== undefined))
  )
    return false;
  if (
    v.related !== undefined &&
    (!Array.isArray(v.related) ||
      !v.related.every(
        (r) =>
          object(r) &&
          name(r.screen) &&
          [
            "replaces",
            "replaced-by",
            "merged-into",
            "split-into",
            "split-from",
          ].includes(String(r.relation)),
      ))
  )
    return false;
  return (
    v.sources === undefined ||
    (Array.isArray(v.sources) &&
      v.sources.every(
        (s) =>
          object(s) &&
          text(s.label) &&
          text(s.url) &&
          pinnedSourceCommit(s.url) !== undefined,
      ))
  );
}
function isMilestone(v: unknown): v is HistoryMilestone {
  return (
    object(v) &&
    version(v.version) &&
    text(v.title) &&
    text(v.summary) &&
    Array.isArray(v.changes) &&
    v.changes.length > 0 &&
    v.changes.every(isChange) &&
    unique(v.changes.map((c) => (c as HistoryChange).id)) &&
    unique(v.changes.map((c) => (c as HistoryChange).screen))
  );
}

/** Validate the static public reference graph, including lifecycle/source identity. */
export function isHistoryCatalog(value: unknown): value is HistoryCatalog {
  if (
    !object(value) ||
    value.schema !== 1 ||
    !text(value.summary) ||
    !Array.isArray(value.releases) ||
    !value.releases.every(isRelease) ||
    !Array.isArray(value.screens) ||
    !value.screens.every(isScreen) ||
    !Array.isArray(value.snapshots) ||
    !value.snapshots.every(isSnapshot) ||
    !Array.isArray(value.milestones) ||
    !value.milestones.every(isMilestone)
  )
    return false;
  const catalog = value as unknown as HistoryCatalog;
  if (
    !unique(catalog.releases.map((r) => r.version)) ||
    !orderedVersions(catalog.releases.map((r) => r.version)) ||
    !unique(catalog.screens.map((s) => s.id)) ||
    !unique(catalog.snapshots.map((s) => s.id)) ||
    !unique(catalog.milestones.map((m) => m.version)) ||
    !orderedVersions(catalog.milestones.map((m) => m.version))
  )
    return false;
  const releases = new Map(
    catalog.releases.map((r, index) => [r.version, { ...r, index }]),
  );
  const screens = new Map(catalog.screens.map((s) => [s.id, s]));
  const snapshots = new Map(catalog.snapshots.map((s) => [s.id, s]));
  const files = new Map<string, string>();
  for (const snapshot of catalog.snapshots) {
    const release = releases.get(snapshot.version),
      screen = screens.get(snapshot.screen);
    if (!release || !screen?.states.some((s) => s.id === snapshot.state))
      return false;
    for (const media of Object.values(snapshot.media)) {
      if (
        !release.commit.startsWith(media.revision) ||
        media.revision.length < 7
      )
        return false;
      if (files.has(media.file) && files.get(media.file) !== media.sha256)
        return false;
      files.set(media.file, media.sha256);
    }
  }
  const used = new Set<string>();
  const commits = new Set(catalog.releases.map((release) => release.commit));
  for (const milestone of catalog.milestones) {
    const release = releases.get(milestone.version);
    if (!release) return false;
    for (const change of milestone.changes) {
      const screen = screens.get(change.screen);
      if (
        !screen ||
        change.sources?.some(
          (source) => !commits.has(pinnedSourceCommit(source.url)!),
        ) ||
        change.related?.some(
          (r) => !screens.has(r.screen) || r.screen === screen.id,
        )
      )
        return false;
      for (const state of change.states) {
        if (!screen.states.some((s) => s.id === state.id)) return false;
        for (const [side, reference] of [
          ["before", state.before],
          ["after", state.after],
        ] as const) {
          if (!reference) continue;
          const snapshot = snapshots.get(reference);
          if (
            !snapshot ||
            snapshot.screen !== screen.id ||
            snapshot.state !== state.id
          )
            return false;
          const source = releases.get(snapshot.version);
          if (
            !source ||
            (side === "before"
              ? source.index >= release.index
              : source.index !== release.index)
          )
            return false;
          used.add(reference);
        }
      }
    }
  }
  return catalog.snapshots.every((s) => used.has(s.id));
}

export function snapshotMedia(
  snapshot: HistorySnapshot,
  layout: HistoryLayout,
): ReviewMedia | undefined {
  return snapshot.media[layout];
}
