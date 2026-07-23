import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import type { PublishedAnalysisReport } from "../reports/analysis_report";
import type { PublishedBotReport, StandingReport } from "../reports/bot_report";

import { LabReportRoute } from "./LabReportRoute";
import { BotReportPanel } from "./lab-report/BotReportPanel";

const reportLoaders = vi.hoisted(() => ({
  analysis: vi.fn(),
  bot: vi.fn(),
}));

vi.mock("../reports/analysis_report", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../reports/analysis_report")>()),
  loadAnalysisReport: reportLoaders.analysis,
}));

vi.mock("../reports/bot_report", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../reports/bot_report")>()),
  loadPublishedBotReport: reportLoaders.bot,
}));

function standing(bot: string): StandingReport {
  return {
    bot,
    wins: 1,
    draws: 0,
    losses: 0,
    sequential_elo: 1516,
    shuffled_elo_avg: 1514,
    shuffled_elo_stddev: 4,
    match_count: 1,
    move_count: 3,
    search_move_count: 2,
    total_time_ms: 20,
    avg_search_time_ms: 10,
    total_nodes: 20,
    avg_nodes: 10,
    avg_depth: 3,
    max_depth: 3,
    budget_exhausted_rate: 0,
  };
}

function botReport(): PublishedBotReport {
  const normal = "search-d3+pattern-eval";
  const easy = "search-d1";
  return {
    schema_version: 2,
    report_kind: "published_tournament",
    source_schema_version: 1,
    board_size: 15,
    move_codec: "cell_index_v1",
    run: {
      bots: [normal, easy],
      schedule: "round-robin",
      rules: { board_size: 15, win_length: 5, variant: "renju" },
      games_per_pair: 1,
      seed: 0,
      opening_plies: 0,
      opening_policy: "centered-suite",
      threads: 1,
      total_wall_time_ms: 20,
    },
    standings: [standing(normal), { ...standing(easy), wins: 0, losses: 1 }],
    pairwise: [
      {
        bot_a: normal,
        bot_b: easy,
        wins_a: 1,
        wins_b: 0,
        draws: 0,
        total: 1,
        score_a: 1,
        score_b: 0,
      },
    ],
    end_reasons: [{ key: "natural", count: 1 }],
    matches: [
      {
        match_index: 1,
        black: normal,
        white: easy,
        result: "black_win",
        winner: normal,
        end_reason: "natural",
        move_cells: [112, 96, 113],
        move_count: 3,
      },
    ],
  };
}

function analysisReport(): PublishedAnalysisReport {
  return {
    schema_version: 4,
    report_kind: "published_analysis",
    source_kind: "published_tournament",
    source_report: "reports/lab/bot-report.json",
    selector: "preset-triangle",
    total: 1,
    analyzed: 1,
    failed: 0,
    elapsed_ms: 10,
    total_elapsed_ms: 10,
    model: { max_depth: 4, max_scan_plies: 64 },
    summary: { unclear: 0, ongoing_or_draw: 0, analysis_error: 0 },
    sections: [
      {
        label: "Normal vs Easy",
        entrant_a: "search-d3+pattern-eval",
        entrant_b: "search-d1",
        total: 1,
        analyzed: 1,
        failed: 0,
        summary: { unclear: 0, ongoing_or_draw: 0, analysis_error: 0 },
        entries: [
          {
            path: "match-1",
            match_report: {
              match_index: 1,
              black: "search-d3+pattern-eval",
              white: "search-d1",
              result: "black_win",
              winner: "search-d3+pattern-eval",
              end_reason: "natural",
              move_cells: [112, 96, 113],
              move_count: 3,
            },
            status: "analyzed",
            root_cause: "missed_defense",
            setup_corridor: { start_ply: 1, end_ply: 3 },
            last_chance_ply: 0,
            critical_loser_ply: 1,
            failure: null,
            proof_details: {
              proof_frames: [
                {
                  label: "winning",
                  ply: 3,
                  side_to_move: "White",
                  status: "forced_win",
                  move_played_notation: "I8",
                  lethal_onset_reached: true,
                  markers: [],
                  reply_outcomes: [],
                },
              ],
            },
            search_details: { search_nodes: 12, branch_probes: 3, max_depth_reached: 4 },
            elapsed_ms: 10,
          },
        ],
      },
    ],
  };
}

function renderRoute(entry = "/lab/") {
  return render(
    <MemoryRouter initialEntries={[entry]}>
      <LabReportRoute />
    </MemoryRouter>,
  );
}

describe("LabReportRoute", () => {
  beforeEach(() => {
    reportLoaders.bot.mockResolvedValue(botReport());
    reportLoaders.analysis.mockResolvedValue(analysisReport());
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
    vi.stubGlobal("cancelAnimationFrame", vi.fn());
    HTMLElement.prototype.scrollIntoView = vi.fn();
  });

  afterEach(() => {
    cleanup();
    reportLoaders.bot.mockReset();
    reportLoaders.analysis.mockReset();
    vi.unstubAllGlobals();
  });

  it("loads only the active report and switches tabs lazily", async () => {
    renderRoute();

    expect(await screen.findByText("search-d3")).toBeInTheDocument();
    expect(reportLoaders.bot).toHaveBeenCalledOnce();
    expect(reportLoaders.analysis).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("tab", { name: "Analysis" }));

    await waitFor(() => expect(reportLoaders.analysis).toHaveBeenCalledOnce());
    expect(await screen.findByText("search-d3+pattern")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Analysis" })).toHaveAttribute(
      "aria-selected",
      "true",
    );
  });

  it("shows report loading failures", async () => {
    reportLoaders.bot.mockRejectedValueOnce(new Error("report unavailable"));

    renderRoute();

    expect(await screen.findByText("report unavailable")).toBeInTheDocument();
  });

  it("opens a deep-linked analysis match", async () => {
    renderRoute("/lab/?tab=analysis&match=match-1");

    expect(await screen.findByText("Search details")).toBeInTheDocument();
    expect(screen.getByText("10ms / 3 probes / 12 nodes / d4")).toBeInTheDocument();
    expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();
  });
});

describe("BotReportPanel", () => {
  afterEach(cleanup);

  it("defers opponent, match, and board rendering until each row opens", () => {
    const { container } = render(<BotReportPanel report={botReport()} view="ranking" />);

    expect(screen.queryByRole("img", { name: /finished board/i })).not.toBeInTheDocument();

    const entrant = container.querySelector("details");
    fireEvent.click(entrant?.querySelector("summary") as HTMLElement);
    const opponent = entrant?.querySelector("details");
    expect(opponent).not.toBeNull();

    fireEvent.click(opponent?.querySelector("summary") as HTMLElement);
    const match = opponent?.querySelector("details");
    expect(match).not.toBeNull();
    expect(screen.queryByRole("img", { name: /finished board/i })).not.toBeInTheDocument();

    fireEvent.click(match?.querySelector("summary") as HTMLElement);
    expect(screen.getByRole("img", { name: /finished board/i })).toBeInTheDocument();
  });
});
