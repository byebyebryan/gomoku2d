import { Component, Suspense, lazy, useEffect, useRef, type ReactNode } from "react";
import { Link, NavLink, Outlet, Route, Routes, useLocation } from "react-router-dom";
import { Icon } from "../ui/Icon";
import { useDocumentTitle } from "../app/useDocumentTitle";
import { PROJECT_SOURCE_URL } from "../app/links";
import { DeveloperOverview, DeveloperProcess } from "./DeveloperPages";
import "./developer.css";

const Assets = lazy(async () => ({
  default: (await import("../routes/AssetPreviewRoute")).AssetPreviewRoute,
}));
const Lab = lazy(async () => ({
  default: (await import("../routes/LabReportRoute")).LabReportRoute,
}));
const Reviews = lazy(async () => ({ default: (await import("./ReviewRoutes")).ReviewRoutes }));
const sections = [
  ["/dev/", "Overview"],
  ["/visuals/", "Design & assets"],
  ["/lab/", "Lab"],
  ["/dev/reviews/", "Reviews"],
  ["/dev/process/", "Process"],
];

class DeveloperRecovery extends Component<{ children: ReactNode }, { failed: boolean }> {
  state = { failed: false };
  static getDerivedStateFromError() {
    return { failed: true };
  }
  render() {
    return this.state.failed ? (
      <section>
        <h1>Page unavailable</h1>
        <p>Reload to try again.</p>
        <button className="uiAction uiActionPrimary" onClick={() => window.location.reload()}>
          Reload
        </button>{" "}
        <a className="uiAction uiActionNeutral" href={import.meta.env.BASE_URL + "dev/"}>
          Developer overview
        </a>
      </section>
    ) : (
      this.props.children
    );
  }
}
function DeveloperFrame() {
  const location = useLocation(),
    scroll = useRef<HTMLDivElement>(null),
    content = useRef<HTMLElement>(null),
    previous = useRef(location.pathname);
  useEffect(() => {
    if (previous.current !== location.pathname) {
      scroll.current?.scrollTo(0, 0);
      content.current?.focus();
    }
    previous.current = location.pathname;
  }, [location.pathname]);
  return (
    <div className="dev-page" ref={scroll} tabIndex={-1}>
      <a
        className="dev-skip"
        href="#developer-content"
        onClick={(event) => {
          event.preventDefault();
          content.current?.focus();
          content.current?.scrollIntoView();
        }}
      >
        Skip to content
      </a>
      <div className="dev-shell">
        <header className="dev-header">
          <Link className="dev-brand" to="/dev/">
            GOMOKU2D <span>/ DEVELOPER</span>
          </Link>
          <nav className="dev-project-links" aria-label="Project links">
            <Link className="uiAction uiActionPrimary" to="/">
              <Icon name="play" />
              Play
            </Link>
            <a className="uiAction uiActionNeutral" href={PROJECT_SOURCE_URL}>
              Source
            </a>
          </nav>
        </header>
        <nav className="dev-nav" aria-label="Developer sections">
          {sections.map(([to, label]) => (
            <NavLink key={to} to={to} end={to === "/dev/"}>
              {label}
            </NavLink>
          ))}
        </nav>
        <main id="developer-content" tabIndex={-1} ref={content}>
          <DeveloperRecovery key={location.pathname}>
            <Suspense fallback={<p aria-busy="true">Loading reference…</p>}>
              <Outlet />
            </Suspense>
          </DeveloperRecovery>
        </main>
        <footer className="dev-footer">
          <span>Gomoku2D · the work behind the game</span>
          <span>
            <Link to="/dev/process/">Process & evidence</Link> ·{" "}
            <a href={`${import.meta.env.BASE_URL}assets/fonts/LICENSE.txt`}>Font license</a>
          </span>
        </footer>
      </div>
    </div>
  );
}
export function DeveloperNotFound() {
  useDocumentTitle("Reference not found");
  return (
    <section>
      <h1>Reference not found</h1>
      <p>Choose a developer section or return to the overview.</p>
      <Link className="uiAction uiActionNeutral" to="/dev/">
        Developer overview
      </Link>
    </section>
  );
}
export function DeveloperApp() {
  return (
    <Routes>
      <Route element={<DeveloperFrame />}>
        <Route path="/dev" element={<DeveloperOverview />} />
        <Route path="/dev/process" element={<DeveloperProcess />} />
        <Route path="/dev/reviews/*" element={<Reviews />} />
        <Route path="/lab" element={<Lab />} />
        <Route path="/visuals" element={<Assets />} />
        <Route path="*" element={<DeveloperNotFound />} />
      </Route>
    </Routes>
  );
}
