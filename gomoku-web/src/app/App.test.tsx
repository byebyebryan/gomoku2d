import {
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter } from "react-router-dom";

import { App } from "./App";

vi.mock("../routes/LocalMatchRoute", () => ({
  LocalMatchRoute: () => (
    <main>
      <h1>Local Match</h1>
      <p>Black to move</p>
    </main>
  ),
}));

vi.mock("../routes/SettingsRoute", () => ({
  SettingsRoute: () => (
    <main>
      <h1>Settings</h1>
      <p>Rule</p>
      <p>Bot</p>
    </main>
  ),
}));

vi.mock("../routes/BotReportRoute", () => ({
  LabReportRoute: () => (
    <main>
      <h1>Lab Report</h1>
    </main>
  ),
}));

vi.mock("../routes/AssetPreviewRoute", () => ({
  AssetPreviewRoute: () => (
    <main>
      <h1>Visual Design</h1>
    </main>
  ),
}));

vi.mock("../routes/RulesRoute", () => ({
  RulesRoute: () => (
    <main>
      <h1>Rules</h1>
    </main>
  ),
}));

describe("App", () => {
  afterEach(() => {
    cleanup();
  });

  it("starts on home and routes into a local bot match", async () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole("heading", { name: /gomoku2d/i }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("link", { name: /play/i }));

    expect(
      await screen.findByRole("heading", { name: /local match/i }),
    ).toBeInTheDocument();
    expect(screen.getByText(/black to move/i)).toBeInTheDocument();
  });

  it("surfaces lab links on the home screen", () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(screen.getByRole("link", { name: /^visuals$/i })).toHaveAttribute(
      "href",
      "/visuals/",
    );
    expect(screen.getByRole("link", { name: /^lab$/i })).toHaveAttribute(
      "href",
      "/lab/",
    );
    expect(screen.getByRole("link", { name: /^rules$/i })).toHaveAttribute(
      "href",
      "/rules/",
    );
    expect(screen.queryByRole("link", { name: /^about$/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /^analysis$/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("link", { name: /^bots$/i })).not.toBeInTheDocument();
  });

  it("routes the lab report through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/lab/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^lab report$/i })).toBeInTheDocument();
  });

  it("routes the visual design reference through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/visuals/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^visual design$/i })).toBeInTheDocument();
  });

  it("routes the rules page through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/rules/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^rules$/i })).toBeInTheDocument();
  });

  it("routes to settings", async () => {
    render(
      <MemoryRouter initialEntries={["/settings"]}>
        <App />
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: /^settings$/i }),
    ).toBeInTheDocument();
    expect(screen.getByText(/^rule$/i)).toBeInTheDocument();
    expect(screen.getByText(/^bot$/i)).toBeInTheDocument();
  });
});
