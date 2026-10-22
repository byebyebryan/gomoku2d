import {
  validatePortableReplay,
  type PortableReplayParticipantV1,
  type PortableReplayV1,
  type ValidatedPortableReplayV1,
} from "./portable_replay";

export const PORTABLE_REPLAY_MAX_ENCODED_CHARS = 8192;
export const PORTABLE_REPLAY_MAX_DECODED_BYTES = 6144;
export const PORTABLE_REPLAY_HASH_PREFIX = "#g2r1=";

type PortableWireRecord = Record<string, unknown>;

function hasOwn(record: Record<string, unknown>, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(record, key);
}

function encodeBase64Url(bytes: Uint8Array): string {
  let binary = "";
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    const chunk = bytes.subarray(offset, Math.min(offset + 0x8000, bytes.length));
    binary += String.fromCharCode(...chunk);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function decodeBase64Url(value: string, label: string): Uint8Array {
  if (!/^[A-Za-z0-9_-]*$/.test(value) || value.length % 4 === 1) {
    throw new Error(`${label} must be unpadded base64url.`);
  }
  const padded = value.replace(/-/g, "+").replace(/_/g, "/")
    + "=".repeat((4 - value.length % 4) % 4);
  let binary: string;
  try {
    binary = atob(padded);
  } catch {
    throw new Error(`${label} is malformed base64url.`);
  }
  const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
  if (encodeBase64Url(bytes) !== value) {
    throw new Error(`${label} must use canonical unpadded base64url.`);
  }
  return bytes;
}

function wireParticipant(participant: PortableReplayParticipantV1): PortableWireRecord {
  if (participant.kind === "human") {
    return {
      k: "h",
      n: participant.name,
      ...(participant.profileUid === undefined ? {} : { u: participant.profileUid }),
    };
  }
  return {
    k: "b",
    n: participant.name,
    e: participant.bot.engine,
    v: participant.bot.version,
    cv: participant.bot.configVersion,
    c: participant.bot.config,
  };
}

function wireRecord(replay: PortableReplayV1, includeViewPly: boolean): PortableWireRecord {
  return {
    v: replay.version,
    r: replay.ruleset,
    m: encodeBase64Url(Uint8Array.from(replay.moves)),
    b: wireParticipant(replay.black),
    w: wireParticipant(replay.white),
    ...(replay.practice === undefined
      ? {}
      : { o: { f: replay.practice.sourceFingerprint, p: replay.practice.inheritedPly } }),
    ...(!includeViewPly || replay.viewPly === undefined ? {} : { at: replay.viewPly }),
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function assertKeys(
  record: Record<string, unknown>,
  required: readonly string[],
  optional: readonly string[],
  label: string,
): void {
  const allowed = new Set([...required, ...optional]);
  for (const key of required) {
    if (!hasOwn(record, key)) {
      throw new Error(`${label} is missing ${key}.`);
    }
  }
  for (const key of Object.keys(record)) {
    if (!allowed.has(key)) {
      throw new Error(`${label} contains unsupported field ${key}.`);
    }
  }
}

function wireToParticipant(value: unknown, label: string): unknown {
  if (!isRecord(value)) {
    throw new Error(`${label} must be an object.`);
  }
  if (value.k === "h") {
    assertKeys(value, ["k", "n"], ["u"], label);
    return {
      kind: "human",
      name: value.n,
      ...(hasOwn(value, "u") ? { profileUid: value.u } : {}),
    };
  }
  if (value.k === "b") {
    assertKeys(value, ["k", "n", "e", "v", "cv", "c"], [], label);
    return {
      bot: {
        config: value.c,
        configVersion: value.cv,
        engine: value.e,
        version: value.v,
      },
      kind: "bot",
      name: value.n,
    };
  }
  throw new Error(`${label}.k must be h or b.`);
}

function wireToReplay(value: unknown): PortableReplayV1 {
  if (!isRecord(value)) {
    throw new Error("Portable replay JSON must contain an object.");
  }
  assertKeys(value, ["v", "r", "m", "b", "w"], ["o", "at"], "Portable replay wire record");
  if (typeof value.m !== "string") {
    throw new Error("Portable replay wire moves must be base64url text.");
  }
  const moves = Array.from(decodeBase64Url(value.m, "Portable replay moves"));
  let practice: PortableReplayV1["practice"];
  if (hasOwn(value, "o")) {
    if (!isRecord(value.o)) {
      throw new Error("Portable replay practice origin must be an object.");
    }
    assertKeys(value.o, ["f", "p"], [], "Portable replay practice origin");
    practice = {
      inheritedPly: value.o.p as number,
      sourceFingerprint: value.o.f as string,
    };
  }

  return {
    black: wireToParticipant(value.b, "Portable replay black player") as PortableReplayParticipantV1,
    moves,
    ...(practice === undefined ? {} : { practice }),
    ruleset: value.r as PortableReplayV1["ruleset"],
    version: value.v as PortableReplayV1["version"],
    ...(hasOwn(value, "at") ? { viewPly: value.at as number } : {}),
    white: wireToParticipant(value.w, "Portable replay white player") as PortableReplayParticipantV1,
  };
}

function encodeCanonicalJson(replay: PortableReplayV1, includeViewPly: boolean): string {
  return JSON.stringify(wireRecord(replay, includeViewPly));
}

function assertEncodedLength(value: string): void {
  if (value.length > PORTABLE_REPLAY_MAX_ENCODED_CHARS) {
    throw new Error(`Portable replay exceeds ${PORTABLE_REPLAY_MAX_ENCODED_CHARS} encoded characters.`);
  }
}

/** Encodes a legal finished replay as unpadded base64url over fixed-order UTF-8 JSON. */
export function encodePortableReplay(value: unknown): string {
  const { replay } = validatePortableReplay(value);
  const json = encodeCanonicalJson(replay, true);
  const bytes = new TextEncoder().encode(json);
  if (bytes.byteLength > PORTABLE_REPLAY_MAX_DECODED_BYTES) {
    throw new Error(`Portable replay exceeds ${PORTABLE_REPLAY_MAX_DECODED_BYTES} decoded bytes.`);
  }
  const encoded = encodeBase64Url(bytes);
  assertEncodedLength(encoded);
  return encoded;
}

/** Decodes strict canonical JSON and runs the same Wasm legality validator as every adapter. */
export function decodePortableReplay(encoded: string): ValidatedPortableReplayV1 {
  if (typeof encoded !== "string" || encoded.length === 0) {
    throw new Error("Portable replay payload is empty.");
  }
  assertEncodedLength(encoded);
  const bytes = decodeBase64Url(encoded, "Portable replay payload");
  if (bytes.byteLength > PORTABLE_REPLAY_MAX_DECODED_BYTES) {
    throw new Error(`Portable replay exceeds ${PORTABLE_REPLAY_MAX_DECODED_BYTES} decoded bytes.`);
  }

  let json: string;
  try {
    json = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    throw new Error("Portable replay payload is not valid UTF-8.");
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch {
    throw new Error("Portable replay payload is not valid JSON.");
  }

  const replay = wireToReplay(parsed);
  const validated = validatePortableReplay(replay);
  const canonicalJson = encodeCanonicalJson(validated.replay, true);
  if (canonicalJson !== json) {
    throw new Error("Portable replay JSON must use the canonical v1 field order and encoding.");
  }
  return validated;
}

/** Fingerprints canonical portable contents, including attribution and origin but excluding viewPly. */
export async function portableReplayFingerprint(validated: ValidatedPortableReplayV1): Promise<string> {
  const { replay } = validated;
  const canonical = `gomoku2d-portable-replay-v1\0${encodeCanonicalJson(replay, false)}`;
  const digest = await globalThis.crypto.subtle.digest("SHA-256", new TextEncoder().encode(canonical));
  return encodeBase64Url(new Uint8Array(digest));
}

export function encodePortableReplayFragment(value: unknown): string {
  return `${PORTABLE_REPLAY_HASH_PREFIX}${encodePortableReplay(value)}`;
}

export function decodePortableReplayFragment(hash: string): ValidatedPortableReplayV1 {
  if (typeof hash !== "string" || !hash.startsWith(PORTABLE_REPLAY_HASH_PREFIX)) {
    throw new Error("Portable replay fragment must begin with #g2r1=.");
  }
  return decodePortableReplay(hash.slice(PORTABLE_REPLAY_HASH_PREFIX.length));
}
