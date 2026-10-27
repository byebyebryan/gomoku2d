import { useEffect, useRef, type CSSProperties, type MouseEvent } from "react";
import { Link, useLocation, useSearchParams } from "react-router-dom";
import data from "./history.json";
import {
  isHistoryCatalog,
  type HistoryCatalog,
  type HistoryChange,
  type HistoryLayout,
  type HistoryMilestone,
  type HistorySnapshot,
  type HistoryStateChange,
} from "./history_record";
import { ReviewComparison } from "./ReviewComparison";
import "./reviews.css";
import "./history.css";

if (!isHistoryCatalog(data)) throw new Error("Invalid UI history catalog");
const catalog: HistoryCatalog = data;
const root = "/dev/reviews/history/";
const mediaRoot = `${import.meta.env.BASE_URL}history/`;
const releaseMap = new Map(catalog.releases.map((r) => [r.version, r]));
const snapshotMap = new Map(catalog.snapshots.map((s) => [s.id, s]));
const screenMap = new Map(catalog.screens.map((s) => [s.id, s]));
const relationLabel = {
  replaces: "Replaces",
  "replaced-by": "Replaced by",
  "merged-into": "Merged into",
  "split-into": "Split into",
  "split-from": "Split from",
};
const kindLabel = { added: "Added", updated: "Updated", removed: "Removed" };
type Origin = {
  search: string;
  version: string;
  scroll: number;
  focus: string;
  open?: string[];
};

function originFrom(value: unknown): Origin | undefined {
  if (!value || typeof value !== "object") return;
  const v = value as Partial<Origin>;
  if (
    typeof v.search !== "string" ||
    (v.search !== "" && !v.search.startsWith("?")) ||
    v.search.length > 2048 ||
    typeof v.version !== "string" ||
    !catalog.milestones.some((m) => m.version === v.version) ||
    typeof v.scroll !== "number" ||
    !Number.isFinite(v.scroll) ||
    v.scroll < 0 ||
    typeof v.focus !== "string" ||
    !/^history-change-[a-zA-Z0-9_.-]+$/.test(v.focus) ||
    (v.open !== undefined &&
      (!Array.isArray(v.open) ||
        v.open.length > 64 ||
        !v.open.every(
          (id) =>
            typeof id === "string" &&
            /^history-(source|records)-[a-zA-Z0-9_.-]+$/.test(id),
        )))
  )
    return;
  return v as Origin;
}
function versionSearch(
  version: string,
  layout: HistoryLayout,
  extra: Record<string, string> = {},
) {
  return new URLSearchParams({ version, layout, ...extra }).toString();
}
function rememberOrigin(origin: Origin) {
  // Preserve React Router's native entry identity and unrelated user state.
  const current = window.history.state;
  const base = current && typeof current === "object" ? current : {};
  const usr = base.usr && typeof base.usr === "object" ? base.usr : {};
  window.history.replaceState(
    { ...base, usr: { ...usr, historyRestore: origin } },
    "",
  );
}
function plainClick(event: MouseEvent) {
  return (
    event.button === 0 &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.shiftKey
  );
}
function captureNavigationOrigin(origin: Origin) {
  origin.scroll =
    document.querySelector<HTMLElement>(".dev-page")?.scrollTop ?? 0;
  origin.open = Array.from(
    document.querySelectorAll<HTMLDetailsElement>(
      ".history-page details[open][id]",
    ),
  )
    .map((node) => node.id)
    .filter((id) => /^history-(source|records)-/.test(id));
  rememberOrigin(origin);
}

export function ReviewNavigation({ designReview = false }: { designReview?: boolean }) {
  const { pathname } = useLocation();
  return (
    <nav className="history-review-nav" aria-label="UI design views">
      <Link
        to="/dev/reviews/core-screens-062/"
        aria-current={
          pathname.includes("core-screens-062") ? "page" : undefined
        }
      >
        Current screens
      </Link>
      <Link
        to={root}
        aria-current={
          pathname.startsWith(root.replace(/\/$/, "")) ? "page" : undefined
        }
      >
        UI history
      </Link>
      <Link
        to="/dev/reviews/"
        aria-current={
          designReview || pathname.replace(/\/$/, "") === "/dev/reviews" ? "page" : undefined
        }
      >
        Design reviews
      </Link>
    </nav>
  );
}

export function historyRouteTitle(screenId?: string) {
  return screenId
    ? `${screenMap.get(screenId)?.title ?? "Screen"} history`
    : "UI history";
}

