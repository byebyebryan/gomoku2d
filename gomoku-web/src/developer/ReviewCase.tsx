import { useRef, type CSSProperties } from "react";
import { useSearchParams } from "react-router-dom";
import type { ReviewRecord } from "./review_record";
import { ReviewChecklist } from "./ReviewChecklist";
import "./reviews.css";

export function ReviewCase({ record, mediaRoot }: { record: ReviewRecord; mediaRoot?: string }) {
  const [params, setParams] = useSearchParams();
  const comparison =
    record.comparisons.find((pair) => pair.id === params.get("pair")) ?? record.comparisons[0];
  const full = params.get("full") !== "0",
    native = params.get("native") === "1",
    sync = params.get("sync") !== "off";
  const panes = useRef<(HTMLDivElement | null)[]>([]),
    busy = useRef(false);
  function update(values: Record<string, string | null>) {
    const next = new URLSearchParams(params);
    for (const [key, value] of Object.entries(values)) {
      if (value === null) next.delete(key);
      else next.set(key, value);
    }
    setParams(next, { preventScrollReset: true });
  }
  const root = mediaRoot ?? `${import.meta.env.BASE_URL}reviews/${record.id}/`;
  const largestWidth = Math.max(...comparison.media.map((media) => media.width));
  const layout = {
    "--review-columns": comparison.media.map((media) => `minmax(0, ${media.width}fr)`).join(" "),
    "--review-max-width": `${comparison.media.reduce((sum, media) => sum + media.width, 0) + 24 * (comparison.media.length - 1)}px`,
  } as CSSProperties;
  return (
    <article className="review-case">
      <header className="dev-heading">
        <p className="dev-eyebrow">Review / {record.scope}</p>
        <h1>{record.title}</h1>
        <p className="dev-lead">{record.question}</p>
        <p className="dev-meta">
          {record.date} · {record.revision} · {record.disposition}
        </p>
        <p className="dev-readable">{record.summary}</p>
      </header>
      {record.comparisons.length > 4 ? (
        <label className="review-choice">
          Screen and conditions
          <select value={comparison.id} onChange={(event) => update({ pair: event.target.value })}>
            {record.comparisons.map((pair) => (
              <option key={pair.id} value={pair.id}>
                {pair.title}
              </option>
            ))}
          </select>
        </label>
      ) : (
        <nav className="dev-views" aria-label="Review comparisons">
          {record.comparisons.map((pair) => (
            <button
              className="dev-tool"
              key={pair.id}
              aria-pressed={pair.id === comparison.id}
              onClick={() => update({ pair: pair.id })}
            >
              {pair.title}
            </button>
          ))}
        </nav>
      )}
      <p className="dev-meta dev-readable">{comparison.conditions}</p>
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
      <ReviewChecklist />
      <section className="dev-section">
        <h2>
          {record.disposition === "screen reference" ? "Reference notes" : "Findings and decisions"}
        </h2>
        <ul className="review-findings">
          {record.findings.map((finding, index) => (
            <li key={index}>
              <span>{finding.state}</span>
              <div>
                <h3>{finding.title}</h3>
                <p className="dev-muted">{finding.detail}</p>
              </div>
            </li>
          ))}
        </ul>
      </section>
      <section className="dev-section dev-readable">
        <h2>Human review and follow-up</h2>
        <p>{record.human}</p>
        <p className="dev-muted">{record.followUp}</p>
      </section>
      <details className="dev-disclosure">
        <summary>Technical evidence, sources and media identity</summary>
        <div>
          <ul>
            {record.technical.map((line, index) => (
              <li key={index}>{line}</li>
            ))}
          </ul>
          <ul>
            {record.sources.map((source) => (
              <li key={source.url}>
                <a href={source.url}>{source.label}</a>
              </li>
            ))}
          </ul>
          {comparison.media.map((media) => (
            <p key={media.file} className="dev-meta">
              {media.label}
              <br />
              <code>{media.sourcePath}</code>
              <br />
              SHA-256 <code>{media.sha256}</code>
            </p>
          ))}
        </div>
      </details>
    </article>
  );
}
