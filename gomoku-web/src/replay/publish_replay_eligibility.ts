import { matchUserSide, type SavedMatchSide, type SavedMatchV2 } from "../match/saved_match";

export function publishReplaySide(
  match: SavedMatchV2,
  uid: string | null,
  localProfileId: string | null,
  ownerPractice = false,
): SavedMatchSide | null {
  if (!uid) return null;
  const side = matchUserSide(match, { profileUid: uid, localProfileId });
  if (side) return match[`player_${side}`].kind === "human" ? side : null;

  // Older continuations are projected from the active owner's private
  // practice records. They can lack the local ID of the human who played.
  if (!ownerPractice || !match.replay_origin || !match.id.startsWith("portable-")) return null;
  const black = match.player_black.kind === "human";
  const white = match.player_white.kind === "human";
  return black === white ? null : black ? "black" : "white";
}
