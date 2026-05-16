import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
  localMatchSessionStore,
} from "../game/local_match_session";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";
import { uiPreferencesStore } from "../profile/ui_preferences_store";

import { LocalMatchRoute } from "./LocalMatchRoute";
import { Board } from "../components/Board/Board";

vi.mock("../components/Board/Board", () => ({
  Board: vi.fn(() => <div data-testid="mock-board" />),
}));

const initialLocalProfileState = localProfileStore.getState();
const initialUiPreferencesState = uiPreferencesStore.getState();
const localProfile = {
  avatarUrl: null,
  createdAt: "2026-05-15T00:00:00.000Z",
  displayName: "Bryan",
  id: "local-1",
  kind: "local" as const,
  updatedAt: "2026-05-15T00:00:00.000Z",
  username: null,
};
const noOpBotRunner = {
  chooseMove: async () => null,
  configure: () => undefined,
  dispose: () => undefined,
};
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
    uiPreferencesStore.setState(initialUiPreferencesState, true);
    vi.unstubAllGlobals();
    mockedBoard.mockClear();
  });

  beforeEach(() => {
    localProfileStore.setState({
      matchHistory: emptyLocalMatchHistory(),
      profile: localProfile,
      settings: { practiceBot: DEFAULT_PRACTICE_BOT_CONFIG, preferredVariant: "freestyle" },
    });
    ensureLocalMatchSession({ botRunner: noOpBotRunner });
  });

  it("shows the bot config summary as the bot subtitle without repeating the full setup", () => {
    renderLocalMatchRoute();

    const botValue = screen.getByTestId("match-bot");

    expect(within(botValue).getByText("Normal")).toBeInTheDocument();
    expect(within(botValue).getByText("D3 · full · threat")).toBeInTheDocument();
    expect(screen.queryByText("Freestyle · Normal")).not.toBeInTheDocument();
  });

  it("passes the selected touch control mode to compact mobile boards", () => {
    mockCompactTouchDevice(true);
    uiPreferencesStore.getState().setTouchControl("pointer");

    renderLocalMatchRoute();

    const latestBoardProps = mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];

    expect(latestBoardProps).toMatchObject({
      touchControlMode: "pointer",
    });
  });

  it("filters optional board hints while keeping overlapping forbidden moves visible", () => {
    const matchStore = localMatchSessionStore.getState().matchStore;
    matchStore?.setState({
      counterThreatMoves: [{ row: 4, col: 4 }],
      forbiddenMoves: [{ row: 8, col: 8 }],
      imminentThreatMoves: [{ row: 3, col: 3 }],
      threatMoves: [{ row: 8, col: 8 }],
      winningMoves: [{ row: 1, col: 1 }],
    });
    uiPreferencesStore.setState({
      boardHints: {
        immediate: "win",
        imminent: "off",
      },
    });

    renderLocalMatchRoute();

    const latestBoardProps = mockedBoard.mock.calls[mockedBoard.mock.calls.length - 1]?.[0];

    expect(latestBoardProps).toMatchObject({
      counterThreatMoves: [],
      forbiddenMoves: [{ row: 8, col: 8 }],
      imminentThreatMoves: [],
      threatMoves: [],
      winningMoves: [{ row: 1, col: 1 }],
    });
  });
});
