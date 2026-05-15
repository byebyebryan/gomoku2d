import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { MemoryRouter } from "react-router-dom";

import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
} from "../game/local_match_session";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";

import { SettingsRoute } from "./SettingsRoute";

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

function renderSettingsRoute() {
  render(
    <MemoryRouter>
      <SettingsRoute />
    </MemoryRouter>,
  );
}

describe("SettingsRoute", () => {
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
  });

  it("persists the selected rule and bot settings", () => {
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /renju/i }));
    fireEvent.click(screen.getByRole("button", { name: /hard/i }));

    expect(localProfileStore.getState().settings).toEqual({
      practiceBot: { mode: "preset", preset: "hard", version: 1 },
      preferredVariant: "renju",
    });
    expect(screen.getByRole("group", { name: /renju hard bot/i })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Renju" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Hard Bot" })).toBeInTheDocument();
  });

  it("shows rule selection as a compact setting row", () => {
    renderSettingsRoute();

    expect(screen.getAllByText(/^Game$/)).toHaveLength(2);
    expect(screen.getByText(/^Rule$/)).toBeInTheDocument();
    expect(screen.getByText(/^Ruleset for new games.$/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Freestyle" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Renju" })).toBeInTheDocument();
  });

  it("shows lab controls as setting labels with option segments", () => {
    renderSettingsRoute();

    expect(screen.getByText(/^Advanced Controls$/)).toBeInTheDocument();
    expect(screen.queryByText(/^Lab Controls$/)).not.toBeInTheDocument();
    expect(screen.getByText(/^Scoring$/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Simple" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Threat" })).toBeInTheDocument();
    expect(screen.getByText(/^Extra pass$/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "None" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Corridor proof" })).toBeInTheDocument();
  });

  it("persists lab control option selections", () => {
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: "Simple" }));
    fireEvent.click(screen.getByRole("button", { name: "Corridor proof" }));

    expect(localProfileStore.getState().settings.practiceBot).toMatchObject({
      corridorProof: true,
      mode: "custom",
      patternScoring: false,
      version: 1,
    });
  });

  it("shows active-game apply actions when saved setup differs from current setup", () => {
    const matchStore = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(matchStore.getState().placeHumanMove(7, 7)).toBe(true);

    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /renju/i }));

    expect(screen.getByText(/saved settings apply next game/i)).toBeInTheDocument();
    expect(screen.getAllByRole("link", { name: /back to game/i })[0]).toHaveAttribute("href", "/match/local");

    fireEvent.click(screen.getByRole("button", { name: /start new game/i }));

    expect(matchStore.getState()).toMatchObject({
      currentVariant: "renju",
      moves: [],
      selectedVariant: "renju",
    });
  });
});
