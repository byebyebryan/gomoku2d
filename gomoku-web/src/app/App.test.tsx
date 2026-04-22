import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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

describe("App", () => {
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
});
