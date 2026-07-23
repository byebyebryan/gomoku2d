import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import {
  disposeLocalMatchSession,
  ensureLocalMatchSession,
} from "../game/local_match_session";
import { localProfileStore } from "../profile/local_profile_store";
import { createDefaultProfileSettings } from "../profile/profile_settings";
import {
  createLocalProfileTestState,
  noOpBotRunner,
} from "../test/local_match_fixtures";

import { SettingsRoute } from "./SettingsRoute";

const initialLocalProfileState = localProfileStore.getState();
const defaultSettings = createDefaultProfileSettings();

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
    vi.unstubAllGlobals();
  });

  beforeEach(() => {
    localProfileStore.setState(createLocalProfileTestState(defaultSettings));
  });

  it("persists the selected rule and bot settings", () => {
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /renju/i }));
    fireEvent.click(screen.getByRole("button", { name: /hard/i }));

    expect(localProfileStore.getState().settings).toEqual({
      ...defaultSettings,
      botConfig: { mode: "preset", preset: "hard", version: 1 },
      gameConfig: {
        opening: "standard",
        ruleset: "renju",
      },
    });
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

    expect(await screen.findByText(/changes apply next game/i)).toBeInTheDocument();
    expect(screen.queryByText(/^Current settings$/)).not.toBeInTheDocument();
    expect(screen.queryByRole("group", { name: /renju normal bot/i })).not.toBeInTheDocument();
  });

  it("keeps compact header actions accessible", () => {
    mockSettingsMedia({ compact: true, touch: true });
    renderSettingsRoute();

    expect(screen.getByRole("link", { name: "Back to Game" })).toHaveAttribute("href", "/match/local");
    expect(screen.getByRole("link", { name: "Profile" })).toHaveAttribute("href", "/profile");
    expect(screen.getByRole("link", { name: "Home" })).toHaveAttribute("href", "/");
  });

  it("shows touch control on compact touch screens", () => {
    mockCompactTouchDevice(true);
    renderSettingsRoute();

    expect(screen.getByRole("button", { name: "Pointer" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Touchpad" })).toBeInTheDocument();
  });

  it("persists touch control in profile settings", () => {
    mockCompactTouchDevice(true);
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: "Pointer" }));

    expect(localProfileStore.getState().settings.touchControl).toBe("pointer");
  });

  it("persists board hint option selections", () => {
    renderSettingsRoute();

    const immediate = screen.getByRole("group", { name: "Immediate hints" });
    const imminent = screen.getByRole("group", { name: "Imminent hints" });
    expect(within(imminent).getByRole("button", { name: "Threat" })).toBeInTheDocument();
    const evidence = screen.getByRole("group", { name: "Evidence hints" });

    fireEvent.click(within(immediate).getByRole("button", { name: "Win" }));
    fireEvent.click(within(evidence).getByRole("button", { name: "Off" }));

    expect(localProfileStore.getState().settings.boardHints).toMatchObject({
      evidence: "off",
      immediate: "win",
      imminent: "threat_counter",
    });
  });

  it("persists lab control option selections", () => {
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: "Simple" }));
    fireEvent.click(screen.getByRole("button", { name: "Corridor proof" }));

    expect(localProfileStore.getState().settings.botConfig).toMatchObject({
      extraPass: "corridor_proof",
      mode: "custom",
      scoring: "simple",
      version: 1,
    });
  });

  it("disables and clamps custom bot widths that are too slow for browser play", () => {
    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /custom/i }));
    fireEvent.click(screen.getByRole("button", { name: "D5" }));

    expect(screen.getByRole("button", { name: "full" })).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: "D7" }));

    expect(screen.getByRole("button", { name: "W16" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "full" })).toBeDisabled();
    expect(localProfileStore.getState().settings.botConfig).toMatchObject({
      depth: 7,
      mode: "custom",
      width: 8,
    });
  });

  it("shows active-game apply actions when saved setup differs from current setup", () => {
    const matchStore = ensureLocalMatchSession({ botRunner: noOpBotRunner });
    expect(matchStore.getState().placeHumanMove(7, 7)).toBe(true);

    renderSettingsRoute();

    fireEvent.click(screen.getByRole("button", { name: /renju/i }));

    expect(screen.getByText(/changes apply next game/i)).toBeInTheDocument();
    expect(screen.getAllByRole("link", { name: /back to game/i })[0]).toHaveAttribute("href", "/match/local");

    fireEvent.click(screen.getByRole("button", { name: /start new game/i }));

    expect(matchStore.getState()).toMatchObject({
      currentVariant: "renju",
      moves: [],
      selectedVariant: "renju",
    });
  });
});
