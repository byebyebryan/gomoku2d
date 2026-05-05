import {
  cleanup,
  fireEvent,
  render,
  screen,
  within,
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

  it("surfaces asset and bot links on the home screen", () => {
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
  });

  it("groups the version above a single footer link row", () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <App />
      </MemoryRouter>,
    );

    const footerLinks = screen.getByRole("navigation", {
      name: /footer links/i,
    });
    const version = screen.getByText(/^v\d+\.\d+\.\d+$/i);

    expect(
      version.compareDocumentPosition(footerLinks)
        & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(version.className).toContain("version");
    expect(within(footerLinks).getByRole("link", { name: /^assets$/i })).toBeInTheDocument();
    expect(within(footerLinks).getByRole("link", { name: /^bots$/i })).toBeInTheDocument();
    expect(within(footerLinks).getByRole("link", { name: /^privacy$/i })).toBeInTheDocument();
    expect(within(footerLinks).getByRole("link", { name: /^terms$/i })).toBeInTheDocument();
    expect(within(footerLinks).getAllByText("/")).toHaveLength(3);
    expect(footerLinks).not.toHaveTextContent("·");
  });
});
