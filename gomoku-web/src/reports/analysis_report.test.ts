import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";

import { loadAnalysisReport } from "./analysis_report";

function artifact() {
  return JSON.parse(readFileSync("../reports/lab/analysis-report.json", "utf8"));
}

function serve(data: unknown) {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify(data))));
}

afterEach(() => vi.unstubAllGlobals());

describe("published analysis contract", () => {
  it("loads the curated native output with its actual failure representation", async () => {
    const source = artifact();
    serve(source);
    const report = await loadAnalysisReport();
    const entries = report.sections.flatMap((section) => section.entries);
    expect(entries.length).toBeGreaterThan(0);
    const failure = entries.find((entry) => entry.failure?.missed_candidates?.length)?.failure;
    expect(failure).toMatchObject({
      prefix_ply: expect.any(Number),
      confidence: expect.any(String),
    });
    expect(failure?.missed_candidates).toEqual(expect.arrayContaining([
      expect.objectContaining({ roles: expect.any(Array), outcome: expect.any(String) }),
    ]));
  });

  it.each(["failure", "onset", "interval", "cell", "count"])("rejects malformed consumed %s data", async (part) => {
    const report = artifact();
    const entry = report.sections.flatMap((section: { entries: unknown[] }) => section.entries)
      .find((candidate: { failure?: unknown }) => candidate.failure);
    if (part === "failure") entry.failure.actual_notation = { text: "J8" };
    else if (part === "onset") entry.lethal_onset.shape.label = 4;
    else if (part === "interval") entry.setup_corridor = { start_ply: 9, end_ply: 3 };
    else if (part === "cell") entry.match_report.move_cells[0] = 225;
    else entry.match_report.move_count += 1;
    serve(report);
    const result = await loadAnalysisReport().then(() => "accepted", (error: Error) => error.message);
    expect(result).toContain("unsupported schema");
  });
});
