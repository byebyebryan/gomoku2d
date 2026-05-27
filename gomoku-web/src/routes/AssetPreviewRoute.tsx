import {
  useEffect,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";

import {
  loadAssetManifest,
  loadIconManifest,
  type AssetManifest,
  type AssetGuideFontRole,
  type AssetSpriteAnimation,
  type AssetSpriteStaticPose,
  type AssetSpriteZLayer,
  type IconManifest,
} from "../reports/asset_manifest";

import styles from "./AssetPreviewRoute.module.css";

const baseUrl = import.meta.env.BASE_URL;

type LoadState =
  | { status: "loading" }
  | { status: "loaded"; manifest: AssetManifest; icons: IconManifest }
  | { status: "error"; message: string };

type AssetTab = "guide" | "sprites" | "icons";

const ASSET_TABS: Array<{ id: AssetTab; label: string }> = [
  { id: "guide", label: "Style" },
  { id: "sprites", label: "Sprites" },
  { id: "icons", label: "Icons" },
];

export function AssetPreviewRoute() {
  const [state, setState] = useState<LoadState>({ status: "loading" });
  const [tab, setTab] = useState<AssetTab>("guide");

  useEffect(() => {
    document.title = "Gomoku2D Visuals";
  }, []);

  useEffect(() => {
    let cancelled = false;
    loadAssetManifest()
      .then(async (manifest) => {
        const icons = await loadIconManifest(manifest.icons.manifest);
        if (!cancelled) {
          setState({ status: "loaded", manifest, icons });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  let content: ReactNode;
  if (state.status === "error") {
    content = <StatePanel message={state.message} />;
  } else if (state.status !== "loaded") {
    content = <StatePanel message="Loading assets…" />;
  } else {
    content = <AssetTabContent tab={tab} manifest={state.manifest} icons={state.icons} />;
  }

  return (
    <main className={styles.page}>
      <TintFilters />
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">Gomoku2D source</p>
              <h1 className={styles.title}>Visuals</h1>
              {state.status === "loaded" ? (
                <p className={styles.summary}>{state.manifest.summary}</p>
              ) : null}
            </div>
            <nav className={styles.links} aria-label="Visuals links">
              <a className="uiAction uiActionNeutral" href={baseUrl}>
                <span className="uiActionLabel">Home</span>
              </a>
              <a className="uiAction uiActionNeutral" href={`${baseUrl}lab/`}>
                <span className="uiActionLabel">Lab</span>
              </a>
            </nav>
          </div>
          <div className={styles.tabs} aria-label="Visuals sections" role="tablist">
            {ASSET_TABS.map((option) => (
              <button
                key={option.id}
                type="button"
                id={`visuals-tab-${option.id}`}
                aria-controls={`visuals-panel-${option.id}`}
                aria-selected={tab === option.id}
                className={tab === option.id ? styles.activeTab : undefined}
                onClick={() => setTab(option.id)}
                role="tab"
              >
                {option.label}
              </button>
            ))}
          </div>
        </header>
        {content}
      </div>
    </main>
  );
}

function AssetTabContent({
  tab,
  manifest,
  icons,
}: {
  tab: AssetTab;
  manifest: AssetManifest;
  icons: IconManifest;
}) {
  let panel: ReactNode;
  if (tab === "guide") {
    panel = <GuidePanel manifest={manifest} />;
  } else if (tab === "sprites") {
    panel = <SpritesPanel manifest={manifest} />;
  } else if (tab === "icons") {
    panel = <IconsPanel manifest={manifest} icons={icons} />;
  } else {
    panel = <SpritesPanel manifest={manifest} />;
  }

  return (
    <div aria-labelledby={`visuals-tab-${tab}`} id={`visuals-panel-${tab}`} role="tabpanel">
      {panel}
    </div>
  );
}

function GuidePanel({ manifest }: { manifest: AssetManifest }) {
  return (
    <>
      <section className={`${styles.panel} ${styles.guideIntro}`}>
        <div>
          <h2>Visual Language</h2>
          <p className={styles.note}>
            A small reference for how Gomoku2D should look and feel: dark cabinet shell,
            tactile controls, semantic colors, and board-space cues that stay below the
            action.
          </p>
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Palette</h2>
        <div className={styles.paletteGrid}>
          {manifest.guide.palette.map((group) => (
            <article key={group.title} className={styles.guideCard}>
              <h3>{group.title}</h3>
              <div className={styles.swatchList}>
                {group.tokens.map((token) => (
                  <div key={token.name} className={styles.swatchRow}>
                    <span
                      className={styles.swatch}
                      style={{ "--swatch-color": token.value } as CSSProperties}
                    />
                    <span className={styles.swatchCopy}>
                      <span className={styles.swatchName}>{token.label}</span>
                      <span className={styles.meta}>
                        {token.name} · {token.value}
                      </span>
                      <span className={styles.note}>{token.role}</span>
                    </span>
                  </div>
                ))}
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Buttons</h2>
        <p className={styles.note}>
          Buttons are chunky, high-contrast, and role-coded. These examples use the live
          app button classes rather than a separate showcase style.
        </p>
        <div className={styles.buttonGrid}>
          <button className="uiAction uiActionPrimary" type="button">Primary</button>
          <button className="uiAction uiActionSecondary" type="button">Secondary</button>
          <button className="uiAction uiActionNeutral" type="button">Neutral</button>
          <button className="uiAction uiActionDanger" type="button">Danger</button>
          <button className="uiAction uiActionAccent" type="button">Accent</button>
          <button className="uiAction uiActionNeutral uiActionIconOnly" type="button" aria-label="Icon-only sample">
            <span className={styles.iconGlyph} aria-hidden="true">&gt;</span>
          </button>
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Typography</h2>
        <p className={styles.note}>
          PixelOperator is an external runtime font family. The guide shows how the app uses
          it; sprites and icons remain the project-authored source assets.
        </p>
        <div className={styles.fontRoleGrid}>
          {manifest.guide.font_roles.map((font) => (
            <article key={font.file} className={styles.guideCard}>
              <h3>{font.role}</h3>
              <p className={fontSampleClass(font)}>{font.sample}</p>
              <p className={styles.meta}>{basename(font.file)}</p>
              <p className={styles.note}>{font.note}</p>
            </article>
          ))}
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Principles</h2>
        <div className={styles.principleGrid}>
          {manifest.guide.principles.map((principle) => (
            <article key={principle.title} className={styles.guideCard}>
              <h3>{principle.title}</h3>
              <p className={styles.note}>{principle.description}</p>
            </article>
          ))}
        </div>
      </section>
    </>
  );
}

function SpritesPanel({ manifest }: { manifest: AssetManifest }) {
  const [paused, setPaused] = useState(false);
  const [scale, setScale] = useState(5);
  const frame = useAnimationFrame(paused);

  return (
    <>
      <section className={styles.panel}>
        <div>
          <h2>Sprites</h2>
          <p className={styles.note}>
            Board-space spritesheets, static poses, animation loops, and z-order cases. Frame
            ranges follow the runtime board constants.
          </p>
        </div>
        <div className={styles.controls}>
          <button className={styles.controlButton} type="button" onClick={() => setPaused((value) => !value)}>
            {paused ? "Play" : "Pause"}
          </button>
          <label className={styles.controlLabel}>
            Scale
            <select
              className={styles.scaleSelect}
              value={scale}
              onChange={(event) => setScale(Number(event.currentTarget.value))}
            >
              {[4, 5, 6, 8].map((value) => (
                <option key={value} value={value}>
                  {value}x
                </option>
              ))}
            </select>
          </label>
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Source Sheets</h2>
        <div className={styles.sheetGrid}>
          {manifest.sprites.sheets.map((sheet) => (
            <article key={sheet.file} className={styles.sheetCard}>
              <div>
                <h3>{basename(sheet.file)}</h3>
                <p className={styles.meta}>
                  {sheet.cols} cols x {sheet.rows} rows · {sheet.label}
                </p>
              </div>
              <img
                alt={`${basename(sheet.file)} source sheet`}
                className={styles.sheetImage}
                src={assetUrl(sheet.file)}
                style={{ "--sheet-width": `${sheet.cols * manifest.sprites.frame_size * 4}px` } as CSSProperties}
              />
            </article>
          ))}
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Static Poses</h2>
        <div className={styles.assetGrid} style={{ "--preview-scale": scale } as CSSProperties}>
          {manifest.sprites.static_poses.map((pose) => (
            <SpriteCard key={pose.name} frameSize={manifest.sprites.frame_size} item={pose} />
          ))}
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Animation Loops</h2>
        <div className={styles.assetGrid} style={{ "--preview-scale": scale } as CSSProperties}>
          {manifest.sprites.animations.map((animation) => (
            <SpriteCard
              key={animation.name}
              frame={frame}
              frameSize={manifest.sprites.frame_size}
              item={animation}
            />
          ))}
        </div>
      </section>

      <section className={styles.panel}>
        <h2>Z-Order Cases</h2>
        <div className={styles.zCaseGrid}>
          {manifest.sprites.z_cases.map((zCase) => (
            <article key={zCase.name} className={styles.zCaseCard}>
              <div className={styles.boardCell} aria-label={`${zCase.title} z-order preview`}>
                {zCase.layers.map((layer) => (
                  <SpriteLayer
                    key={`${zCase.name}-${layer.name}`}
                    className={zClassName(layer.z)}
                    frame={frame}
                    frameSize={manifest.sprites.frame_size}
                    item={layer}
                  />
                ))}
                {zCase.sequence ? <div className={styles.sequenceNumber}>{zCase.sequence}</div> : null}
              </div>
              <div>
                <h3>{zCase.title}</h3>
                <p className={styles.note}>{zCase.note}</p>
              </div>
              <ol className={styles.zCaseNotes}>
                {zCase.layers.map((layer) => (
                  <li key={`${zCase.name}-note-${layer.name}`}>{layer.name}</li>
                ))}
              </ol>
            </article>
          ))}
        </div>
        <ol className={styles.layerList}>
          <li>next-move hover</li>
          <li>sequence number</li>
          <li>stone</li>
          <li>pointer</li>
          <li>marker / caution surface</li>
          <li>highlight surface</li>
          <li>board / grid</li>
        </ol>
      </section>
    </>
  );
}

function SpriteCard({
  frame,
  frameSize,
  item,
}: {
  frame?: number;
  frameSize: number;
  item: AssetSpriteAnimation | AssetSpriteStaticPose;
}) {
  const range =
    "start" in item ? `frames ${item.start}-${item.end} · ${item.fps} fps` : `frame ${item.frame}`;
  return (
    <article className={styles.assetCard}>
      <div className={styles.stage}>
        <SpriteLayer frame={frame} frameSize={frameSize} item={item} />
      </div>
      <div>
        <h3>{item.name}</h3>
        <p className={styles.meta}>
          {item.group} · {range}
        </p>
        <p className={styles.note}>{item.role}</p>
      </div>
    </article>
  );
}

function SpriteLayer({
  className,
  frame,
  frameSize,
  item,
}: {
  className?: string;
  frame?: number;
  frameSize: number;
  item: AssetSpriteAnimation | AssetSpriteStaticPose | AssetSpriteZLayer;
}) {
  const actualFrame = resolveFrame(item, frame ?? 0);
  const row = Math.floor(actualFrame / item.cols);
  const col = actualFrame % item.cols;
  const style = {
    "--frame-position": `-${col * frameSize}px -${row * frameSize}px`,
    "--frame-size": `${frameSize}px`,
    "--sheet-size": `${item.cols * frameSize}px ${item.rows * frameSize}px`,
    backgroundImage: `url("${assetUrl(item.file)}")`,
  } as CSSProperties;

  return (
    <div
      aria-label={item.name}
      className={[className ?? styles.sprite].filter(Boolean).join(" ")}
      data-tint={item.tint ?? undefined}
      role="img"
      style={style}
    />
  );
}

function IconsPanel({
  manifest,
  icons,
}: {
  manifest: AssetManifest;
  icons: IconManifest;
}) {
  return (
    <>
      <section className={styles.panel}>
        <div>
          <h2>Icons</h2>
          <p className={styles.note}>
            Source sheet plus exported SVG pack. The grid uses external SVG files for visual
            inspection and filters them to white for a consistent guide view.
          </p>
        </div>
        <img
          alt="Current icon sheet"
          className={styles.iconSheet}
          src={assetUrl(`${manifest.icons.directory}/${icons.source_sheet}`)}
        />
      </section>
      <section className={styles.panel}>
        <h2>Exported SVGs</h2>
        <div className={styles.iconGrid}>
          {icons.icons.map((icon) => (
            <article key={icon.name} className={styles.iconCard}>
              <img src={assetUrl(`${manifest.icons.directory}/${icon.name}.svg`)} alt={icon.label} />
              <div className={styles.iconName}>{icon.name}</div>
              <div className={styles.meta}>
                r{icon.row} c{icon.col} · {icon.category}
              </div>
            </article>
          ))}
        </div>
      </section>
    </>
  );
}

function StatePanel({ message }: { message: string }) {
  return (
    <section className={styles.state}>
      <h2>Visuals</h2>
      <p className={styles.note}>{message}</p>
    </section>
  );
}

function useAnimationFrame(paused: boolean): number {
  const [now, setNow] = useState(0);

  useEffect(() => {
    let frameId = 0;
    const tick = (time: number) => {
      if (!paused) {
        setNow(time);
      }
      frameId = requestAnimationFrame(tick);
    };
    frameId = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frameId);
    };
  }, [paused]);

  return now;
}

function resolveFrame(
  item: AssetSpriteAnimation | AssetSpriteStaticPose | AssetSpriteZLayer,
  now: number,
): number {
  if ("frame" in item && typeof item.frame === "number") {
    return item.frame;
  }
  if (
    !("start" in item) ||
    !("end" in item) ||
    !("fps" in item) ||
    typeof item.start !== "number" ||
    typeof item.end !== "number" ||
    typeof item.fps !== "number"
  ) {
    return 0;
  }
  const frameCount = item.end - item.start + 1;
  const offset = Math.floor((now / 1000) * item.fps) % frameCount;
  return item.start + offset;
}

function assetUrl(path: string): string {
  return `${baseUrl}assets/${path}`;
}

function basename(path: string): string {
  const parts = path.split("/");
  return parts[parts.length - 1] ?? path;
}

function zClassName(z: string): string {
  const zClasses: Record<string, string> = {
    highlight: styles.zHighlight,
    hover: styles.zHover,
    pointer: styles.zPointer,
    stone: styles.zStone,
    stoneWhite: styles.zStoneWhite,
    surface: styles.zSurface,
  };
  return [styles.stackSprite, zClasses[z] ?? styles.zSurface].join(" ");
}

function fontSampleClass(font: AssetGuideFontRole): string {
  if (font.family === "PixelOperatorBold") {
    return `${styles.fontSample} ${styles.fontDisplay}`;
  }
  if (font.family === "PixelOperator8Bold") {
    return `${styles.fontSample} ${styles.fontSequence}`;
  }
  return `${styles.fontSample} ${styles.fontBody}`;
}

function TintFilters() {
  return (
    <svg aria-hidden="true" height="0" width="0">
      <filter id="asset-tint-black" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.0534 0.1797 0.0181 0 0 0.0534 0.1797 0.0181 0 0 0.0534 0.1797 0.0181 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-green" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.0567 0.1907 0.0193 0 0 0.1842 0.6202 0.0626 0 0 0.0567 0.1907 0.0193 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-red" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.2126 0.7152 0.0722 0 0 0.0567 0.1907 0.0193 0 0 0.0567 0.1907 0.0193 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-gray" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.1134 0.3814 0.0385 0 0 0.1134 0.3814 0.0385 0 0 0.1134 0.3814 0.0385 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-pink" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.2126 0.7152 0.0722 0 0 0.1016 0.3418 0.0345 0 0 0.1518 0.5108 0.0515 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-purple" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.1535 0.5165 0.0521 0 0 0.0993 0.3339 0.0337 0 0 0.2126 0.7152 0.0722 0 0 0 0 0 1 0" />
      </filter>
      <filter id="asset-tint-teal" colorInterpolationFilters="sRGB">
        <feColorMatrix type="matrix" values="0.0793 0.2667 0.0269 0 0 0.1659 0.5576 0.0563 0 0 0.1618 0.5442 0.0549 0 0 0 0 0 1 0" />
      </filter>
    </svg>
  );
}