function SourceContext({ snapshots }: { snapshots: HistorySnapshot[] }) {
  return (
    <details
      className="dev-disclosure history-source-context"
      id={`history-source-${snapshots.map((s) => s.id).join(".")}`}
    >
      <summary>Capture conditions and source</summary>
      <div>
        {snapshots.map((snapshot) => (
          <section key={snapshot.id}>
            <h3>
              v{snapshot.version} ·{" "}
              {snapshot.captureType === "retrospective"
                ? "Retrospective capture"
                : "Historical capture"}
            </h3>
            <p className="dev-meta">
              Captured {snapshot.capturedAt.slice(0, 10)} ·{" "}
              {snapshot.conditions}
            </p>
            <p className="dev-meta">
              Capture reconstruction and automated checks do not establish
              original-period, human or physical-device acceptance.
            </p>
            <p>
              <a
                href={`https://github.com/byebyebryan/gomoku2d/tree/${releaseMap.get(snapshot.version)?.commit}`}
              >
                Tagged application source
              </a>
              {" · "}
              <a
                href={`${mediaRoot}manifests/${snapshot.manifest.sha256}.json`}
              >
                Capture manifest
              </a>
            </p>
            {Object.entries(snapshot.media).map(([layout, media]) => (
              <p className="dev-meta" key={layout}>
                {layout} · <code>{media.sourcePath}</code>
                <br />
                SHA-256 <code>{media.sha256}</code>
              </p>
            ))}
          </section>
        ))}
      </div>
    </details>
  );
}

function Relations({
  change,
  version,
  layout,
  origin,
  capture = false,
}: {
  change: HistoryChange;
  version: string;
  layout: HistoryLayout;
  origin?: Origin;
  capture?: boolean;
}) {
  if (!change.related?.length) return null;
  return (
    <ul className="history-relations">
      {change.related.map((relation) => (
        <li key={`${relation.relation}-${relation.screen}`}>
          {relationLabel[relation.relation]}{" "}
          <Link
            to={`${root}${relation.screen}/?${versionSearch(version, layout)}`}
            state={origin ? { historyOrigin: origin } : undefined}
            onClick={(event) => {
              if (capture && origin && plainClick(event))
                captureNavigationOrigin(origin);
            }}
          >
            {screenMap.get(relation.screen)?.title}
          </Link>
        </li>
      ))}
    </ul>
  );
}

function ChangeImages({
  change,
  transition,
  milestone,
  layout,
  firstId,
  navigationState,
  overview = false,
}: {
  change: HistoryChange;
  transition: HistoryStateChange;
  milestone: HistoryMilestone;
  layout: HistoryLayout;
  firstId: string;
  navigationState?: unknown;
  overview?: boolean;
}) {
  const snapshots = [transition.before, transition.after].flatMap((id) => {
    const s = id ? snapshotMap.get(id) : undefined;
    return s ? [s] : [];
  });
  const screen = screenMap.get(change.screen)!;
  const selectedMedia = snapshots.flatMap((s) =>
    s.media[layout] ? [s.media[layout]!] : [],
  );
  const missing = snapshots.filter((s) => !s.media[layout]);
  const status =
    change.kind === "added"
      ? `Introduced in v${milestone.version}`
      : change.kind === "removed"
        ? `Removed in v${milestone.version}`
        : !transition.before
          ? `New view in v${milestone.version}`
          : !transition.after
            ? `View removed in v${milestone.version}`
            : undefined;
  const comparison = {
    id: `${change.id}.${transition.id}`,
    title:
      screen.states.find((s) => s.id === transition.id)?.title ?? screen.title,
    conditions: snapshots.map((s) => s.conditions).join(" / "),
    media: selectedMedia,
  };
  return (
    <div className="history-state-comparison" data-state={transition.id}>
      {change.states.length > 1 && <h3>{comparison.title}</h3>}
      {status && <p className="history-lifecycle-note">{status}</p>}
      {!overview && (
        <p className="dev-muted dev-readable">{transition.summary}</p>
      )}
      {missing.map((s) => (
        <p className="history-coverage-note" key={s.id}>
          v{s.version}:{" "}
          {s.unavailable?.[layout] ?? "This layout has not been captured."}
        </p>
      ))}
      {selectedMedia.length > 0 && (
        <ReviewComparison
          comparison={comparison}
          root={mediaRoot}
          firstId={firstId}
          navigationState={navigationState}
          controlsPosition={overview ? "after" : "before"}
          compactCaption={overview}
        />
      )}
      <SourceContext snapshots={snapshots} />
    </div>
  );
}

