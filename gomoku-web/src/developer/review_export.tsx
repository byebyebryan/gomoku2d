import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import { ReviewCase } from "./ReviewCase";
import { isReviewRecord } from "./review_record";
import "../app/global.css";
import "./developer.css";

const record:unknown=JSON.parse(document.getElementById("review-record")?.textContent??"null");
const root=document.getElementById("root");
if(!root||!isReviewRecord(record))throw new Error("Review export has no valid record");
document.title=`Gomoku2D review · ${record.title}`;
createRoot(root).render(<BrowserRouter><div className="dev-page" tabIndex={-1}><div className="dev-shell"><header className="dev-header"><a className="dev-brand" href="https://gomoku2d.byebyebryan.com/dev/reviews/">GOMOKU2D / REVIEW</a><span className="dev-meta">Standalone record · {record.date}</span></header><main><ReviewCase record={record} mediaRoot="media/"/></main><footer className="dev-footer"><span>Frozen media · source-qualified evidence</span><a href="LICENSE.txt">Font license</a><a href="record.json">Review record</a></footer></div></div></BrowserRouter>);
