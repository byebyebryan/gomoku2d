import {
  BOT_CONFIG_VERSION,
  BOT_ENGINE,
  BOT_ID,
  SAVED_MATCH_SCHEMA_VERSION,
  decodeMoveCell,
  type SavedMatchPlayer,
  type SavedMatchV2,
} from "../match/saved_match";

import type { CloudAuthUser } from "./auth_store";

export const CLOUD_MATCH_SCHEMA_VERSION = SAVED_MATCH_SCHEMA_VERSION;
export const CLOUD_MATCH_SOURCE_CLOUD_SAVED = "cloud_saved";
export const CLOUD_MATCH_TRUST_CLIENT_UPLOADED = "client_uploaded";
export {
  BOT_CONFIG_VERSION,
  BOT_ENGINE,
  BOT_ID,
};

export type CloudSavedMatch = SavedMatchV2 & {
  source: typeof CLOUD_MATCH_SOURCE_CLOUD_SAVED;
  trust: typeof CLOUD_MATCH_TRUST_CLIENT_UPLOADED;
};

function assertFinishedMatch(match: Pick<SavedMatchV2, "status">): void {
  if (match.status !== "black_won" && match.status !== "white_won" && match.status !== "draw") {
    throw new Error("Cloud history sync only supports finished matches.");
  }
}

function assertValidMovePayload(match: Pick<SavedMatchV2, "move_count" | "move_cells">): void {
  if (match.move_count !== match.move_cells.length) {
    throw new Error("Cloud history sync requires move_count to match move_cells.");
  }

  for (const cell of match.move_cells) {
    decodeMoveCell(cell);
  }
}

function assertValidSavedAt(match: Pick<SavedMatchV2, "saved_at">): void {
  if (!Number.isFinite(Date.parse(match.saved_at))) {
    throw new Error("Cloud history sync requires a valid saved_at timestamp.");
  }
}

function assertLocalHistoryMatch(match: Pick<SavedMatchV2, "source" | "trust">): void {
  if (match.source !== "local_history" || match.trust !== "local_only") {
    throw new Error("Cloud history sync only supports local history records.");
  }
}

function assertLocalVsBotPlayers(match: Pick<SavedMatchV2, "match_kind" | "player_black" | "player_white">): void {
  const humanCount = [match.player_black, match.player_white].filter((player) => player.kind === "human").length;
  const botCount = [match.player_black, match.player_white].filter((player) => player.kind === "bot").length;

  if (match.match_kind !== "local_vs_bot" || humanCount !== 1 || botCount !== 1) {
    throw new Error("Cloud history sync requires one human player and one bot player.");
  }
}

function cloudSavedPlayer(
  player: SavedMatchPlayer,
  user: Pick<CloudAuthUser, "uid">,
): SavedMatchPlayer {
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

export function createCloudSavedMatch(
  user: Pick<CloudAuthUser, "uid">,
  match: SavedMatchV2,
): CloudSavedMatch {
  assertLocalHistoryMatch(match);
  assertFinishedMatch(match);
  assertValidMovePayload(match);
  assertValidSavedAt(match);
  assertLocalVsBotPlayers(match);

  return {
    ...match,
    player_black: cloudSavedPlayer(match.player_black, user),
    player_white: cloudSavedPlayer(match.player_white, user),
    source: CLOUD_MATCH_SOURCE_CLOUD_SAVED,
    trust: CLOUD_MATCH_TRUST_CLIENT_UPLOADED,
  };
}
