export interface ReviewMedia {
  file: string;
  sourcePath: string;
  sha256: string;
  width: number;
  height: number;
  label: string;
  alt: string;
  kind: "application capture" | "asset render" | "design mockup";
  revision: string;
}
export interface ReviewComparison {
  id: string;
  title: string;
  conditions: string;
  purpose?: "layout-check";
  media: ReviewMedia[];
}
export interface ReviewRecord {
  schema: 1;
  id: string;
  title: string;
  scope: string;
  date: string;
  revision: string;
  question: string;
  summary: string;
  disposition:
    | "accepted"
    | "historical checkpoint"
    | "screen reference"
    | "retrospective comparison";
  comparisons: ReviewComparison[];
  findings: { state: "fixed" | "deferred" | "open" | "recorded"; title: string; detail: string }[];
  technical: string[];
  human: string;
  followUp: string;
  sources: { label: string; url: string }[];
}
const object = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === "object" && !Array.isArray(value);
const text = (value: unknown): value is string =>
  typeof value === "string" && value.trim().length > 0;
const safeName = (value: unknown): value is string =>
  text(value) && /^[a-zA-Z0-9][a-zA-Z0-9_.-]*$/.test(value) && !value.includes("..");
export function isReviewRecord(value: unknown): value is ReviewRecord {
  if (!object(value) || value.schema !== 1 || !safeName(value.id)) return false;
  if (
    !["title", "scope", "date", "revision", "question", "summary", "human", "followUp"].every(
      (key) => text(value[key]),
    )
  )
    return false;
  if (
    !["accepted", "historical checkpoint", "screen reference", "retrospective comparison"].includes(
      String(value.disposition),
    )
  )
    return false;
  if (
    !Array.isArray(value.comparisons) ||
    !value.comparisons.length ||
    !value.comparisons.every(
      (pair) =>
        object(pair) &&
        safeName(pair.id) &&
        text(pair.title) &&
        text(pair.conditions) &&
        (pair.purpose === undefined || pair.purpose === "layout-check") &&
        Array.isArray(pair.media) &&
        pair.media.length > 0 &&
        pair.media.length <= 2 &&
        pair.media.every(
          (media) =>
            object(media) &&
            safeName(media.file) &&
            text(media.sourcePath) &&
            !media.sourcePath.startsWith("/") &&
            !media.sourcePath.split("/").includes("..") &&
            /^[a-f0-9]{64}$/.test(String(media.sha256)) &&
            Number.isInteger(media.width) &&
            Number(media.width) > 0 &&
            Number.isInteger(media.height) &&
            Number(media.height) > 0 &&
            ["label", "alt", "revision"].every((key) => text(media[key])) &&
            ["application capture", "asset render", "design mockup"].includes(String(media.kind)),
        ),
    )
  )
    return false;
  if (
    new Set(value.comparisons.map((pair) => (pair as { id: string }).id)).size !==
    value.comparisons.length
  )
    return false;
  if (
    !Array.isArray(value.findings) ||
    !value.findings.every(
      (item) =>
        object(item) &&
        ["fixed", "deferred", "open", "recorded"].includes(String(item.state)) &&
        text(item.title) &&
        text(item.detail),
    )
  )
    return false;
  if (!Array.isArray(value.technical) || !value.technical.every(text)) return false;
  return (
    Array.isArray(value.sources) &&
    value.sources.every(
      (source) =>
        object(source) &&
        text(source.label) &&
        text(source.url) &&
        /^https:\/\/github\.com\/byebyebryan\/gomoku2d\//.test(source.url),
    )
  );
}
