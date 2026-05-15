import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import { DEFAULT_PRACTICE_BOT_CONFIG } from "../core/practice_bot_config";
import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
} from "../game/local_match_session";
import { emptyLocalMatchHistory, localProfileStore } from "../profile/local_profile_store";
import { uiPreferencesStore } from "../profile/ui_preferences_store";

import { SettingsRoute } from "./SettingsRoute";

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

function renderSettingsRoute() {
  render(
    <MemoryRouter>
      <SettingsRoute />
    </MemoryRouter>,
  );
}

function mockSettingsMedia(options: { compact?: boolean; touch?: boolean }) {
  vi.stubGlobal("matchMedia", vi.fn((query: string) => ({
    addEventListener: vi.fn(),
    matches: query.includes("pointer: coarse")
      ? Boolean(options.touch)
      : query.includes("max-width: 760px") && Boolean(options.compact),
    removeEventListener: vi.fn(),
  })));
}

function mockCompactTouchDevice(matches: boolean) {
  mockSettingsMedia({ compact: matches, touch: matches });
}

describe("SettingsRoute", () => {
  afterEach(() => {
    cleanup();
    disposeLocalMatchSession();
    localProfileStore.setState(initialLocalProfileState, true);
    uiPreferencesStore.setState(initialUiPreferencesState, true);
    vi.unstubAllGlobals();
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

  it("hides touch control on non-mobile settings screens", () => {
    renderSettingsRoute();

    expect(screen.queryByText(/^Controls$/)).not.toBeInTheDocument();
    expect(screen.queryByText(/^Touch control$/)).not.toBeInTheDocument();
  });

  it("hides the current settings summary on compact settings screens", async () => {
    mockSettingsMedia({ compact: true });
    renderSettingsRoute();

    await waitFor(() => {
      expect(screen.queryByText(/^Current settings$/)).not.toBeInTheDocument();
      expect(screen.queryByRole("group", { name: /freestyle normal bot/i })).not.toBeInTheDocument();
    });
  });

  it("keeps the apply-next-game panel visible on compact settings screens", async () => {
    mockSettingsMedia({ compact: true });
    const matchStore = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(matchStore.getState().placeHumanMove(7, 7)).toBe(true);

    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /renju/i }));

    expect(await screen.findByText(/saved settings apply next game/i)).toBeInTheDocument();
    expect(screen.queryByText(/^Current settings$/)).not.toBeInTheDocument();
    expect(screen.queryByRole("group", { name: /renju normal bot/i })).not.toBeInTheDocument();
  });

  it("shows touch control as a device-local control on compact touch screens", () => {
    mockCompactTouchDevice(true);
    renderSettingsRoute();

    expect(screen.getByText(/^Controls$/)).toBeInTheDocument();
    expect(screen.getByText(/^Touch control$/)).toBeInTheDocument();
    expect(screen.getByText(/^How mobile taps move the board cursor.$/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Pointer" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Touchpad" })).toBeInTheDocument();
  });

  it("persists touch control as a UI preference", () => {
    mockCompactTouchDevice(true);
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: "Pointer" }));

    expect(uiPreferencesStore.getState().touchControl).toBe("pointer");
    expect(localProfileStore.getState().settings).toEqual({
      practiceBot: DEFAULT_PRACTICE_BOT_CONFIG,
      preferredVariant: "freestyle",
    });
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
