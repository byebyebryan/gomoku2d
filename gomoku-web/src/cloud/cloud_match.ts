import { serverTimestamp, type FieldValue } from "firebase/firestore";

import {
  PRACTICE_BOT_CONFIG_VERSION,
  PRACTICE_BOT_DEPTH,
  PRACTICE_BOT_ENGINE,
  PRACTICE_BOT_ID,
  SAVED_MATCH_SCHEMA_VERSION,
  decodeMoveCell,
  type SavedMatchBotIdentity,
  type SavedMatchPlayer,
  type SavedMatchV1,
} from "../match/saved_match";
import type { GuestProfileIdentity, GuestSavedMatch } from "../profile/guest_profile_store";

import type { CloudAuthUser } from "./auth_store";

export const CLOUD_MATCH_SCHEMA_VERSION = SAVED_MATCH_SCHEMA_VERSION;
export const CLOUD_MATCH_SOURCE_GUEST_IMPORT = "guest_import";
export const CLOUD_MATCH_SOURCE_CLOUD_SAVED = "cloud_saved";
export const CLOUD_MATCH_TRUST_CLIENT_UPLOADED = "client_uploaded";
export {
  PRACTICE_BOT_CONFIG_VERSION,
  PRACTICE_BOT_DEPTH,
  PRACTICE_BOT_ENGINE,
  PRACTICE_BOT_ID,
};

export type CloudMatchBotIdentity = SavedMatchBotIdentity;
export type CloudMatchPlayerDocument = SavedMatchPlayer;

/** Document shape for matches promoted from local guest history. */
export interface CloudGuestImportDocument
  extends Omit<SavedMatchV1, "player_black" | "player_white" | "source" | "trust"> {
  imported_at: FieldValue;
  local_match_id: string;
  local_origin_id: string;
  player_black: CloudMatchPlayerDocument;
  player_white: CloudMatchPlayerDocument;
  source: typeof CLOUD_MATCH_SOURCE_GUEST_IMPORT;
  trust: typeof CLOUD_MATCH_TRUST_CLIENT_UPLOADED;
}

/** Document shape for matches saved directly to cloud while signed in. */
export interface CloudDirectSavedDocument
  extends Omit<SavedMatchV1, "player_black" | "player_white" | "source" | "trust"> {
  created_at: FieldValue;
  player_black: CloudMatchPlayerDocument;
  player_white: CloudMatchPlayerDocument;
  source: typeof CLOUD_MATCH_SOURCE_CLOUD_SAVED;
  trust: typeof CLOUD_MATCH_TRUST_CLIENT_UPLOADED;
}

function assertFinishedMatch(match: Pick<SavedMatchV1, "status">): void {
  if (match.status !== "black_won" && match.status !== "white_won" && match.status !== "draw") {
    throw new Error("Cloud match promotion only supports finished matches.");
  }
}

function assertValidMovePayload(match: Pick<SavedMatchV1, "move_count" | "move_cells">): void {
  if (match.move_count !== match.move_cells.length) {
    throw new Error("Cloud match promotion requires move_count to match move_cells.");
  }

  for (const cell of match.move_cells) {
    decodeMoveCell(cell);
  }
}

function assertGuestLocalMatch(match: Pick<SavedMatchV1, "source" | "trust">): void {
  if (match.source !== "local_history" || match.trust !== "local_only") {
    throw new Error("Cloud match promotion only supports local history records.");
  }
}

function assertLocalVsBotPlayers(match: Pick<SavedMatchV1, "match_kind" | "player_black" | "player_white">): void {
  const humanCount = [match.player_black, match.player_white].filter((player) => player.kind === "human").length;
  const botCount = [match.player_black, match.player_white].filter((player) => player.kind === "bot").length;

  if (match.match_kind !== "local_vs_bot" || humanCount !== 1 || botCount !== 1) {
    throw new Error("Cloud match promotion requires one human player and one bot player.");
  }
}

function guestImportPlayerDocument(
  player: SavedMatchPlayer,
  user: Pick<CloudAuthUser, "uid">,
  guestProfile: Pick<GuestProfileIdentity, "displayName" | "id">,
): CloudMatchPlayerDocument {
  if (player.kind === "human") {
    return {
      ...player,
      bot: null,
      display_name: guestProfile.displayName,
      local_profile_id: player.local_profile_id ?? guestProfile.id,
      profile_uid: user.uid,
    };
  }

  return {
    ...player,
    bot: player.bot,
    local_profile_id: null,
    profile_uid: null,
  };
}

function cloudDirectSavedPlayerDocument(
  player: SavedMatchPlayer,
  user: Pick<CloudAuthUser, "uid">,
): CloudMatchPlayerDocument {
  if (player.kind === "human") {
    return {
      ...player,
      bot: null,
      // local_profile_id is null: cross-device identity uses profile_uid only
      local_profile_id: null,
      profile_uid: user.uid,
    };
  }

  return {
    ...player,
    bot: player.bot,
    local_profile_id: null,
    profile_uid: null,
  };
}

export function cloudMatchIdForGuestMatch(match: Pick<GuestSavedMatch, "id">): string {
  return `local-${encodeURIComponent(match.id)}`;
}

export function localOriginIdForGuestMatch(
  guestProfile: Pick<GuestProfileIdentity, "id">,
  match: Pick<GuestSavedMatch, "id">,
): string {
  return `guest:${guestProfile.id}:${match.id}`;
}

export function cloudSavedMatchFromGuestMatch(
  user: Pick<CloudAuthUser, "uid">,
  guestProfile: Pick<GuestProfileIdentity, "displayName" | "id">,
  match: GuestSavedMatch,
): CloudGuestImportDocument {
  assertGuestLocalMatch(match);
  assertFinishedMatch(match);
  assertValidMovePayload(match);
  assertLocalVsBotPlayers(match);

  const matchId = cloudMatchIdForGuestMatch(match);

  return {
    ...match,
    id: matchId,
    imported_at: serverTimestamp(),
    local_match_id: match.id,
    local_origin_id: localOriginIdForGuestMatch(guestProfile, match),
    player_black: guestImportPlayerDocument(match.player_black, user, guestProfile),
    player_white: guestImportPlayerDocument(match.player_white, user, guestProfile),
    source: CLOUD_MATCH_SOURCE_GUEST_IMPORT,
    trust: CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
  };
}

export function cloudDirectSavedMatchId(match: Pick<SavedMatchV1, "id">): string {
  return match.id;
}

export function createCloudDirectSavedDocument(
  user: Pick<CloudAuthUser, "uid">,
  match: SavedMatchV1,
): CloudDirectSavedDocument {
  assertFinishedMatch(match);
  assertValidMovePayload(match);
  assertLocalVsBotPlayers(match);

  return {
    ...match,
    created_at: serverTimestamp(),
    player_black: cloudDirectSavedPlayerDocument(match.player_black, user),
    player_white: cloudDirectSavedPlayerDocument(match.player_white, user),
    source: CLOUD_MATCH_SOURCE_CLOUD_SAVED,
    trust: CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
  };
}
