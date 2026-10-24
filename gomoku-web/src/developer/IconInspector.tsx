import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router-dom";
import { Icon } from "../ui/Icon";
import { canonicalIcons, selectedIcons, type CanonicalIcon } from "./icon_catalog";

function Glyph({ icon }: { icon: CanonicalIcon }) {
  return <span aria-hidden="true" dangerouslySetInnerHTML={{ __html: icon.svg }} />;
}
export function IconInspector() {
  const [params, setParams] = useSearchParams(),
    selected = selectedIcons(params),
    search = params.get("q") ?? "",
    grid = params.get("grid") !== "off",
    light = params.get("surface") === "light";
  const [announcement, setAnnouncement] = useState("");
  const catalog = useRef<HTMLHeadingElement>(null),
    heading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    setAnnouncement("");
  }, [params]);
  function update(values: Record<string, string | null>, replace = false) {
    const next = new URLSearchParams(params);
    next.set("view", "icons");
    for (const [key, value] of Object.entries(values)) {
      if (value === null) next.delete(key);
      else next.set(key, value);
    }
    setParams(next, { replace, preventScrollReset: true });
  }
  function choose(names: string[]) {
    update({
      icon: names.length === 1 ? names[0] : null,
      compare: names.length > 1 ? names.join(",") : null,
    });
  }
  function compare(name: string) {
    if (!selected.includes(name) && selected.length === 4) {
      setAnnouncement("Compare up to four icons. Remove one before adding another.");
      return;
    }
    const next = selected.includes(name) ? selected.filter((n) => n !== name) : [...selected, name];
    choose(next.length ? next : ["share"]);
  }
  const matching = canonicalIcons.filter((icon) =>
    `${icon.name} ${icon.label} ${icon.category}`.toLowerCase().includes(search.toLowerCase()),
  );
  return (
    <>
      <div className="dev-tools">
        <label>
          Find an icon
          <input
            type="search"
            value={search}
            placeholder="Name or category"
            onChange={(event) => update({ q: event.target.value || null }, true)}
          />
        </label>
        <span className="dev-meta">{canonicalIcons.length} source icons · 24 × 24</span>
      </div>
      <section
        className={`asset-inspector ${light ? "asset-light" : ""} ${grid ? "" : "asset-grid-off"}`}
        aria-label="Icon inspector"
      >
        <div className="asset-inspector-head">
          <div>
            <h2 ref={heading} tabIndex={-1}>
              Pixel inspection
            </h2>
            <p className="dev-meta">Native samples beside the enlarged source grid.</p>
          </div>
          <div className="dev-tools">
            <button
              className="dev-tool"
              aria-pressed={grid}
              onClick={() => update({ grid: grid ? "off" : null })}
            >
              Grid {grid ? "on" : "off"}
            </button>
            <button
              className="dev-tool"
              aria-pressed={light}
              onClick={() => update({ surface: light ? null : "light" })}
            >
              {light ? "Light" : "Dark"} surface
            </button>
            <button
              className="dev-tool"
              onClick={() => {
                catalog.current?.focus();
                catalog.current?.scrollIntoView({ block: "start" });
              }}
            >
              Browse all
            </button>
          </div>
        </div>
        <div className="asset-comparison" data-count={selected.length}>
          {selected.map((name) => {
            const icon = canonicalIcons.find((i) => i.name === name)!;
            return (
              <article className="asset-icon-inspection" data-inspect={name} key={name}>
                <h3>
                  {icon.label}
                  {selected.length > 1 && (
                    <button
                      className="dev-tool asset-remove"
                      aria-label={`Remove ${icon.label} from comparison`}
                      onClick={() => choose(selected.filter((n) => n !== name))}
                    >
                      <Icon name="minus" />
                    </button>
                  )}
                </h3>
                <div className="asset-inspection-surface">
                  <div className="asset-native-samples">
                    <div>
                      <small>1×</small>
                      <div className="asset-native-one">
                        <Glyph icon={icon} />
                      </div>
                    </div>
                    <div>
                      <small>2×</small>
                      <div className="asset-native-two">
                        <Glyph icon={icon} />
                      </div>
                    </div>
                  </div>
                  <div
                    className="asset-pixel-grid"
                    role="img"
                    aria-label={`${icon.label}, 24 by 24 pixel grid`}
                  >
                    <Glyph icon={icon} />
                  </div>
                </div>
                <p className="dev-meta asset-metrics">
                  Margins L / T / R / B: {icon.margins.join(" / ")}
                  <br />
                  {icon.pixels} filled pixels ·{" "}
                  <a href={`${import.meta.env.BASE_URL}assets/icons/${name}.svg`}>Source SVG</a>
                </p>
                <details className="dev-disclosure">
                  <summary>Source identity</summary>
                  <p className="dev-meta">
                    {"row" in icon
                      ? `Original sheet, row ${icon.row}, column ${icon.col}. `
                      : "Standalone addition. "}
                    {icon.note}
                  </p>
                </details>
              </article>
            );
          })}
          {selected.length === 1 && (
            <aside className="asset-inspector-note">
              <h3>One grid. A consistent family.</h3>
              <p>
                Inspect actual source pixels at 1× and 2×. The larger view makes contours, line
                weight and alignment easy to compare.
              </p>
              <p>Gold axes mark the centre. Existing margins are shown as facts.</p>
              <button className="dev-tool" onClick={() => choose(["share", "save", "publish"])}>
                Compare sharing actions
              </button>
            </aside>
          )}
        </div>
      </section>
      <section>
        <h2 ref={catalog} tabIndex={-1}>
          The complete icon family
        </h2>
        <p className="dev-meta">
          Inspect one or compare up to four. Catalog samples are native 24px.
        </p>
        <div className="asset-icon-catalog">
          {matching.map((icon) => (
            <article key={icon.name} className={selected.includes(icon.name) ? "selected" : ""}>
              <button
                className="asset-catalog-open"
                onClick={() => {
                  choose([icon.name]);
                  heading.current?.focus();
                  heading.current?.scrollIntoView({ block: "start" });
                }}
              >
                <Glyph icon={icon} />
                <span>{icon.label}</span>
              </button>
              <button
                className="asset-compare-add"
                aria-label={`Compare ${icon.label}`}
                aria-pressed={selected.includes(icon.name)}
                onClick={() => compare(icon.name)}
              >
                Compare
              </button>
            </article>
          ))}
        </div>
        {!matching.length && <p role="status">No matching icons.</p>}
      </section>
      <p role="status" className="asset-announcement">
        {announcement}
      </p>
    </>
  );
}
