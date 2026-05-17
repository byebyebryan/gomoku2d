import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import doubleNextSvgRaw from "../../assets/icons/double_next.svg?raw";
import doublePrevSvgRaw from "../../assets/icons/double_prev.svg?raw";
import nextSvgRaw from "../../assets/icons/next.svg?raw";
import prevSvgRaw from "../../assets/icons/prev.svg?raw";

import { Board } from "../components/Board/Board";
import { createLocalSavedMatch } from "../match/saved_match";
import type { LocalProfileSavedMatch } from "../profile/local_profile_store";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
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

function rectSignatureFromSvg(svg: string): string[] {
  return [...svg.matchAll(/<rect x="([^"]+)" y="([^"]+)" width="([^"]+)" height="([^"]+)"/g)].map(
    (match) => `${match[1]},${match[2]},${match[3]},${match[4]}`,
  );
}

function rectSignatureFromButton(button: HTMLElement): string[] {
  return [...button.querySelectorAll("rect")].map((rect) => (
    `${rect.getAttribute("x")},${rect.getAttribute("y")},${rect.getAttribute("width")},${rect.getAttribute("height")}`
  ));
}

describe("ReplayRoute analysis overlays", () => {
  afterEach(() => {
    cleanup();
    localProfileStore.setState(initialLocalProfileState, true);
    mockedBoard.mockClear();
    runnerMock.callbacks = null;
    runnerMock.instances = [];
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

  it("uses double arrows for turn stepping and single arrows for move stepping", () => {
    renderReplayRoute();

    expect(rectSignatureFromButton(screen.getByRole("button", { name: "Previous turn" }))).toEqual(
      rectSignatureFromSvg(doublePrevSvgRaw),
    );
    expect(rectSignatureFromButton(screen.getByRole("button", { name: "Previous move" }))).toEqual(
      rectSignatureFromSvg(prevSvgRaw),
    );
    expect(rectSignatureFromButton(screen.getByRole("button", { name: "Next move" }))).toEqual(
      rectSignatureFromSvg(nextSvgRaw),
    );
    expect(rectSignatureFromButton(screen.getByRole("button", { name: "Next turn" }))).toEqual(
      rectSignatureFromSvg(doubleNextSvgRaw),
    );
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
