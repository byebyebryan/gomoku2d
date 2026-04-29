import { describe, expect, it } from "vitest";

import { createLocalSavedMatch } from "../match/saved_match";
import type { GuestSavedMatch } from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_MATCH_SCHEMA_VERSION,
  CLOUD_MATCH_SOURCE_CLOUD_SAVED,
  CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
  PRACTICE_BOT_CONFIG_VERSION,
  PRACTICE_BOT_DEPTH,
  PRACTICE_BOT_ENGINE,
  PRACTICE_BOT_ID,
  createCloudSavedMatch,
} from "./cloud_match";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const match: GuestSavedMatch = createLocalSavedMatch({
  id: "match-1",
  localProfileId: "guest-1",
  moves: [
    { col: 7, moveNumber: 1, player: 1, row: 7 },
    { col: 8, moveNumber: 2, player: 2, row: 7 },
  ],
  players: [
    { kind: "human", name: "ByeByeBryan", stone: "black" },
    { kind: "bot", name: "Practice Bot", stone: "white" },
  ],
  savedAt: "2026-04-27T01:02:03.000Z",
  status: "black_won",
  undoFloor: 99,
  variant: "renju",
});

describe("cloud match serialization", () => {
  it("keeps local match identifiers for embedded profile history", () => {
    expect(createCloudSavedMatch(user, match).id).toBe("match-1");
  });

  it("serializes a finished local match as a cloud-saved embedded record", () => {
    const document = createCloudSavedMatch(user, match);

    expect(document).toMatchObject({
      board_size: 15,
      id: "match-1",
      match_kind: "local_vs_bot",
      move_cells: [112, 113],
      move_count: 2,
      player_black: {
        bot: null,
        display_name: "ByeByeBryan",
        kind: "human",
        local_profile_id: null,
        profile_uid: "uid-1",
      },
      player_white: {
        bot: {
          config: {
            depth: PRACTICE_BOT_DEPTH,
            kind: "baseline",
          },
          config_version: PRACTICE_BOT_CONFIG_VERSION,
          engine: PRACTICE_BOT_ENGINE,
          id: PRACTICE_BOT_ID,
          version: 1,
        },
        display_name: "Practice Bot",
        kind: "bot",
        local_profile_id: null,
        profile_uid: null,
      },
      saved_at: "2026-04-27T01:02:03.000Z",
      schema_version: CLOUD_MATCH_SCHEMA_VERSION,
      source: CLOUD_MATCH_SOURCE_CLOUD_SAVED,
      status: "black_won",
      trust: CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
      undo_floor: 2,
      variant: "renju",
    });
  });

  it("sets local_profile_id to null on the human player for cross-device identity", () => {
    const document = createCloudSavedMatch(user, match);
    expect(document.player_black.local_profile_id).toBeNull();
  });

  it("rejects unfinished local matches", () => {
    expect(() =>
      createCloudSavedMatch(user, {
        ...match,
        status: "playing",
      } as unknown as GuestSavedMatch),
    ).toThrow("finished matches");
  });

  it("rejects moves outside the board", () => {
    expect(() =>
      createCloudSavedMatch(user, {
        ...match,
        move_cells: [225],
        move_count: 1,
      }),
    ).toThrow("outside the board");
  });

  it("rejects records that are not local-only history", () => {
    expect(() =>
      createCloudSavedMatch(user, {
        ...match,
        source: "cloud_saved",
        trust: "client_uploaded",
      } as unknown as GuestSavedMatch),
    ).toThrow("local history records");
  });

  it("rejects imports without one human and one bot player", () => {
    expect(() =>
      createCloudSavedMatch(user, {
        ...match,
        player_white: {
          ...match.player_white,
          bot: null,
          kind: "human",
          local_profile_id: "guest-2",
        },
      }),
    ).toThrow("one human player and one bot player");
  });

  it("rejects records without a valid saved_at timestamp", () => {
    expect(() =>
      createCloudSavedMatch(user, {
        ...match,
        saved_at: "not-a-date",
      }),
    ).toThrow("valid saved_at timestamp");
  });
});
