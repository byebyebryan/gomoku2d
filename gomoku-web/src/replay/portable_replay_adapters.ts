import {
  BOT_CONFIG_VERSION,
  BOT_ENGINE,
  BOT_ID,
  BOT_VERSION,
  SAVED_MATCH_BOARD_SIZE,
  SAVED_MATCH_SCHEMA_VERSION,
  type SavedMatchPlayer,
  type SavedMatchV2,
} from "../match/saved_match";
import { botLabel, labSpecForBot } from "../core/bot_config";
import { portableReplayFingerprint } from "./portable_replay_codec";
import {
  validatePortableReplay,
  type PortableReplayParticipantV1,
  type PortableReplayV1,
} from "./portable_replay";

export interface PortableReplaySavedMatchProjection {
  fingerprint: string;
  match: SavedMatchV2;
  replay: PortableReplayV1;
}

function savedPlayer(player: PortableReplayParticipantV1): SavedMatchPlayer {
  if (player.kind === "human") {
    return {
      bot: null,
      display_name: player.name,
      kind: "human",
      local_profile_id: null,
      profile_uid: player.profileUid ?? null,
    };
  }

  const config = player.bot.config;
  return {
    bot: {
      config,
      config_version: BOT_CONFIG_VERSION,
      engine: BOT_ENGINE,
      id: BOT_ID,
      label: botLabel(config),
      lab_spec: labSpecForBot(config),
      version: BOT_VERSION,
    },
    display_name: player.name,
    kind: "bot",
    local_profile_id: null,
    profile_uid: null,
  };
}

/**
 * Creates an ephemeral SavedMatchV2 projection for the existing replay frame
 * and analyzer. It is marked client-uploaded and must not be written into
 * legacy played history; the portable record remains the source of provenance.
 */
export async function portableReplayToSavedMatch(
  value: unknown,
  viewedAt = new Date().toISOString(),
): Promise<PortableReplaySavedMatchProjection> {
  if (!Number.isFinite(Date.parse(viewedAt))) {
    throw new Error("Portable replay projection requires a valid viewedAt timestamp.");
  }

  const validated = validatePortableReplay(value);
  const fingerprint = await portableReplayFingerprint(validated);
  const humans = [validated.replay.black, validated.replay.white]
    .filter((player) => player.kind === "human").length;
  const bots = 2 - humans;
  const match: SavedMatchV2 = {
    board_size: SAVED_MATCH_BOARD_SIZE,
    id: `portable-${fingerprint}`,
    match_kind: humans === 1 && bots === 1 ? "local_vs_bot" : "local_pvp",
    move_cells: [...validated.replay.moves],
    move_count: validated.replay.moves.length,
    player_black: savedPlayer(validated.replay.black),
    player_white: savedPlayer(validated.replay.white),
    saved_at: viewedAt,
    schema_version: SAVED_MATCH_SCHEMA_VERSION,
    source: "cloud_saved",
    status: validated.status,
    trust: "client_uploaded",
    undo_floor: validated.replay.practice?.inheritedPly ?? 0,
    ruleset: validated.replay.ruleset,
  };
  return { fingerprint, match, replay: validated.replay };
}
