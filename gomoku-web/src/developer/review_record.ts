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
  captureType?: "historical" | "retrospective";
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
  versions?: string[];
  question: string;
  summary: string;
  disposition:
    | "accepted"
    | "historical checkpoint"
    | "screen reference"
    | "retrospective comparison"
    | "retrospective review";
  comparisons: ReviewComparison[];
  process?: { id: string; title: string; detail: string; comparisons: string[] }[];
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

const canonicalVersionParts = (value: unknown): string[] | undefined => {
  if (typeof value !== "string") return undefined;
  const match = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/.exec(value);
  return match ? [match[1], match[2], match[3]] : undefined;
};

const compareVersionParts = (left: string[], right: string[]): number => {
  for (let index = 0; index < left.length; index += 1) {
    if (left[index].length !== right[index].length)
      return left[index].length < right[index].length ? -1 : 1;
    if (left[index] !== right[index]) return left[index] < right[index] ? -1 : 1;
  }
  return 0;
};

const isAscendingVersionList = (value: unknown): value is string[] => {
  if (!Array.isArray(value) || value.length === 0) return false;
  const parts = value.map(canonicalVersionParts);
  if (parts.some((part) => part === undefined)) return false;
  return parts.slice(1).every((part, index) => compareVersionParts(parts[index]!, part!) < 0);
};

function isReviewProcess(
  value: unknown,
  comparisons: ReviewComparison[],
): value is NonNullable<ReviewRecord["process"]> {
  if (!Array.isArray(value) || value.length === 0) return false;
  const normalComparisonIds = new Set(
    comparisons.filter((pair) => pair.purpose !== "layout-check").map((pair) => pair.id),
  );
  const stepIds = new Set<string>();
  const referencedComparisonIds = new Set<string>();

  for (const step of value) {
    if (
      !object(step) ||
      !safeName(step.id) ||
      !text(step.title) ||
      !text(step.detail) ||
      !Array.isArray(step.comparisons) ||
      step.comparisons.length === 0 ||
      stepIds.has(step.id)
    )
      return false;
    stepIds.add(step.id);

    for (const comparisonId of step.comparisons) {
      if (
        !text(comparisonId) ||
        !normalComparisonIds.has(comparisonId) ||
        referencedComparisonIds.has(comparisonId)
      )
        return false;
      referencedComparisonIds.add(comparisonId);
    }
  }

  return referencedComparisonIds.size === normalComparisonIds.size;
}

export function isReviewRecord(value: unknown): value is ReviewRecord {
  if (!object(value) || value.schema !== 1 || !safeName(value.id)) return false;
  if (
    !["title", "scope", "date", "revision", "question", "summary", "human", "followUp"].every(
      (key) => text(value[key]),
    )
  )
    return false;
  if (
    ![
      "accepted",
      "historical checkpoint",
      "screen reference",
      "retrospective comparison",
      "retrospective review",
    ].includes(String(value.disposition))
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
            ["application capture", "asset render", "design mockup"].includes(String(media.kind)) &&
            (media.captureType === undefined ||
              (media.kind === "application capture" &&
                ["historical", "retrospective"].includes(String(media.captureType)))),
        ),
    )
  )
    return false;
  if (
    new Set(value.comparisons.map((pair) => (pair as { id: string }).id)).size !==
    value.comparisons.length
  )
    return false;
  const comparisons = value.comparisons as unknown as ReviewComparison[];
  if (value.versions !== undefined && !isAscendingVersionList(value.versions)) return false;
  if (value.process !== undefined && !isReviewProcess(value.process, comparisons)) return false;
  if (
    value.disposition === "retrospective review" &&
    (value.versions === undefined || value.process === undefined)
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
