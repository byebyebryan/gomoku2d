import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { labSpecForBot } from "../core/bot_config";
import { buildLocalReplayFrame, canResumeReplay } from "./local_replay";
import { portableReplayToSavedMatch } from "./portable_replay_adapters";
import { decodePortableReplayFragment, portableReplayFingerprint } from "./portable_replay_codec";
import { validatePortableReplay } from "./portable_replay";
import { exampleReplayPath, REPLAY_EXAMPLES } from "./replay_examples";

const report = JSON.parse(readFileSync(resolve(process.cwd(), "../reports/lab/analysis-report.json"), "utf8")) as {
  sections: Array<{ entries: Array<{ path: string; match_report: { black: string; white: string; move_cells: number[]; result: string } }> }>;
};

describe("static replay example corpus", () => {
  it.each(REPLAY_EXAMPLES)("keeps $id legal, portable, truthful and independently usable", async (example) => {
    const source = report.sections.flatMap((section) => section.entries).find((entry) => entry.path === example.source);
    expect(source).toBeDefined();
    const validated = validatePortableReplay(example.replay);
    expect(validated.status).toBe(source!.match_report.result);
    expect(example.replay.moves).toEqual(source!.match_report.move_cells);
    for (const [side, player] of [["black", example.replay.black], ["white", example.replay.white]] as const) {
      expect(player.kind).toBe("bot");
      expect(player).not.toHaveProperty("profileUid");
      if (player.kind === "bot") expect(labSpecForBot(player.bot.config)).toBe(source!.match_report[side]);
    }
    const path = new URL(exampleReplayPath(example), "https://example.test");
    expect(path.pathname).toBe("/replay/shared/");
    expect(path.search).toBe("");
    expect(decodePortableReplayFragment(path.hash).replay).toEqual(example.replay);
    expect(example.replay).not.toHaveProperty("practice");
    const projection = await portableReplayToSavedMatch(example.replay);
    const ply = example.replay.viewPly ?? example.replay.moves.length;
    const frame = buildLocalReplayFrame(projection.match, ply);
    expect(canResumeReplay(frame, projection.match.undo_floor)).toBe(example.replay.viewPly !== undefined);
    expect(await portableReplayFingerprint(validatePortableReplay({ ...example.replay, viewPly: ply - 1 })))
      .toBe(projection.fingerprint);
  });

  it("has a bounded pair of unique, stable content IDs", () => {
    expect(REPLAY_EXAMPLES.map((example) => example.id)).toEqual(["finished-game", "another-line"]);
    expect(new Set(REPLAY_EXAMPLES.map((example) => example.source)).size).toBe(2);
  });
});