export function HistoryRoutes({ screenId }: { screenId?: string }) {
  const location = useLocation();
  const [params, setParams] = useSearchParams();
  const screen = screenId ? screenMap.get(screenId) : undefined;
  const layout: HistoryLayout =
    params.get("layout") === "portrait" ? "portrait" : "desktop";
  const changes = screen
    ? catalog.milestones.flatMap((m) => {
        const c = m.changes.find((c) => c.screen === screen.id);
        return c ? [{ milestone: m, change: c }] : [];
      })
    : [];
  const selectedState =
    screen?.states.find((s) => s.id === params.get("state")) ??
    screen?.states[0];
  const stateChanges = selectedState
    ? changes.filter((c) =>
        c.change.states.some((s) => s.id === selectedState.id),
      )
    : changes;
  const milestones = screen
    ? stateChanges.map((c) => c.milestone)
    : catalog.milestones;
  const milestone =
    milestones.find((m) => m.version === params.get("version")) ??
    milestones[milestones.length - 1];
  const selectedChange = screen
    ? stateChanges.find((c) => c.milestone.version === milestone?.version)
        ?.change
    : undefined;
  const selectedTransition = selectedChange?.states.find(
    (s) => s.id === selectedState?.id,
  );
  const rail = useRef<HTMLDetailsElement>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  const sections = useRef(new Map<string, HTMLElement>());
  const previous = useRef({
    path: "",
    version: "",
    state: "",
    pair: "",
    layout: "",
  });
  const stateObject =
    location.state && typeof location.state === "object"
      ? (location.state as Record<string, unknown>)
      : {};
  const origin = originFrom(stateObject.historyOrigin);
  const restored = originFrom(stateObject.historyRestore);
  const navigationState = origin ? { historyOrigin: origin } : undefined;
  const firstId =
    screen && selectedChange && selectedTransition
      ? `${selectedChange.id}.${selectedTransition.id}`
      : milestone?.changes[0]
        ? `${milestone.changes[0].id}.${milestone.changes[0].states[0].id}`
        : "";

  useEffect(() => {
    const query = matchMedia("(min-width: 781px)");
    let disposed = false;
    const fit = () => {
      const node = rail.current;
      const nav = node?.querySelector("nav");
      if (!node || !nav) return;
      nav.style.maxHeight = query.matches
        ? `${Math.max(44, innerHeight - Math.max(20, node.getBoundingClientRect().top) - 20)}px`
        : "";
    };
    const update = () => {
      if (!rail.current) return;
      rail.current.open = query.matches;
      fit();
      if (query.matches) {
        const nav = rail.current.querySelector("nav");
        const selected = nav?.querySelector<HTMLElement>(
          '[aria-current="page"]',
        );
        if (nav && selected)
          nav.scrollTop =
            selected.offsetTop - nav.offsetTop - nav.clientHeight / 2;
      }
    };
    update();
    void document.fonts.ready.then(() => {
      if (!disposed) update();
    });
    const scroll = document.querySelector(".dev-page");
    scroll?.addEventListener("scroll", fit, { passive: true });
    window.addEventListener("resize", fit);
    query.addEventListener("change", update);
    return () => {
      disposed = true;
      scroll?.removeEventListener("scroll", fit);
      window.removeEventListener("resize", fit);
      query.removeEventListener("change", update);
    };
  }, [screenId, milestone?.version]);
  useEffect(() => {
    if (!milestone) return;
    const old = previous.current;
    const current = {
      path: location.pathname,
      version: milestone.version,
      state: selectedState?.id ?? "",
      pair: params.get("pair") ?? "",
      layout,
    };
    previous.current = current;
    if (
      rail.current &&
      !matchMedia("(min-width: 781px)").matches &&
      old.version !== current.version
    )
      rail.current.open = false;
    const restore =
      !screen && restored?.version === milestone.version ? restored : undefined;
    const changed =
      old.path !== current.path ||
      old.version !== current.version ||
      old.state !== current.state ||
      (old.layout === layout && old.pair !== current.pair);
    if (!restore && !changed) return;
    const section = current.pair
      ? sections.current.get(current.pair)
      : undefined;
    const target = restore
      ? document.getElementById(restore.focus)
      : (section ??
        (screen && params.has("version")
          ? document.getElementById("history-inspect")
          : old.path
            ? heading.current
            : null));
    if (!restore && target?.contains(document.activeElement)) return;
    let second = 0;
    const frame = requestAnimationFrame(() => {
      second = requestAnimationFrame(() => {
        if (restore) {
          for (const id of restore.open ?? []) {
            const details = document.getElementById(id);
            if (details instanceof HTMLDetailsElement) details.open = true;
          }
          const scroll = document.querySelector<HTMLElement>(".dev-page");
          if (scroll) scroll.scrollTop = restore.scroll;
          target?.focus({ preventScroll: true });
        } else {
          target?.scrollIntoView({ block: "start" });
          target?.focus({ preventScroll: true });
        }
      });
    });
    return () => {
      cancelAnimationFrame(frame);
      cancelAnimationFrame(second);
    };
  }, [
    location.key,
    location.pathname,
    milestone,
    params,
    screen,
    selectedState?.id,
    restored,
    layout,
  ]);

  function changeLayout(next: HistoryLayout) {
    const search = new URLSearchParams(params);
    search.set("layout", next);
    for (const key of ["pair", "native", "full", "sync"]) search.delete(key);
    setParams(search, { preventScrollReset: true, state: navigationState });
  }
  function layoutControls() {
    return (
      <div
        className="history-layout-controls"
        role="group"
        aria-label="Capture layout"
      >
        {(["desktop", "portrait"] as const).map((value) => (
          <button
            className="dev-tool"
            key={value}
            aria-pressed={layout === value}
            onClick={() => changeLayout(value)}
          >
            {value === "desktop" ? "Desktop" : "Portrait"}
          </button>
        ))}
      </div>
    );
  }
  function overviewSearch(version: string, pair?: string) {
    return versionSearch(version, layout, pair ? { pair } : {});
  }
  function inspectLink(change: HistoryChange, state: string) {
    const focus = `history-change-${change.id}`;
    const returnPoint: Origin = {
      version: milestone!.version,
      search: location.search,
      focus,
      scroll: 0,
    };
    return (
      <Link
        className="dev-tool"
        to={`${root}${change.screen}/?${versionSearch(milestone!.version, layout, { state })}`}
        onClick={(event) => {
          if (!plainClick(event)) return;
          captureNavigationOrigin(returnPoint);
        }}
        state={{ historyOrigin: returnPoint }}
        preventScrollReset
      >
        Inspect {screenMap.get(change.screen)?.title} history
      </Link>
    );
  }
  function renderChange(change: HistoryChange) {
    return (
      <section
        key={change.id}
        className="history-change"
        data-change={change.id}
        id={`history-change-${change.id}`}
        tabIndex={-1}
      >
        <header className="history-change-heading">
          <h2>{screenMap.get(change.screen)?.title}</h2>
          <span>{kindLabel[change.kind]}</span>
        </header>
        <p className="dev-readable">{change.summary}</p>
        <Relations
          change={change}
          version={milestone!.version}
          layout={layout}
          origin={{
            version: milestone!.version,
            search: location.search,
            focus: `history-change-${change.id}`,
            scroll: 0,
          }}
          capture
        />
        {change.states.map((transition) => (
          <section
            key={transition.id}
            data-comparison={`${change.id}.${transition.id}`}
            className="history-view"
            tabIndex={-1}
            ref={(element) => {
              const key = `${change.id}.${transition.id}`;
              if (element) sections.current.set(key, element);
              else sections.current.delete(key);
            }}
          >
            <ChangeImages
              change={change}
              transition={transition}
              milestone={milestone!}
              layout={layout}
              firstId={firstId}
              navigationState={navigationState}
              overview
            />
          </section>
        ))}
        {inspectLink(change, change.states[0].id)}
        {change.sources?.length ? (
          <details
            className="dev-disclosure"
            id={`history-records-${change.id}`}
          >
            <summary>Design and change records</summary>
            <div>
              <ul>
                {change.sources.map((source) => (
                  <li key={source.url}>
                    <a href={source.url}>{source.label}</a>
                  </li>
                ))}
              </ul>
            </div>
          </details>
        ) : null}
      </section>
    );
  }
  function neighbourControls() {
    const index = milestones.findIndex((m) => m.version === milestone?.version);
    return (
      <nav className="history-neighbours" aria-label="Neighbouring milestones">
        {[milestones[index - 1], milestones[index + 1]].map((m, i) =>
          m ? (
            <Link
              key={i}
              className={`dev-tool ${i ? "history-next" : ""}`}
              to={{
                search: versionSearch(
                  m.version,
                  layout,
                  selectedState ? { state: selectedState.id } : {},
                ),
              }}
              preventScrollReset
              state={navigationState}
            >
              {i ? `v${m.version} →` : `← v${m.version}`}
            </Link>
          ) : (
            <span key={i} />
          ),
        )}
      </nav>
    );
  }
  if (screenId && !screen)
    return (
      <section>
        <ReviewNavigation />
        <h1>Screen history not found</h1>
        <p>Choose a screen from the curated history.</p>
        <Link className="dev-tool" to={root}>
          UI history
        </Link>
      </section>
    );
  if (!milestone)
    return (
      <section>
        <ReviewNavigation />
        <h1>UI history</h1>
        <p>The curated history is being prepared.</p>
      </section>
    );

  const trail: {
    snapshot: HistorySnapshot;
    version: string;
    summary: string;
  }[] = [];
  if (screen && selectedState) {
    const seen = new Set<string>();
    for (const entry of stateChanges) {
      const transition = entry.change.states.find(
        (s) => s.id === selectedState.id,
      )!;
      for (const ref of [transition.before, transition.after]) {
        const snapshot = ref ? snapshotMap.get(ref) : undefined;
        if (!snapshot || seen.has(snapshot.id)) continue;
        seen.add(snapshot.id);
        trail.push({
          snapshot,
          version: entry.milestone.version,
          summary: transition.summary,
        });
      }
    }
  }
  const largestWidth = Math.max(
    1,
    ...trail.map((entry) => entry.snapshot.media[layout]?.width ?? 1),
  );
  const backSearch = origin?.search ?? `?${overviewSearch(milestone.version)}`;
  return (
    <article
      className="history-page"
      data-history-view={screen ? "screen" : "milestones"}
    >
      <ReviewNavigation />
      <header className="dev-heading history-heading">
        <h1 ref={heading} tabIndex={-1}>
          {screen ? `${screen.title}, over time` : "UI history"}
        </h1>
        <p className="dev-lead">{screen?.description ?? catalog.summary}</p>
      </header>
      {screen ? (
        <>
          <Link
            className="history-back"
            to={root + backSearch}
            state={origin ? { historyRestore: origin } : undefined}
          >
            ← Back to v{origin?.version ?? milestone.version} changes
          </Link>
          <nav
            className="dev-views history-screen-nav"
            aria-label="Screen histories"
          >
            {catalog.screens.map((s) => (
              <Link
                className="dev-tool"
                key={s.id}
                to={`${root}${s.id}/?${versionSearch(milestone.version, layout)}`}
                state={navigationState}
                aria-current={s.id === screen.id ? "page" : undefined}
              >
                {s.title}
              </Link>
            ))}
          </nav>
          {screen.states.length > 1 && (
            <nav className="dev-views" aria-label={`${screen.title} views`}>
              {screen.states.map((s) => (
                <Link
                  className="dev-tool"
                  key={s.id}
                  to={{
                    search: versionSearch(milestone.version, layout, {
                      state: s.id,
                    }),
                  }}
                  preventScrollReset
                  state={location.state}
                  aria-current={s.id === selectedState?.id ? "page" : undefined}
                >
                  {s.title}
                </Link>
              ))}
            </nav>
          )}
          <div className="history-toolbar">
            {layoutControls()}
            <a
              className="dev-tool"
              href="#history-inspect"
              onClick={(event) => {
                event.preventDefault();
                const node = document.getElementById("history-inspect");
                node?.scrollIntoView({ block: "start" });
                node?.focus({ preventScroll: true });
              }}
            >
              Inspect selected change
            </a>
          </div>
          <div
            className={`history-trail history-trail-${layout}`}
            style={
              { "--history-source-width": `${largestWidth}px` } as CSSProperties
            }
          >
            {trail.reverse().map((entry) => {
              const media = entry.snapshot.media[layout];
              return (
                <section
                  className="history-trail-entry"
                  key={entry.snapshot.id}
                  data-snapshot={entry.snapshot.id}
                >
                  <h2>v{entry.snapshot.version}</h2>
                  <p className="dev-meta">
                    {entry.snapshot.captureType === "retrospective"
                      ? "Retrospective capture"
                      : "Historical capture"}
                  </p>
                  {media ? (
                    <figure>
                      <img
                        src={mediaRoot + media.file}
                        width={media.width}
                        height={media.height}
                        style={
                          {
                            "--history-image-ratio": media.width / largestWidth,
                          } as CSSProperties
                        }
                        alt={media.alt}
                        loading="lazy"
                      />
                      <figcaption>{entry.summary}</figcaption>
                    </figure>
                  ) : (
                    <p className="history-coverage-note">
                      {entry.snapshot.unavailable?.[layout]}
                    </p>
                  )}
                  <Link
                    className="dev-tool"
                    to={{
                      search: versionSearch(entry.version, layout, {
                        state: selectedState!.id,
                      }),
                    }}
                    preventScrollReset
                    state={location.state}
                  >
                    Inspect change
                  </Link>
                </section>
              );
            })}
          </div>
          {changes
            .filter((entry) => entry.change.kind === "removed")
            .map((entry) => (
              <section
                className="history-retired"
                key={entry.milestone.version}
              >
                <h2>Removed in v{entry.milestone.version}</h2>
                <p>{entry.change.summary}</p>
                <Relations
                  change={entry.change}
                  version={entry.milestone.version}
                  layout={layout}
                  origin={origin}
                />
              </section>
            ))}
          {selectedChange && selectedTransition && (
            <section
              id="history-inspect"
              tabIndex={-1}
              className="history-inspect"
            >
              <header className="history-change-heading">
                <h2>
                  {screen.title} · v{milestone.version}
                </h2>
                <span>{kindLabel[selectedChange.kind]}</span>
              </header>
              <p className="dev-readable">{selectedChange.summary}</p>
              {neighbourControls()}
              <Relations
                change={selectedChange}
                version={milestone.version}
                layout={layout}
                origin={origin}
              />
              <section
                data-comparison={firstId}
                tabIndex={-1}
                ref={(element) => {
                  if (element) sections.current.set(firstId, element);
                  else sections.current.delete(firstId);
                }}
              >
                <ChangeImages
                  change={selectedChange}
                  transition={selectedTransition}
                  milestone={milestone}
                  layout={layout}
                  firstId={firstId}
                  navigationState={navigationState}
                />
              </section>
              <Link
                className="history-back"
                to={root + backSearch}
                state={origin ? { historyRestore: origin } : undefined}
              >
                ← Back to milestone changes
              </Link>
            </section>
          )}
        </>
      ) : (
        <div className="history-workbench">
          <details className="history-milestone-rail" ref={rail}>
            <summary>Milestones · v{milestone.version}</summary>
            <nav aria-label="UI milestones">
              {Array.from(
                new Set(
                  catalog.milestones.map((m) =>
                    m.version.split(".").slice(0, 2).join("."),
                  ),
                ),
              ).reverse().map((phase) => (
                <section key={phase}>
                  <h2>{phase}</h2>
                  {catalog.milestones
                    .filter((m) => m.version.startsWith(phase + "."))
                    .reverse()
                    .map((m) => (
                      <Link
                        key={m.version}
                        className="history-milestone-link"
                        to={{ search: overviewSearch(m.version) }}
                        preventScrollReset
                        aria-current={
                          m.version === milestone.version ? "page" : undefined
                        }
                      >
                        <strong>v{m.version}</strong>
                        <span>{m.title}</span>
                      </Link>
                    ))}
                </section>
              ))}
            </nav>
          </details>
          <div className="history-milestone-content">
            <header className="history-milestone-heading">
              <p className="dev-eyebrow">
                v{milestone.version} · {releaseMap.get(milestone.version)?.date}
              </p>
              <h2
                tabIndex={-1}
                ref={(node) => {
                  if (node) heading.current = node as HTMLHeadingElement;
                }}
              >
                {milestone.title}
              </h2>
              <p className="dev-readable">{milestone.summary}</p>
              <div className="history-toolbar">
                <div className="history-release-navigation">
                  {neighbourControls()}
                </div>
                {layoutControls()}
              </div>
            </header>
            <nav
              className="history-change-jumps"
              aria-label="Changes in this milestone"
            >
              {milestone.changes.map((change) => (
                <Link
                  key={change.id}
                  to={{
                    search: overviewSearch(
                      milestone.version,
                      `${change.id}.${change.states[0].id}`,
                    ),
                  }}
                  preventScrollReset
                  aria-current={
                    params.get("pair")?.startsWith(change.id + ".")
                      ? "location"
                      : undefined
                  }
                >
                  {screenMap.get(change.screen)?.title}
                  <span>{kindLabel[change.kind]}</span>
                </Link>
              ))}
            </nav>
            {milestone.changes.map(renderChange)}
          </div>
        </div>
      )}
    </article>
  );
}
