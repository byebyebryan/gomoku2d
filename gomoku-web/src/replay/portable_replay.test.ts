import { describe, expect, it } from "vitest";

import { createLocalSavedMatch, movesFromMoveCells } from "../match/saved_match";
import type { MatchPlayer } from "../game/types";

import {
  PORTABLE_REPLAY_MAX_DECODED_BYTES,
  PORTABLE_REPLAY_MAX_ENCODED_CHARS,
  decodePortableReplay,
  decodePortableReplayFragment,
  encodePortableReplay,
  encodePortableReplayFragment,
  portableReplayFingerprint,
} from "./portable_replay_codec";
import { portableReplayFromSavedMatch, validatePortableReplay } from "./portable_replay";
import { portableReplayToSavedMatch } from "./portable_replay_adapters";
import type { PortableReplayV1 } from "./portable_replay";

const WIN_MOVES = [110, 0, 111, 1, 112, 2, 113, 3, 114];
const RENJU_DOUBLE_FOUR_MOVES = [110, 0, 111, 14, 113, 210, 82, 224, 97, 7, 127, 217, 112];
const FINGERPRINT = "A".repeat(43);

const HUMAN: PortableReplayV1["black"] = { kind: "human", name: "Guest" };
const BOT: PortableReplayV1["white"] = {
  bot: {
    config: { mode: "preset", preset: "hard", version: 1 },
    configVersion: 1,
    engine: "search_bot",
    version: 1,
  },
  kind: "bot",
  name: "Hard Bot",
};

function winningReplay(ruleset: PortableReplayV1["ruleset"] = "freestyle"): PortableReplayV1 {
  return {
    black: HUMAN,
    moves: WIN_MOVES,
    ruleset,
    version: 1,
    white: BOT,
  };
}

function fullDrawMoves(): number[] {
  const black: number[] = [];
  const white: number[] = [];
  for (let row = 0; row < 15; row += 1) {
    for (let col = 0; col < 15; col += 1) {
      const blackCell = ((((row + 2 * col) & 3) >> 1) ^ (row & 1)) === 1;
      (blackCell ? black : white).push(row * 15 + col);
    }
  }

  const moves: number[] = [];
  for (let index = 0; index < black.length; index += 1) {
    moves.push(black[index]!);
    if (index < white.length) {
      moves.push(white[index]!);
    }
  }
  return moves;
}

function maxDrawReplay(): PortableReplayV1 {
  const maxName = "💠".repeat(80);
  return {
    black: { kind: "human", name: maxName, profileUid: "b".repeat(128) },
    moves: fullDrawMoves(),
    practice: { inheritedPly: 4, sourceFingerprint: FINGERPRINT },
    ruleset: "freestyle",
    version: 1,
    viewPly: 225,
    white: { kind: "human", name: maxName, profileUid: "w".repeat(128) },
  };
}

