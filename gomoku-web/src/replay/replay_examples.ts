import type { BotPresetId } from "../core/bot_config";
import type { PortableReplayParticipantV1, PortableReplayV1 } from "./portable_replay";
import { encodePortableReplayFragment } from "./portable_replay_codec";

export interface ReplayExample {
  id: string;
  title: string;
  description: string;
  source: string;
  replay: PortableReplayV1;
}

function exampleBot(preset: BotPresetId): PortableReplayParticipantV1 {
  return {
    kind: "bot",
    name: `${preset[0]!.toUpperCase()}${preset.slice(1)} Bot`,
    bot: { config: { mode: "preset", preset, version: 1 }, configVersion: 1, engine: "search_bot", version: 1 },
  };
}

/** Small static corpus extracted from the published June 2026 lab report.
 * These are bot-generated games, never played-user history or public profiles.
 * See docs/reference/app/replay_examples.md for provenance and validation.
 */
export const REPLAY_EXAMPLE_CORPUS_VERSION = 1;
export const REPLAY_EXAMPLES: readonly ReplayExample[] = [
  {
    id: "finished-game",
    title: "Review a finished game",
    description: "Step back through the ending and inspect the analysis.",
    source: "match_0095__search-d1__vs__search-d3_pattern-eval",
    replay: {
      version: 1, ruleset: "freestyle", black: exampleBot("easy"), white: exampleBot("normal"),
      moves: [112, 126, 96, 142, 80, 128, 64, 48, 65, 114, 100, 127, 125, 110, 94, 174, 158, 129, 130, 144, 159, 141, 145, 99, 84, 113, 85, 155],
    },
  },
  {
    id: "another-line",
    title: "Try another line",
    description: "Try move 26 against your selected bot.",
    source: "match_1129__search-d3_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval_corridor-proof-c16-d8-w4",
    replay: {
      version: 1, ruleset: "freestyle", black: exampleBot("normal"), white: exampleBot("hard"), viewPly: 26,
      moves: [112, 128, 111, 82, 110, 113, 98, 126, 70, 127, 108, 109, 125, 99, 85, 129, 130, 84, 114, 97, 146, 162, 100, 115, 81, 143, 101, 131, 83, 145, 161, 173, 158, 159, 117, 187],
    },
  },
];

export function exampleReplayPath(example: ReplayExample): string {
  return `/replay/shared/${encodePortableReplayFragment(example.replay)}`;
}
