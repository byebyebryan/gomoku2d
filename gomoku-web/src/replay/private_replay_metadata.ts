export interface PrivateReplayMetadata {
  title: string;
  note: string;
  opening_ply: number | null;
}

export const DEFAULT_PRIVATE_REPLAY_METADATA: PrivateReplayMetadata = {
  title: "",
  note: "",
  opening_ply: null,
};

function isPlainRecord(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return false;
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function hasExactKeys(value: Record<string, unknown>, expected: readonly string[]): boolean {
  const actual = Object.keys(value).sort();
  const wanted = [...expected].sort();
  return actual.length === wanted.length && actual.every((key, index) => key === wanted[index]);
}

function hasLoneSurrogate(value: string): boolean {
  for (let index = 0; index < value.length; index += 1) {
    const code = value.charCodeAt(index);
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = value.charCodeAt(index + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return true;
      index += 1;
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      return true;
    }
  }
  return false;
}

function assertPlainText(value: string, label: string, allowLineFeed: boolean): void {
  if (hasLoneSurrogate(value)) throw new Error(`${label} cannot contain lone Unicode surrogates.`);
  for (const character of value) {
    const code = character.codePointAt(0)!;
    if ((code <= 0x1f && !(allowLineFeed && code === 0x0a))
      || (code >= 0x7f && code <= 0x9f)
      || code === 0x2028
      || code === 0x2029) {
      throw new Error(`${label} contains a control character that cannot be saved.`);
    }
  }
}

export function privateMetadataCodePointCount(value: string): number {
  return Array.from(value).length;
}

/**
 * Normalize and validate private replay details. When moveCount is provided,
 * opening_ply is checked against that replay's actual move count; otherwise it
 * is bounded to the maximum 15×15 board move count for undecoded cloud data.
 */
export function validatePrivateReplayMetadata(
  value: unknown,
  moveCount?: number,
): PrivateReplayMetadata {
  if (!isPlainRecord(value) || !hasExactKeys(value, ["title", "note", "opening_ply"])) {
    throw new Error("Replay details must contain exactly title, note and opening_ply.");
  }
  if (typeof value.title !== "string" || typeof value.note !== "string") {
    throw new Error("Replay title and note must be plain text.");
  }
  const title = value.title.trim();
  const note = value.note.replace(/\r\n?/g, "\n").trim();
  assertPlainText(title, "Replay title", false);
  assertPlainText(note, "Replay note", true);
  if (privateMetadataCodePointCount(title) > 80 || new TextEncoder().encode(title).byteLength > 320) {
    throw new Error("Replay title must be at most 80 Unicode characters and 320 UTF-8 bytes.");
  }
  if (privateMetadataCodePointCount(note) > 280 || new TextEncoder().encode(note).byteLength > 1_120) {
    throw new Error("Replay note must be at most 280 Unicode characters and 1,120 UTF-8 bytes.");
  }
  const maximumPly = moveCount === undefined ? 225 : moveCount;
  if (moveCount !== undefined && (!Number.isInteger(moveCount) || moveCount < 0 || moveCount > 225)) {
    throw new Error("Replay move count must be an integer from 0 to 225.");
  }
  const openingPly = value.opening_ply;
  if (openingPly !== null && (typeof openingPly !== "number" || !Number.isInteger(openingPly)
    || openingPly < 0 || openingPly > maximumPly)) {
    throw new Error(`Replay opening position must be null or an integer from 0 to ${maximumPly}.`);
  }
  return { title, note, opening_ply: openingPly as number | null };
}