function encodeAsciiJson(value: unknown): string {
  return btoa(JSON.stringify(value)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

function wireFromPayload(payload: string): Record<string, unknown> {
  const bytes = atob(payload.replace(/-/g, "+").replace(/_/g, "/")
    + "=".repeat((4 - payload.length % 4) % 4));
  return JSON.parse(bytes) as Record<string, unknown>;
}

function savedMatchForWinningReplay(status: "black_won" | "white_won" = "black_won") {
  const moves = movesFromMoveCells(WIN_MOVES);
  const players: [MatchPlayer, MatchPlayer] = [
    { kind: "human", name: "Guest", stone: "black" },
    { kind: "bot", name: "Hard Bot", stone: "white" },
  ];
  const match = createLocalSavedMatch({
    botConfig: { mode: "preset", preset: "hard", version: 1 },
    id: "private-match-id",
    localProfileId: "private-local-profile-id",
    moves,
    players,
    savedAt: "2026-10-21T00:00:00.000Z",
    status,
    ruleset: "renju",
  });
  match.player_black.profile_uid = "public_uid-123";
  return match;
}

describe("portable replay v1 rules validation", () => {
  it.each(["freestyle", "renju"] as const)("validates and derives a finished %s result with Wasm", (ruleset) => {
    expect(validatePortableReplay(winningReplay(ruleset)).status).toBe("black_won");
  });

  it("rejects Black's double-four as a forbidden Renju move in a full replay", () => {
    expect(() => validatePortableReplay({
      ...winningReplay("renju"),
      moves: RENJU_DOUBLE_FOUR_MOVES,
    })).toThrow("illegal move at ply 13");
  });

  it("accepts a full-board draw at the maximum move count", () => {
    const validated = validatePortableReplay(maxDrawReplay());
    expect(validated.replay.moves).toHaveLength(225);
    expect(validated.status).toBe("draw");
  });

  it("rejects duplicate cells, moves after terminal result, and ongoing games", () => {
    expect(() => validatePortableReplay({ ...winningReplay(), moves: [...WIN_MOVES.slice(0, 8), 113] }))
      .toThrow("repeats cell");
    expect(() => validatePortableReplay({ ...winningReplay(), moves: [...WIN_MOVES, 224] }))
      .toThrow("illegal move at ply 10");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      moves: [112, 0, 110, 2, 114, 4, 108, 6, 116],
    })).toThrow("must end with a win or a full-board draw");
  });

  it("requires a practice prefix to be legal, ongoing, and before the final move", () => {
    const base = winningReplay();
    expect(validatePortableReplay({
      ...base,
      practice: { inheritedPly: 4, sourceFingerprint: FINGERPRINT },
    }).status).toBe("black_won");
    expect(validatePortableReplay({
      ...base,
      practice: { inheritedPly: 8, sourceFingerprint: FINGERPRINT },
    }).replay.practice).toEqual({ inheritedPly: 8, sourceFingerprint: FINGERPRINT });
    expect(() => validatePortableReplay({
      ...base,
      practice: { inheritedPly: 3, sourceFingerprint: FINGERPRINT },
    })).toThrow("at least 4");
    expect(() => validatePortableReplay({
      ...base,
      practice: { inheritedPly: 9, sourceFingerprint: FINGERPRINT },
    })).toThrow("before the final move");
  });
});

describe("portable replay v1 format and identity", () => {
  it("round trips fixed-order UTF-8 JSON through the route fragment", () => {
    const replay = winningReplay("renju");
    const hash = encodePortableReplayFragment(replay);
    expect(hash.startsWith("#g2r1=")).toBe(true);
    expect(decodePortableReplayFragment(hash)).toMatchObject({
      replay: { ruleset: "renju", moves: WIN_MOVES },
      status: "black_won",
    });
  });

  it("fits the maximum Unicode, UID, practice-origin, and 225-move fixture within the URL bounds", () => {
    const replay = maxDrawReplay();
    const encoded = encodePortableReplay(replay);
    expect(encoded.length).toBeLessThanOrEqual(PORTABLE_REPLAY_MAX_ENCODED_CHARS);
    expect(decodePortableReplay(encoded).replay).toEqual(replay);

    const decodedJsonLength = Uint8Array.from(atob(encoded.replace(/-/g, "+").replace(/_/g, "/")
      + "=".repeat((4 - encoded.length % 4) % 4)), (character) => character.charCodeAt(0)).length;
    expect(decodedJsonLength).toBeLessThanOrEqual(PORTABLE_REPLAY_MAX_DECODED_BYTES);
  });

  it("round trips a supported custom bot identity through the canonical wire format", () => {
    const replay: PortableReplayV1 = {
      ...winningReplay(),
      white: {
        ...BOT,
        bot: {
          ...BOT.bot,
          config: {
            mode: "custom",
            depth: 5,
            extraPass: "none",
            scoring: "pattern",
            version: 1,
            width: 16,
          },
        },
      },
    };

    expect(decodePortableReplay(encodePortableReplay(replay)).replay.white).toEqual(replay.white);
  });

  it("rejects unsupported wire versions and malformed UTF-8", () => {
    const payload = encodePortableReplay(winningReplay());
    const wire = wireFromPayload(payload);
    expect(() => decodePortableReplay(encodeAsciiJson({ ...wire, v: 2 })))
      .toThrow("Unsupported portable replay version: 2");
    expect(() => decodePortableReplay("_w")).toThrow("not valid UTF-8");
  });

  it.each([-1, WIN_MOVES.length + 1])("rejects viewing ply %i outside the game", (viewPly) => {
    const replay = winningReplay();
    expect(() => validatePortableReplay({ ...replay, viewPly }))
      .toThrow("between 0 and the move count");

    const wire = wireFromPayload(encodePortableReplay(replay));
    expect(() => decodePortableReplay(encodeAsciiJson({ ...wire, at: viewPly })))
      .toThrow("between 0 and the move count");
  });

  it("keeps fingerprint stable across view plies and changes it for attribution or origin", async () => {
    const final = validatePortableReplay(winningReplay());
    const opening = validatePortableReplay({ ...winningReplay(), viewPly: 0 });
    const renamed = validatePortableReplay({
      ...winningReplay(),
      black: { kind: "human", name: "Another player" },
    });
    const practice = validatePortableReplay({
      ...winningReplay(),
      practice: { inheritedPly: 4, sourceFingerprint: FINGERPRINT },
    });

    expect(await portableReplayFingerprint(final)).toBe(await portableReplayFingerprint(opening));
    expect(await portableReplayFingerprint(final)).not.toBe(await portableReplayFingerprint(renamed));
    expect(await portableReplayFingerprint(final)).not.toBe(await portableReplayFingerprint(practice));
  });

  it("rejects unsupported fields, bot versions/configs, invalid names, and malformed or oversized payloads", () => {
    expect(() => validatePortableReplay({ ...winningReplay(), localProfileId: "private" }))
      .toThrow("unsupported field localProfileId");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      black: { kind: "human", name: "Guest", profileUid: "uid.with.dot" },
    })).toThrow("profileUid");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      black: { kind: "human", name: "💠".repeat(81) },
    })).toThrow("Unicode code points");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      black: { kind: "human", name: "Bad\nName" },
    })).toThrow("control characters");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      practice: { inheritedPly: 4, sourceFingerprint: `${"A".repeat(42)}B` },
    })).toThrow("SHA-256 base64url");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      white: { ...BOT, bot: { ...BOT.bot, engine: "future_engine" } },
    })).toThrow("unsupported engine or version");
    expect(() => validatePortableReplay({
      ...winningReplay(),
      white: {
        ...BOT,
        bot: {
          ...BOT.bot,
          config: {
            mode: "custom",
            depth: 5,
            extraPass: "none",
            scoring: "pattern",
            version: 1,
            width: "full",
          },
        },
      },
    })).toThrow("depth and width combination");
    expect(() => decodePortableReplay("*")).toThrow("base64url");
    expect(() => decodePortableReplay("AB")).toThrow("canonical unpadded base64url");
    expect(() => decodePortableReplay("ew")).toThrow("not valid JSON");
    const encoded = encodePortableReplay(winningReplay());
    const wire = wireFromPayload(encoded);
    expect(() => decodePortableReplay(encodeAsciiJson({ ...wire, providerUid: "private-provider-id" })))
      .toThrow("unsupported field providerUid");
    expect(() => decodePortableReplay("A".repeat(PORTABLE_REPLAY_MAX_ENCODED_CHARS + 1)))
      .toThrow("encoded characters");
  });
});

