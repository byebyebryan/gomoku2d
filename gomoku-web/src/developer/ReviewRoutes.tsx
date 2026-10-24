import { Link, useLocation } from "react-router-dom";
import records from "./reviews.json";
import { isReviewRecord, type ReviewRecord } from "./review_record";
import { ReviewCase } from "./ReviewCase";
import { useDocumentTitle } from "../app/useDocumentTitle";

const catalog: ReviewRecord[] = records.filter(isReviewRecord);
if (catalog.length !== records.length) throw new Error("Unsupported review catalog");
export function ReviewRoutes() {
  const { pathname } = useLocation();
  const path = decodeURIComponent(pathname).replace(/\/+$/, "");
  const id = path.split("/").slice(3).join("/");
  const record = catalog.find((item) => item.id === id);
  useDocumentTitle(record?.title ?? "Reviews");
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
        <Link className="review-index-link" to="/dev/reviews/">
          ← Review index
        </Link>
        <ReviewCase key={record.id} record={record} />
      </>
    );
  return (
    <>
      <header className="dev-heading">
        <p className="dev-eyebrow">Reviews & references</p>
        <h1>Inspect the work</h1>
        <p className="dev-lead">
          Concrete comparisons, the findings they produced and what happened next.
        </p>
      </header>
      <p className="dev-readable dev-muted">
        Current screen references and historical review checkpoints have different jobs. Each record
        keeps its own source, capture conditions and acceptance status.
      </p>
      <div className="review-index">
        {catalog.map((item) => (
          <article key={item.id}>
            <div>
              <h2>{item.title}</h2>
              <p className="dev-muted">{item.summary}</p>
              <p className="dev-meta">
                {item.date} · {item.scope} · {item.disposition}
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
    </>
  );
}
