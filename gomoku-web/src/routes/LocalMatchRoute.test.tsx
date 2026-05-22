import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
  localMatchSessionStore,
} from "../game/local_match_session";
import { localProfileStore } from "../profile/local_profile_store";
import {
  createLocalProfileTestState,
  noOpBotRunner,
} from "../test/local_match_fixtures";

import { LocalMatchRoute } from "./LocalMatchRoute";
import { Board } from "../components/Board/Board";

vi.mock("../components/Board/Board", () => ({
  Board: vi.fn(() => <div data-testid="mock-board" />),
}));

const initialLocalProfileState = localProfileStore.getState();
const mockedBoard = vi.mocked(Board);

function mockCompactTouchDevice(matches: boolean) {
  vi.stubGlobal("matchMedia", vi.fn().mockReturnValue({
    addEventListener: vi.fn(),
    matches,
    removeEventListener: vi.fn(),
  }));
}

function renderLocalMatchRoute() {
  render(
    <MemoryRouter>
      <LocalMatchRoute />
    </MemoryRouter>,
  );
}

describe("LocalMatchRoute", () => {
  afterEach(() => {
    cleanup();
    disposeLocalMatchSession();
    localProfileStore.setState(initialLocalProfileState, true);
    vi.useRealTimers();
    vi.unstubAllGlobals();
    mockedBoard.mockClear();
  });

  beforeEach(() => {
    localProfileStore.setState(createLocalProfileTestState());
    ensureLocalMatchSession({ botRunner: noOpBotRunner });
  });

  it("shows the bot config summary as the bot subtitle without repeating the full setup", () => {
    renderLocalMatchRoute();

    const botValue = screen.getByTestId("match-bot");

    expect(within(botValue).getByText("Normal")).toBeInTheDocument();
    expect(within(botValue).getByText("D3 · full · pattern")).toBeInTheDocument();
    expect(screen.queryByText("Freestyle · Normal")).not.toBeInTheDocument();
  });

  it("passes the selected touch control mode to compact mobile boards", () => {
    mockCompactTouchDevice(true);
    localProfileStore.getState().updateSettings({ touchControl: "pointer" });

    renderLocalMatchRoute();

    const latestBoardProps = mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];

    expect(latestBoardProps?.model.interaction).toMatchObject({
      touchControlMode: "pointer",
    });
  });

  it("filters optional board hints while keeping overlapping forbidden moves visible", () => {
    const matchStore = localMatchSessionStore.getState().matchStore;
    matchStore?.setState({
      counterThreatEvidenceCells: [{ row: 4, col: 3 }],
      counterThreatMoves: [{ row: 4, col: 4 }],
      forbiddenMoves: [{ row: 8, col: 8 }],
      immediateThreatEvidenceCells: [{ row: 8, col: 7 }],
      imminentThreatEvidenceCells: [{ row: 3, col: 2 }],
      imminentThreatMoves: [{ row: 3, col: 3 }],
      threatMoves: [{ row: 8, col: 8 }],
      winningEvidenceCells: [{ row: 1, col: 0 }],
      winningMoves: [{ row: 1, col: 1 }],
    });
    localProfileStore.getState().updateSettings({
      boardHints: {
        evidence: "on",
        immediate: "win",
        imminent: "off",
      },
    });

    renderLocalMatchRoute();

    const latestBoardProps = mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];

    expect(latestBoardProps?.model).toMatchObject({
      forbiddenMoves: [{ row: 8, col: 8 }],
    });
    expect(latestBoardProps?.model.overlays).toEqual([
      { cell: { row: 1, col: 0 }, kind: "evidence", role: "winning" },
      { cell: { row: 1, col: 1 }, kind: "hint", role: "winning" },
    ]);
  });

  it("can hide source-stone evidence while keeping hint targets visible", () => {
    const matchStore = localMatchSessionStore.getState().matchStore;
    matchStore?.setState({
      counterThreatEvidenceCells: [{ row: 4, col: 3 }],
      counterThreatMoves: [{ row: 4, col: 4 }],
      forbiddenMoves: [],
      immediateThreatEvidenceCells: [{ row: 8, col: 7 }],
      imminentThreatEvidenceCells: [{ row: 3, col: 2 }],
      imminentThreatMoves: [{ row: 3, col: 3 }],
      threatMoves: [{ row: 8, col: 8 }],
      winningEvidenceCells: [{ row: 1, col: 0 }],
      winningMoves: [{ row: 1, col: 1 }],
    });
    localProfileStore.getState().updateSettings({
      boardHints: {
        evidence: "off",
        immediate: "win_threat",
        imminent: "threat_counter",
      },
    });

    renderLocalMatchRoute();

    const latestBoardProps = mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];

    expect(latestBoardProps?.model.overlays).toEqual([
      { cell: { row: 1, col: 1 }, kind: "hint", role: "winning" },
      { cell: { row: 8, col: 8 }, kind: "hint", role: "immediateThreat" },
      { cell: { row: 3, col: 3 }, kind: "hint", role: "imminentThreat" },
      { cell: { row: 4, col: 4 }, kind: "hint", role: "counterThreat" },
    ]);
  });

  it("shows compact player totals and the active move timer in player cards", () => {
    vi.useFakeTimers();
    vi.setSystemTime(5_000);
    const matchStore = localMatchSessionStore.getState().matchStore;
    matchStore?.setState({
      currentPlayer: 2,
      pendingBotMove: true,
      playerClockMs: [12_300, 38_000],
      status: "playing",
      turnStartedAtMs: 2_600,
    });

    renderLocalMatchRoute();

    const black = screen.getByTestId("player-row-black");
    const white = screen.getByTestId("player-row-white");

    expect(within(black).getByText("12.3s")).toBeInTheDocument();
    expect(within(black).queryByText(/^\+/)).not.toBeInTheDocument();
    expect(within(white).getByText("38.0s")).toBeInTheDocument();
    expect(within(white).getByText("+2.4s")).toBeInTheDocument();
  });
});
