import { readFileSync } from "node:fs";
import { afterEach, expect, it, vi } from "vitest";

import { loadPublishedBotReport } from "./bot_report";

function artifact() { return JSON.parse(readFileSync("../reports/lab/bot-report.json", "utf8")); }
function serve(value: unknown) {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify(value))));
}
afterEach(() => vi.unstubAllGlobals());

it("accepts the curated native tournament projection", async () => {
  serve(artifact());
  expect((await loadPublishedBotReport()).matches.length).toBeGreaterThan(0);
});

it.each(["count", "duplicate", "provenance", "rules"])("rejects malformed published %s", async (part) => {
  const report = artifact();
  if (part === "count") report.matches[0].move_count += 1;
  else if (part === "duplicate") report.matches[0].move_cells[1] = report.matches[0].move_cells[0];
  else if (part === "provenance") report.provenance.git_commit = { revision: "abc" };
  else report.run.rules.board_size += 1;
  serve(report);
  const result = await loadPublishedBotReport().then(() => "accepted", (error: Error) => error.message);
  expect(result).toContain("unsupported schema");
});
