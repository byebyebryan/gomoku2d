import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";

import { Board } from "../components/Board/Board";
import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import type { ReplayAnalysisCachedResult } from "../replay/replay_analysis_cache";
import type { ReplayAnalysisCallbacks } from "../replay/replay_analysis_runner";

import { ReplayRoute } from "./ReplayRoute";

const runnerMock = vi.hoisted(() => ({
  callbacks: null as ReplayAnalysisCallbacks | null,
  instances: [] as Array<{
    analyze: ReturnType<typeof vi.fn>;
    cancel: ReturnType<typeof vi.fn>;
    dispose: ReturnType<typeof vi.fn>;
  }>,
}));
const cacheMock = vi.hoisted(() => ({
  read: vi.fn((): ReplayAnalysisCachedResult | null => null),
  write: vi.fn(),
}));

vi.mock("../components/Board/Board", () => ({
  Board: vi.fn(() => <div data-testid="mock-board" />),
}));

vi.mock("../replay/replay_analysis_runner", () => ({
  ReplayAnalysisRunner: vi.fn().mockImplementation(function () {
    const instance = {
      analyze: vi.fn((_match, callbacks: ReplayAnalysisCallbacks) => {
        runnerMock.callbacks = callbacks;
        return 1;
      }),
      cancel: vi.fn(),
      dispose: vi.fn(),
    };
    runnerMock.instances.push(instance);
    return instance;
  }),
}));

vi.mock("../replay/replay_analysis_cache", () => ({
  readReplayAnalysisCache: cacheMock.read,
  writeReplayAnalysisCache: cacheMock.write,
}));

vi.mock("../replay/local_replay_core", () => ({
  winningCellsFromCore: vi.fn(() => []),
}));

const mockedBoard = vi.mocked(Board);
const initialLocalProfileState = localProfileStore.getState();

const localProfile = {
  avatarUrl: null,
  createdAt: "2026-05-15T00:00:00.000Z",
  displayName: "Bryan",
  id: "local-1",
  kind: "local" as const,
  updatedAt: "2026-05-15T00:00:00.000Z",
  username: null,
};

const MOVES = [
  { col: 5, moveNumber: 1, player: 1 as const, row: 7 },
  { col: 0, moveNumber: 2, player: 2 as const, row: 0 },
  { col: 6, moveNumber: 3, player: 1 as const, row: 7 },
  { col: 1, moveNumber: 4, player: 2 as const, row: 0 },
  { col: 7, moveNumber: 5, player: 1 as const, row: 7 },
  { col: 2, moveNumber: 6, player: 2 as const, row: 0 },
  { col: 8, moveNumber: 7, player: 1 as const, row: 7 },
  { col: 3, moveNumber: 8, player: 2 as const, row: 0 },
  { col: 9, moveNumber: 9, player: 1 as const, row: 7 },
];

function localMatch(id: string): LocalProfileSavedMatch {
  return createLocalSavedMatch({
    id,
    localProfileId: "local-1",
    moves: MOVES,
    players: [
      { kind: "human", name: "Black", stone: "black" },
      { kind: "bot", name: "White", stone: "white" },
    ],
    ruleset: "renju",
    savedAt: "2026-05-16T12:00:00.000Z",
    status: "black_won",
  });
}

function renderReplayRoute(matchId = "match-1") {
  render(
    <MemoryRouter initialEntries={[`/replay/${matchId}`]}>
      <Routes>
        <Route element={<ReplayRoute />} path="/replay/:matchId" />
      </Routes>
    </MemoryRouter>,
  );
}

function latestBoardProps() {
  return mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];
}

