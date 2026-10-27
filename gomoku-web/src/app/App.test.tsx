import {
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MemoryRouter, useNavigate } from "react-router-dom";

import { App } from "./App";
import { CloudAuthController, CloudSessionController } from "../cloud/CloudSessionController";

vi.mock("../cloud/CloudSessionController", () => ({
  CloudAuthController: vi.fn(() => null),
  CloudSessionController: vi.fn(() => null),
}));

vi.mock("../routes/SharedReplayRoute", () => ({
  SharedReplayRoute: () => <main><h1>Shared replay</h1></main>,
}));

vi.mock("../routes/PublicPlayerRoute", () => ({
  PublicPlayerRoute: () => <main><h1>Public player</h1></main>,
}));

vi.mock("../routes/ProfileRoute", () => ({
  ProfileRoute: () => <main><h1>Private profile</h1></main>,
}));

vi.mock("../routes/ReplayRoute", () => ({
  ReplayRoute: () => <main><h1>Private replay</h1></main>,
}));

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

vi.mock("../routes/LabReportRoute", () => ({
  LabReportRoute: () => (
    <section>
      <h1>Lab Report</h1>
    </section>
  ),
}));

vi.mock("../routes/AssetPreviewRoute", () => ({
  AssetPreviewRoute: () => (
    <section>
      <h1>Visuals</h1>
    </section>
  ),
}));

vi.mock("../routes/RulesRoute", () => ({
  RulesRoute: () => (
    <main>
      <h1>Rules</h1>
    </main>
  ),
}));

vi.mock("../routes/GuideRoute", () => ({
  GuideRoute: () => (
    <main>
      <h1>Guide</h1>
    </main>
  ),
}));

vi.mock("../routes/PolicyRoute", () => ({
  PolicyRoute: ({ kind }: { kind: "privacy" | "terms" }) => (
    <main>
      <h1>{kind === "privacy" ? "Privacy" : "Terms"}</h1>
    </main>
  ),
}));

describe("App", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    HTMLElement.prototype.scrollTo = vi.fn();
  });

  afterEach(() => {
    cleanup();
  });

  it.each([
    ["/dev/", "Inside Gomoku2D"], ["/DEV/", "Inside Gomoku2D"],
    ["/%64ev/", "Inside Gomoku2D"], ["/dev/process/", "Design reviews"],
    ["/dev/unknown/", "Reference not found"], ["/visuals/", "Visuals"], ["/lab/", "Lab Report"],
  ])("renders public developer entry %s without starting auth or private controllers",async(path,heading)=>{
    render(<MemoryRouter initialEntries={[path]}><App/></MemoryRouter>);
    // Allow the cold lazy module transform to finish under parallel test load.
    expect(await screen.findByRole("heading", { name: heading }, { timeout: 3_000 })).toBeInTheDocument();
    expect(CloudAuthController).not.toHaveBeenCalled();
    expect(CloudSessionController).not.toHaveBeenCalled();
  });

  it.each([
    ["/replay/shared", "Shared replay"],
    ["/replay/shared/", "Shared replay"],
    ["/Replay/Shared/", "Shared replay"],
    ["/replay/shared//", "Shared replay"],
    ["/replay/%73hared/", "Shared replay"],
    ["/%72eplay/shared", "Shared replay"],
    ["/player", "Public player"],
    ["/player/", "Public player"],
    ["/PLAYER/", "Public player"],
    ["/player///", "Public player"],
    ["/%70layer/", "Public player"],
  ])("renders %s without starting the private cloud session", async (path, heading) => {
    render(<MemoryRouter initialEntries={[path]}><App /></MemoryRouter>);

    expect(await screen.findByRole("heading", { name: heading })).toBeInTheDocument();
    expect(CloudSessionController).not.toHaveBeenCalled();
  });

  it.each([
    ["/profile/", "Private profile"],
    ["/replay/private-match/", "Private replay"],
    ["/replay/shared-copy/", "Private replay"],
    ["/player/extra", "gomoku2d"],
    ["/replay/shared/extra", "gomoku2d"],
  ])("keeps the private cloud session enabled on %s", async (path, heading) => {
    render(<MemoryRouter initialEntries={[path]}><App /></MemoryRouter>);

    expect(await screen.findByRole("heading", { name: new RegExp(heading, "i") })).toBeInTheDocument();
    expect(CloudSessionController).toHaveBeenCalled();
  });

  it("starts the private cloud session when leaving a shared replay", async () => {
    function OpenProfile() {
      const navigate = useNavigate();
      return <button onClick={() => navigate("/profile/")}>Open private profile</button>;
    }
    render(<MemoryRouter initialEntries={["/Replay/Shared//"]}><OpenProfile /><App /></MemoryRouter>);

    expect(await screen.findByRole("heading", { name: "Shared replay" })).toBeInTheDocument();
    expect(CloudSessionController).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Open private profile" }));
    expect(await screen.findByRole("heading", { name: "Private profile" })).toBeInTheDocument();
    expect(CloudSessionController).toHaveBeenCalled();
  });

  it("starts on home and routes into a local bot match", async () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole("heading", { name: /gomoku2d/i }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("link", { name: /play/i }));

    expect(
      await screen.findByRole("heading", { name: /local match/i }),
    ).toBeInTheDocument();
    expect(screen.getByText(/black to move/i)).toBeInTheDocument();
  });

  it("surfaces developer links on the home screen", async () => {
    render(
      <MemoryRouter initialEntries={["/"]}>
        <App />
      </MemoryRouter>,
    );

    await screen.findByRole("heading", { name: /gomoku2d/i });

    expect(screen.getByRole("link", { name: /^visual design$/i })).toHaveAttribute(
      "href",
      "/visuals/",
    );
    expect(screen.getByRole("link", { name: /^ui design$/i })).toHaveAttribute(
      "href",
      "/dev/reviews/core-screens-062/",
    );
    expect(screen.getByRole("link", { name: /^bot lab$/i })).toHaveAttribute(
      "href",
      "/lab/",
    );
    expect(screen.getByRole("link", { name: /^rules$/i })).toHaveAttribute(
      "href",
      "/rules/",
    );
    expect(screen.getByRole("link", { name: /^guide$/i })).toHaveAttribute(
      "href",
      "/guide/",
    );
    expect(screen.getByRole("link", { name: /^source$/i })).toHaveAttribute(
      "href",
      "https://github.com/byebyebryan/gomoku2d",
    );
    expect(screen.getByRole("link", { name: /^privacy$/i })).toHaveAttribute(
      "href",
      "/privacy/",
    );
    expect(screen.getByRole("link", { name: /^terms$/i })).toHaveAttribute(
      "href",
      "/terms/",
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

  it("routes the visuals reference through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/visuals/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^visuals$/i })).toBeInTheDocument();
  });

  it("routes the rules page through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/rules/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^rules$/i })).toBeInTheDocument();
  });

  it("routes the guide page through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/guide/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^guide$/i })).toBeInTheDocument();
  });

  it("routes the privacy page through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/privacy/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^privacy$/i })).toBeInTheDocument();
  });

  it("routes the terms page through the app", async () => {
    render(
      <MemoryRouter initialEntries={["/terms/"]}>
        <App />
      </MemoryRouter>,
    );

    expect(await screen.findByRole("heading", { name: /^terms$/i })).toBeInTheDocument();
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
