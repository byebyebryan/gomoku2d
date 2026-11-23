import type { PrivateLibraryEntry } from "../cloud/sharing_private";

type ReplayIdentityFields = Pick<PrivateLibraryEntry, "payload" | "kind">;
type ReplayMetadataFields = Pick<PrivateLibraryEntry, "metadata">;

export function matchingFingerprintHasSamePayloadAndKind(
  left: ReplayIdentityFields,
  right: ReplayIdentityFields,
): boolean {
  return left.payload === right.payload && left.kind === right.kind;
}

export function replayEntriesHaveSameMetadata(
  left: ReplayMetadataFields,
  right: ReplayMetadataFields,
): boolean {
  // Merge plans compare entries after verification has normalized their metadata.
  return JSON.stringify(left.metadata) === JSON.stringify(right.metadata);
}

export function validateReplayMergeChoices(
  rows: readonly { fingerprint: string; status: string }[],
  choices: Readonly<Record<string, unknown>>,
  acceptedChoices: readonly [string, string],
  nonConflictMessage: string,
): void {
  for (const row of rows) {
    if (row.status === "metadata-conflict"
      && !acceptedChoices.some((choice) => choices[row.fingerprint] === choice)) {
      throw new Error("Choose one complete set of private details for every differing replay.");
    }
    if (row.status !== "metadata-conflict" && Object.prototype.hasOwnProperty.call(choices, row.fingerprint)) {
      throw new Error(nonConflictMessage);
    }
  }
  for (const fingerprint of Object.keys(choices)) {
    if (!rows.some((row) => row.fingerprint === fingerprint && row.status === "metadata-conflict")) {
      throw new Error("A metadata choice refers to a replay outside this review.");
    }
  }
}

export function mergeReplayEntries(
  left: PrivateLibraryEntry,
  right: PrivateLibraryEntry,
  preferred: "left" | "right",
): PrivateLibraryEntry {
  const selected = preferred === "left" ? left : right;
  return {
    ...selected,
    saved_at: left.saved_at < right.saved_at ? left.saved_at : right.saved_at,
  };
}
