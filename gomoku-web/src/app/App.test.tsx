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

    expect(screen.getByRole("link", { name: /^assets$/i })).toHaveAttribute(
      "href",
      "/assets/",
    );
    expect(screen.getByRole("link", { name: /^bots$/i })).toHaveAttribute(
      "href",
      "/bot-report/",
    );
    expect(screen.getByRole("link", { name: /^analysis$/i })).toHaveAttribute(
      "href",
      "/analysis-report/",
    );
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
