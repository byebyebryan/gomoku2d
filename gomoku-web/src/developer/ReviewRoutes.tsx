import { Link, useLocation } from "react-router-dom";
import records from "./reviews.json";
import { isReviewRecord, type ReviewRecord } from "./review_record";
import { ReviewCase } from "./ReviewCase";
import { ReviewChecklist } from "./ReviewChecklist";
import { useDocumentTitle } from "../app/useDocumentTitle";

const catalog: ReviewRecord[] = records.filter(isReviewRecord);
if (catalog.length !== records.length) throw new Error("Unsupported review catalog");
export function ReviewRoutes() {
  const { pathname } = useLocation();
  const path = decodeURIComponent(pathname).replace(/\/+$/, "");
  const id = path.split("/").slice(3).join("/");
  const record = catalog.find((item) => item.id === id);
  useDocumentTitle(record?.title ?? "UI review");
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
        <p className="dev-eyebrow">Screens, comparisons & findings</p>
        <h1>UI review</h1>
        <p className="dev-lead">
          Inspect the screens. Compare the changes. Follow up on what still needs work.
        </p>
      </header>
      <ReviewChecklist />
      {[
        {
          title: "Current screens",
          records: catalog.filter((item) => item.disposition === "screen reference"),
        },
        {
          title: "UI evolution",
          records: catalog.filter((item) => item.disposition === "retrospective comparison"),
        },
        {
          title: "Design & layout reviews",
          records: catalog.filter(
            (item) =>
              item.disposition !== "screen reference" &&
              item.disposition !== "retrospective comparison",
          ),
        },
      ]
        .filter((group) => group.records.length > 0)
        .map((group) => (
          <section className="dev-section" key={group.title}>
            <h2>{group.title}</h2>
            <div className="review-index">
              {group.records.map((item) => (
                <article key={item.id}>
                  <div>
                    <h2>{item.title}</h2>
                    <p className="dev-muted">{item.summary}</p>
                    <p className="dev-meta">
                      {item.disposition === "retrospective comparison" ? "Recaptured " : ""}
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
          </section>
        ))}
    </>
  );
}