describe("portable and saved-match adapters", () => {
  it("exports only portable fields and rejects a saved status that disagrees with Wasm", () => {
    const saved = savedMatchForWinningReplay();
    const portable = portableReplayFromSavedMatch(saved, 4);
    expect(portable.status).toBe("black_won");
    expect(portable.replay.viewPly).toBe(4);
    expect(portable.replay.black).toEqual({ kind: "human", name: "Guest", profileUid: "public_uid-123" });
    const payload = JSON.stringify(portable.replay);
    expect(payload).not.toContain("private-match-id");
    expect(payload).not.toContain("private-local-profile-id");
    expect(payload).not.toContain("saved_at");
    expect(payload).not.toContain("client_uploaded");

    expect(() => portableReplayFromSavedMatch(savedMatchForWinningReplay("white_won")))
      .toThrow("does not match the core result");
  });

  it("projects a portable replay for existing viewer/analyzer APIs as explicitly untrusted", async () => {
    const candidate = {
      ...winningReplay(),
      practice: { inheritedPly: 4, sourceFingerprint: FINGERPRINT },
    };
    const { fingerprint, match, replay } = await portableReplayToSavedMatch(
      candidate,
      "2026-10-21T00:00:00.000Z",
    );

    expect(fingerprint).toMatch(/^[A-Za-z0-9_-]{43}$/);
    expect(match.id).toBe(`portable-${fingerprint}`);
    expect(match.source).toBe("cloud_saved");
    expect(match.trust).toBe("client_uploaded");
    expect(match.status).toBe("black_won");
    expect(match.undo_floor).toBe(4);
    expect(match.player_black.local_profile_id).toBeNull();
    expect(match.player_white.local_profile_id).toBeNull();
    expect(replay.practice).toEqual(candidate.practice);
    expect(JSON.stringify(match)).not.toContain(FINGERPRINT);
  });
});
