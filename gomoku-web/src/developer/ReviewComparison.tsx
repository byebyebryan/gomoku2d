import { useRef, type CSSProperties } from "react";
import { useSearchParams } from "react-router-dom";
import type { ReviewComparison as Comparison } from "./review_record";

export function ReviewComparison({
  comparison,
  root,
  firstId,
}: {
  comparison: Comparison;
  root: string;
  firstId: string;
}) {
  const [params, setParams] = useSearchParams();
  const active = (params.get("pair") ?? firstId) === comparison.id;
  const full = !active || params.get("full") !== "0";
  const native = active && params.get("native") === "1";
  const sync = !active || params.get("sync") !== "off";
  const panes = useRef<(HTMLDivElement | null)[]>([]);
  const busy = useRef(false);
  function update(values: Record<string, string | null>) {
    const next = new URLSearchParams(params);
    if (!active) for (const key of ["native", "full", "sync"]) next.delete(key);
    next.set("pair", comparison.id);
    for (const [key, value] of Object.entries(values)) {
      if (value === null) next.delete(key);
      else next.set(key, value);
    }
    setParams(next, { preventScrollReset: true });
  }
  const largestWidth = Math.max(...comparison.media.map((media) => media.width));
  const layout = {
    "--review-columns": comparison.media.map((media) => `minmax(0, ${media.width}fr)`).join(" "),
    "--review-max-width": `${comparison.media.reduce((sum, media) => sum + media.width, 0) + 24 * (comparison.media.length - 1)}px`,
  } as CSSProperties;
  return (
    <>
      <div className="dev-tools">
        <button
          className="dev-tool"
          aria-pressed={!native}
          onClick={() => update({ native: null })}
        >
          Matched scale
        </button>
        <button className="dev-tool" aria-pressed={native} onClick={() => update({ native: "1" })}>
          Native pixels
        </button>
        <button
          className="dev-tool"
          aria-pressed={full}
          onClick={() => update({ full: full ? "0" : null })}
        >
          Full captures
        </button>
        {comparison.media.length > 1 && (!full || native) && (
          <button
            className="dev-tool"
            aria-pressed={sync}
            onClick={() => update({ sync: sync ? "off" : null })}
          >
            Scroll together {sync ? "on" : "off"}
          </button>
        )}
      </div>
      <div
        className={`review-pair ${full ? "review-full" : ""} ${native ? "review-native" : ""}`}
        data-count={comparison.media.length}
        style={layout}
      >
        {comparison.media.map((media, index) => (
          <figure
            key={media.file}
            style={
              {
                "--review-width-ratio": media.width / largestWidth,
              } as CSSProperties
            }
          >
            <figcaption>
              <strong>{media.label}</strong>
              <span>
                {media.kind} · {media.width} × {media.height} · {media.revision}
              </span>
            </figcaption>
            <div
              className="review-capture"
              tabIndex={!full || native ? 0 : -1}
              role="region"
              aria-label={`${media.label} capture${!full || native ? ", scroll to inspect" : ""}`}
              ref={(element) => {
                panes.current[index] = element;
              }}
              onScroll={(event) => {
                if (!sync || busy.current) return;
                busy.current = true;
                const source = event.currentTarget;
                for (const target of panes.current) {
                  if (!target || source === target) continue;
                  const y = source.scrollHeight - source.clientHeight,
                    x = source.scrollWidth - source.clientWidth;
                  target.scrollTop =
                    y > 0
                      ? (source.scrollTop / y) * (target.scrollHeight - target.clientHeight)
                      : 0;
                  target.scrollLeft =
                    x > 0 ? (source.scrollLeft / x) * (target.scrollWidth - target.clientWidth) : 0;
                }
                requestAnimationFrame(() => {
                  busy.current = false;
                });
              }}
            >
              <img
                src={root + media.file}
                width={media.width}
                height={media.height}
                alt={media.alt}
                loading="lazy"
              />
            </div>
            <a className="review-original dev-meta" href={root + media.file}>
              Original image
            </a>
          </figure>
        ))}
      </div>
    </>
  );
}
