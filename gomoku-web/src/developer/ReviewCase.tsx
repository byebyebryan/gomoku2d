import { useEffect, useRef } from "react";
import { Link, useSearchParams } from "react-router-dom";
import type { ReviewRecord } from "./review_record";
import { ReviewComparison } from "./ReviewComparison";
import { ReviewChecklist } from "./ReviewChecklist";
import "./reviews.css";

export function ReviewCase({ record, mediaRoot }: { record: ReviewRecord; mediaRoot?: string }) {
  const [params] = useSearchParams();
  const primary = record.comparisons.filter((pair) => pair.purpose !== "layout-check");
  const checks = record.comparisons.filter((pair) => pair.purpose === "layout-check");
  const first = primary[0] ?? record.comparisons[0];
  const selected = record.comparisons.find((pair) => pair.id === params.get("pair"));
  const sections = useRef(new Map<string, HTMLElement>());
  const checkDetails = useRef<HTMLDetailsElement>(null);
  const previous = useRef<string | undefined>(undefined);
  useEffect(() => {
    const old = previous.current;
    previous.current = selected?.id;
    if (!selected && !old) return;
    if (selected?.purpose === "layout-check" && checkDetails.current)
      checkDetails.current.open = true;
    const frame = requestAnimationFrame(() => {
      const section = sections.current.get(selected?.id ?? first.id);
      if (section?.contains(document.activeElement)) return;
      section?.scrollIntoView({ block: "start" });
      section?.focus({ preventScroll: true });
    });
    return () => cancelAnimationFrame(frame);
  }, [selected?.id, selected?.purpose, first.id]);
  const commonConditions = primary.every((pair) => pair.conditions === first.conditions)
    ? first.conditions
    : null;
  const root = mediaRoot ?? `${import.meta.env.BASE_URL}reviews/${record.id}/`;
  function renderComparison(pair: ReviewRecord["comparisons"][number]) {
    return (
      <section
        className="review-comparison"
        data-comparison={pair.id}
        key={pair.id}
        tabIndex={-1}
        aria-labelledby={`review-${record.id}-${pair.id}`}
        ref={(element) => {
          if (element) sections.current.set(pair.id, element);
          else sections.current.delete(pair.id);
        }}
      >
        <h2 id={`review-${record.id}-${pair.id}`}>{pair.title}</h2>
        {pair.conditions !== commonConditions && (
          <p className="dev-meta dev-readable">{pair.conditions}</p>
        )}
        <ReviewComparison comparison={pair} root={root} firstId={first.id} />
      </section>
    );
  }
  return (
    <article className="review-case">
      <header className="dev-heading">
        <p className="dev-eyebrow">Review / {record.scope}</p>
        <h1>{record.title}</h1>
        <p className="dev-lead">{record.question}</p>
        <p className="dev-meta">
          {record.disposition === "retrospective comparison" ? "Recaptured " : ""}
          {record.date} · {record.revision} · {record.disposition}
        </p>
        <p className="dev-readable">{record.summary}</p>
      </header>
      <nav className="dev-views" aria-label="Screens in this review">
        {primary.map((pair) => {
          const next = new URLSearchParams(params);
          next.set("pair", pair.id);
          for (const key of ["native", "full", "sync"]) next.delete(key);
          return (
            <Link
              className="dev-tool"
              key={pair.id}
              to={{ search: next.toString() }}
              preventScrollReset
              aria-current={selected?.id === pair.id ? "location" : undefined}
            >
              {pair.title}
            </Link>
          );
        })}
      </nav>
      {commonConditions && <p className="dev-meta dev-readable">{commonConditions}</p>}
      <div className="review-gallery">{primary.map(renderComparison)}</div>
      {checks.length > 0 && (
        <details className="dev-disclosure review-layout-checks" ref={checkDetails}>
          <summary>Layout checks: landscape & large text</summary>
          <div>{checks.map(renderComparison)}</div>
        </details>
      )}
      <ReviewChecklist />
      <section className="dev-section">
        <h2>
          {record.disposition === "screen reference"
            ? "Reference notes"
            : record.disposition === "retrospective comparison"
              ? "Visible changes"
              : "Findings and decisions"}
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
          {record.comparisons
            .flatMap((pair) => pair.media)
            .map((media) => (
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
