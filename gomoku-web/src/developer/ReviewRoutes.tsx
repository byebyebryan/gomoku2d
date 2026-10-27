import { Link, useLocation } from "react-router-dom";
import records from "./reviews.json";
import { isReviewRecord, type ReviewRecord } from "./review_record";
import { ReviewCase } from "./ReviewCase";
import { ReviewChecklist } from "./ReviewChecklist";
import { useDocumentTitle } from "../app/useDocumentTitle";
import {
  HistoryRoutes,
  ReviewNavigation,
  historyRouteTitle,
} from "./HistoryRoutes";

const rawRecords: unknown[] = records;
const catalog: ReviewRecord[] = rawRecords.filter(isReviewRecord);
if (catalog.length !== records.length)
  throw new Error("Unsupported review catalog");

function latestVersion(record: ReviewRecord) {
  return record.versions?.[record.versions.length - 1] ?? "";
}
function versionLabel(record: ReviewRecord) {
  const versions = record.versions;
  if (!versions?.length) return record.scope;
  return versions.length === 1
    ? `v${versions[0]}`
    : `v${versions[0]}–${versions[versions.length - 1]}`;
}
function phaseOf(record: ReviewRecord) {
  return latestVersion(record).split(".").slice(0, 2).join(".");
}
const designReviews = catalog
  .filter(
    (record) =>
      record.disposition !== "screen reference" &&
      record.disposition !== "retrospective comparison",
  )
  .sort(
    (a, b) =>
      latestVersion(b).localeCompare(latestVersion(a), "en", { numeric: true }) ||
      b.date.localeCompare(a.date) || a.title.localeCompare(b.title),
  );
const phases = Array.from(
  new Set(designReviews.map(phaseOf)),
);

export function ReviewRoutes() {
  const { pathname } = useLocation();
  const path = decodeURIComponent(pathname).replace(/\/+$/, "");
  const id = path.split("/").slice(3).join("/");
  const record = catalog.find((item) => item.id === id);
  useDocumentTitle(
    record?.title ??
      (id === "history" || id.startsWith("history/")
        ? historyRouteTitle(
            id === "history" ? undefined : id.slice("history/".length),
          )
        : "Design reviews"),
  );
  if (id === "history" || id.startsWith("history/"))
    return (
      <HistoryRoutes
        screenId={id === "history" ? undefined : id.slice("history/".length)}
      />
    );
  if (id && !record)
    return (
      <section>
        <h1>Review not found</h1>
        <p>This review is not in the curated catalog.</p>
        <Link className="uiAction uiActionNeutral" to="/dev/reviews/">
          Review index
        </Link>
      </section>
    );
  if (record)
    return (
      <>
        <ReviewNavigation designReview={record.disposition !== "screen reference" && record.disposition !== "retrospective comparison"} />
        {!record.process && (
          <Link className="review-index-link" to="/dev/reviews/">
            ← Review index
          </Link>
        )}
        <ReviewCase key={record.id} record={record} />
      </>
    );
  return (
    <>
      <ReviewNavigation />
      <header className="dev-heading">
        <p className="dev-eyebrow">Choices, revisions & findings</p>
        <h1>Design reviews</h1>
        <p className="dev-lead">
          Inspect the choices and revisions behind the interface.
        </p>
      </header>
      <ReviewChecklist />
      {phases.map((phase) => (
          <section className="dev-section review-phase" key={phase || "other"}>
            <h2>{phase || "Other reviews"}</h2>
            <div className="review-index">
              {designReviews.filter((item) => phaseOf(item) === phase).map((item) => (
                <article key={item.id} data-review={item.id}>
                  <div>
                    <p className="dev-meta">{versionLabel(item)}</p>
                    <h3>{item.title}</h3>
                    <p className="dev-muted">{item.summary}</p>
                    <p className="dev-meta">
                      {item.disposition === "retrospective review" ? "Assembled " : ""}
                      {item.date} · {item.disposition}
                    </p>
                  </div>
                  <Link
                    className="uiAction uiActionNeutral"
                    to={`/dev/reviews/${item.id}/`}
                    aria-label={`Review: ${item.title}`}
                  >
                    Review
                  </Link>
                </article>
              ))}
            </div>
          </section>
        ))}
      <details className="dev-disclosure review-reference-links">
        <summary>Reference galleries</summary>
        <div>
          <ul className="dev-record-list">
            {catalog.filter((record) => record.disposition === "screen reference" || record.disposition === "retrospective comparison").map((record) => (
              <li key={record.id}>
                <Link to={`/dev/reviews/${record.id}/`} aria-label={`Review: ${record.title}`}>
                  {record.title}
                </Link>
                <small>{record.summary}</small>
              </li>
            ))}
          </ul>
        </div>
      </details>
    </>
  );
}
