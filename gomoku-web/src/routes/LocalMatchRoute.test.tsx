import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
} from "../game/local_match_session";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";

import { LocalMatchRoute } from "./LocalMatchRoute";

vi.mock("../components/Board/Board", () => ({
  Board: () => <div data-testid="mock-board" />,
}));

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
const noOpBotRunner = {
  chooseMove: async () => null,
  configure: () => undefined,
  dispose: () => undefined,
};

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
});