describe("ReplayRoute analysis overlays", () => {
  afterEach(() => {
    cleanup();
    localProfileStore.setState(initialLocalProfileState, true);
    mockedBoard.mockClear();
    runnerMock.callbacks = null;
    runnerMock.instances = [];
    cacheMock.read.mockReset();
    cacheMock.read.mockReturnValue(null);
    cacheMock.write.mockReset();
  });

  beforeEach(() => {
    localProfileStore.setState({
      matchHistory: {
        ...emptyLocalMatchHistory(),
        replayMatches: [localMatch("match-1"), localMatch("match-2")],
      },
      profile: localProfile,
      settings: createDefaultProfileSettings(),
    });
  });

  it("starts replay analysis one frame at a time and previews the next actual move", () => {
    renderReplayRoute();

    const instance = runnerMock.instances[0];
    expect(instance.analyze).toHaveBeenCalledWith(
      expect.objectContaining({ id: "match-1" }),
      expect.any(Object),
      { maxDepth: 4, maxScanPlies: 64 },
      1,
    );
    expect(screen.getByTestId("replay-move-count")).toHaveTextContent("Move 9 / 9");
    expect(latestBoardProps()).toMatchObject({
      nextReplayMove: null,
      status: "black_won",
    });
  });

  it("uses outer replay controls for turn stepping", () => {
    renderReplayRoute();

    fireEvent.click(screen.getByRole("button", { name: "Previous turn" }));
    expect(screen.getByTestId("replay-move-count")).toHaveTextContent("Move 7 / 9");
    expect(latestBoardProps()).toMatchObject({
      currentPlayer: 2,
      nextReplayMove: { row: 0, col: 3 },
      status: "playing",
    });

    fireEvent.click(screen.getByRole("button", { name: "Next turn" }));
    expect(screen.getByTestId("replay-move-count")).toHaveTextContent("Move 9 / 9");
  });

  it("adds loser-frame analysis overlays from worker progress", () => {
    renderReplayRoute();

    act(() => {
      runnerMock.callbacks?.onProgress?.({
        analysis: null,
        annotations: [
          {
            highlights: [
              {
                mv: { col: 6, row: 7 },
                notation: "G8",
                role: "immediate_threat",
                side: "Black",
              },
            ],
            markers: [
              {
                mv: { col: 8, row: 7 },
                notation: "I8",
                role: "forced_loss",
                side: "White",
              },
            ],
            ply: 9,
            side_to_move: "White",
          },
          {
            highlights: [
              {
                mv: { col: 10, row: 7 },
                notation: "K8",
                role: "corridor_entry",
                side: "Black",
              },
            ],
            markers: [],
            ply: 5,
            side_to_move: "Black",
          },
        ],
        counters: { branch_roots: 0, prefixes_analyzed: 2, proof_nodes: 0 },
        current_ply: 3,
        done: false,
        error: null,
        schema_version: 1,
        status: "running",
      });
    });

    expect(latestBoardProps()?.analysisOverlays).toEqual([
      expect.objectContaining({ highlight: "immediateThreat", row: 7, col: 6 }),
      expect.objectContaining({ marker: "forcedLoss", row: 7, col: 8 }),
    ]);
  });

  it("hydrates completed replay analysis from local cache before starting the worker", () => {
    cacheMock.read.mockReturnValue({
      annotationsByPly: {
        9: {
          highlights: [
            {
              mv: { col: 6, row: 7 },
              notation: "G8",
              role: "immediate_threat",
              side: "Black",
            },
          ],
          markers: [],
          ply: 9,
          side_to_move: "White",
        },
      },
      step: {
        analysis: { schema_version: 1 },
        annotations: [],
        counters: { branch_roots: 2, prefixes_analyzed: 3, proof_nodes: 321 },
        current_ply: null,
        done: true,
        error: null,
        schema_version: 1,
        status: "resolved",
      },
    });

    renderReplayRoute();

    expect(runnerMock.instances).toHaveLength(0);
    expect(latestBoardProps()?.analysisOverlays).toEqual([
      expect.objectContaining({ highlight: "immediateThreat", row: 7, col: 6 }),
    ]);
  });

  it("stores completed replay analysis with accumulated annotations", () => {
    renderReplayRoute();
    const progressAnnotation = {
      highlights: [],
      markers: [
        {
          mv: { col: 8, row: 7 },
          notation: "I8",
          role: "forced_loss" as const,
          side: "White" as const,
        },
      ],
      ply: 9,
      side_to_move: "White" as const,
    };
    const completeStep = {
      analysis: { schema_version: 1 },
      annotations: [
        {
          highlights: [],
          markers: [],
          ply: 7,
          side_to_move: "White" as const,
        },
      ],
      counters: { branch_roots: 2, prefixes_analyzed: 3, proof_nodes: 321 },
      current_ply: null,
      done: true,
      error: null,
      schema_version: 1,
      status: "resolved" as const,
    };

    act(() => {
      runnerMock.callbacks?.onProgress?.({
        analysis: null,
        annotations: [progressAnnotation],
        counters: { branch_roots: 1, prefixes_analyzed: 1, proof_nodes: 1 },
        current_ply: 9,
        done: false,
        error: null,
        schema_version: 1,
        status: "running",
      });
      runnerMock.callbacks?.onComplete?.(completeStep);
    });

    expect(cacheMock.write).toHaveBeenCalledWith(
      expect.objectContaining({ id: "match-1" }),
      { maxDepth: 4, maxScanPlies: 64 },
      {
        annotationsByPly: {
          9: progressAnnotation,
          7: completeStep.annotations[0],
        },
        step: completeStep,
      },
    );
  });

  it("shows analysis status and traceback progress timeline markers", () => {
    renderReplayRoute();

    expect(screen.getByText("Status")).toBeInTheDocument();
    expect(screen.getByTestId("replay-analysis-status")).toHaveTextContent("Analyzing replay");

    act(() => {
      runnerMock.callbacks?.onProgress?.({
        analysis: null,
        annotations: [
          {
            highlights: [],
            markers: [
              {
                mv: { col: 8, row: 7 },
                notation: "I8",
                role: "forced_loss",
                side: "White",
              },
            ],
            ply: 7,
            side_to_move: "White",
          },
          {
            highlights: [],
            markers: [
              {
                mv: { col: 6, row: 7 },
                notation: "G8",
                role: "confirmed_escape",
                side: "White",
              },
            ],
            ply: 5,
            side_to_move: "White",
          },
        ],
        counters: { branch_roots: 2, prefixes_analyzed: 3, proof_nodes: 321 },
        current_ply: 5,
        done: false,
        error: null,
        schema_version: 1,
        status: "running",
      });
    });

    expect(screen.getByTestId("replay-analysis-status")).toHaveTextContent("Analyzing replay");
    expect(screen.getByTestId("replay-analysis-detail")).toHaveTextContent("Move 5 · 321 nodes");
    expect(screen.getByTestId("replay-timeline-analyzed")).toBeInTheDocument();
    expect(screen.queryByTestId("replay-timeline-setup-corridor")).not.toBeInTheDocument();
    expect(screen.getByTestId("replay-timeline-escape")).toBeInTheDocument();
  });

  it("does not show a classified mistake shortcut", () => {
    renderReplayRoute();

    act(() => {
      runnerMock.callbacks?.onComplete?.({
        analysis: {
          failure: {
            actual_move: { col: 2, row: 0 },
            actual_notation: "C1",
            confidence: "confirmed",
            missed_candidates: [
              {
                mv: { col: 6, row: 7 },
                notation: "G8",
                outcome: "confirmed_escape",
                roles: ["imminent_defense"],
              },
            ],
            mode: "missed_imminent_response",
            prefix_ply: 5,
            prevented_onset_ply: null,
            side: "White",
          },
          setup_corridor: { start_ply: 5, end_ply: 8 },
        },
        annotations: [],
        counters: { branch_roots: 1, prefixes_analyzed: 4, proof_nodes: 512 },
        current_ply: null,
        done: true,
        error: null,
        schema_version: 1,
        status: "resolved",
      });
    });

    expect(screen.getByTestId("replay-analysis-status")).toHaveTextContent("Black has won");
    expect(screen.getByTestId("replay-analysis-detail")).toHaveTextContent("Lethal sequence");
    expect(screen.queryByRole("button", { name: "Review Mistake" })).not.toBeInTheDocument();
  });

  it("does not show mistake controls for non-actionable failures", () => {
    renderReplayRoute();

    act(() => {
      runnerMock.callbacks?.onComplete?.({
        analysis: {
          failure: {
            actual_move: null,
            actual_notation: null,
            confidence: "confirmed",
            missed_candidates: [],
            mode: "missed_escape",
            prefix_ply: 7,
            prevented_onset_ply: 7,
            side: "White",
          },
        },
        annotations: [],
        counters: { branch_roots: 1, prefixes_analyzed: 4, proof_nodes: 512 },
        current_ply: null,
        done: true,
        error: null,
        schema_version: 1,
        status: "resolved",
      });
    });

    expect(screen.getByTestId("replay-analysis-status")).toHaveTextContent("Black has won");
    expect(screen.getByTestId("replay-analysis-detail")).toHaveTextContent("Lethal sequence");
    expect(screen.queryByRole("button", { name: "Review Mistake" })).not.toBeInTheDocument();
    expect(latestBoardProps()?.analysisOverlays).toEqual([]);
  });

  it("cancels analysis when leaving the replay", () => {
    const { unmount } = render(
      <MemoryRouter initialEntries={["/replay/match-1"]}>
        <Routes>
          <Route element={<ReplayRoute />} path="/replay/:matchId" />
        </Routes>
      </MemoryRouter>,
    );

    unmount();

    expect(runnerMock.instances[0].cancel).toHaveBeenCalled();
    expect(runnerMock.instances[0].dispose).toHaveBeenCalled();
  });
});
