import { describe, expect, it } from "vitest";

import type { CloudAuthUser } from "./auth_store";
import {
  CLOUD_MATCH_SCHEMA_VERSION,
  CLOUD_MATCH_SOURCE_CLOUD_SAVED,
  CLOUD_MATCH_SOURCE_GUEST_IMPORT,
  CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
  PRACTICE_BOT_CONFIG_VERSION,
  PRACTICE_BOT_DEPTH,
  PRACTICE_BOT_ENGINE,
  PRACTICE_BOT_ID,
  cloudDirectSavedMatchId,
  cloudMatchIdForGuestMatch,
  cloudSavedMatchFromGuestMatch,
  createCloudDirectSavedDocument,
  localOriginIdForGuestMatch,
} from "./cloud_match";
import type { GuestProfileIdentity, GuestSavedMatch } from "../profile/guest_profile_store";
import { createLocalSavedMatch } from "../match/saved_match";

const user: CloudAuthUser = {
  avatarUrl: null,
  displayName: "Bryan",
  email: "bryan@example.com",
  providerIds: ["google.com"],
  uid: "uid-1",
};

const guestProfile: GuestProfileIdentity = {
  avatarUrl: null,
  createdAt: "2026-04-27T00:00:00.000Z",
  displayName: "ByeByeBryan",
  id: "guest-1",
  kind: "guest",
  updatedAt: "2026-04-27T00:00:00.000Z",
  username: null,
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
  it("uses deterministic local import identifiers", () => {
    expect(cloudMatchIdForGuestMatch(match)).toBe("local-match-1");
    expect(localOriginIdForGuestMatch(guestProfile, match)).toBe("guest:guest-1:match-1");
  });

  it("serializes a finished guest match for private cloud import", () => {
    const document = cloudSavedMatchFromGuestMatch(user, guestProfile, match);

    expect(document).toMatchObject({
      board_size: 15,
      id: "local-match-1",
      local_match_id: "match-1",
      local_origin_id: "guest:guest-1:match-1",
      match_kind: "local_vs_bot",
      match_saved_at: expect.anything(),
      move_cells: [112, 113],
      move_count: 2,
      player_black: {
        bot: null,
        display_name: "ByeByeBryan",
        kind: "human",
        local_profile_id: "guest-1",
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
      source: CLOUD_MATCH_SOURCE_GUEST_IMPORT,
      status: "black_won",
      trust: CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
      undo_floor: 2,
      variant: "renju",
    });
    expect(document.match_saved_at.toDate().toISOString()).toBe(match.saved_at);
  });

  it("uses the current promoted guest display name for human player snapshots", () => {
    const defaultGuestMatch = createLocalSavedMatch({
      id: "match-2",
      localProfileId: "guest-1",
      moves: [{ col: 7, moveNumber: 1, player: 1, row: 7 }],
      players: [
        { kind: "human", name: "Guest", stone: "black" },
        { kind: "bot", name: "Practice Bot", stone: "white" },
      ],
      savedAt: "2026-04-27T01:02:03.000Z",
      status: "draw",
      variant: "freestyle",
    });

    const document = cloudSavedMatchFromGuestMatch(user, { ...guestProfile, displayName: "Bryan" }, defaultGuestMatch);

    expect(document.player_black).toMatchObject({
      display_name: "Bryan",
      kind: "human",
      profile_uid: "uid-1",
    });
  });

  it("rejects unfinished local matches", () => {
    expect(() =>
      cloudSavedMatchFromGuestMatch(user, guestProfile, {
        ...match,
        status: "playing",
      } as unknown as GuestSavedMatch),
    ).toThrow("finished matches");
  });

  it("rejects moves outside the board", () => {
    expect(() =>
      cloudSavedMatchFromGuestMatch(user, guestProfile, {
        ...match,
        move_cells: [225],
        move_count: 1,
      }),
    ).toThrow("outside the board");
  });

  it("rejects records that are not local-only history", () => {
    expect(() =>
      cloudSavedMatchFromGuestMatch(user, guestProfile, {
        ...match,
        source: "cloud_saved",
        trust: "client_uploaded",
      } as unknown as GuestSavedMatch),
    ).toThrow("local history records");
  });

  it("rejects imports without one human and one bot player", () => {
    expect(() =>
      cloudSavedMatchFromGuestMatch(user, guestProfile, {
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

  it("rejects imports without a valid saved_at timestamp", () => {
    expect(() =>
      cloudSavedMatchFromGuestMatch(user, guestProfile, {
        ...match,
        saved_at: "not-a-date",
      }),
    ).toThrow("valid saved_at timestamp");
  });
});

describe("cloud direct-saved match", () => {
  it("uses the local match id as the cloud doc id", () => {
    expect(cloudDirectSavedMatchId(match)).toBe("match-1");
  });

  it("serializes a finished signed-in match for direct cloud save", () => {
    const document = createCloudDirectSavedDocument(user, match);

    expect(document).toMatchObject({
      board_size: 15,
      id: "match-1",
      match_kind: "local_vs_bot",
      match_saved_at: expect.anything(),
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
          config: { depth: PRACTICE_BOT_DEPTH, kind: "baseline" },
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
      variant: "renju",
    });
    expect(document.match_saved_at.toDate().toISOString()).toBe(match.saved_at);
  });

  it("sets local_profile_id to null on the human player for cross-device identity", () => {
    const document = createCloudDirectSavedDocument(user, match);
    expect(document.player_black.local_profile_id).toBeNull();
  });

  it("rejects unfinished matches", () => {
    expect(() =>
      createCloudDirectSavedDocument(user, { ...match, status: "playing" } as unknown as typeof match),
    ).toThrow("finished matches");
  });

  it("rejects records that are already cloud sourced", () => {
    expect(() =>
      createCloudDirectSavedDocument(user, {
        ...match,
        source: "guest_import",
        trust: "client_uploaded",
      }),
    ).toThrow("local history records");
  });

  it("rejects direct saves without a valid saved_at timestamp", () => {
    expect(() =>
      createCloudDirectSavedDocument(user, {
        ...match,
        saved_at: "not-a-date",
      }),
    ).toThrow("valid saved_at timestamp");
  });
});
