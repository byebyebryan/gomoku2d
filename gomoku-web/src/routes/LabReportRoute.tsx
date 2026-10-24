import { useEffect, useRef, useState, type ReactNode } from "react";
import { Link, useSearchParams } from "react-router-dom";

import { useDocumentTitle } from "../app/useDocumentTitle";
import { loadPublishedBotReport, type PublishedBotReport } from "../reports/bot_report";
import { loadAnalysisReport, type PublishedAnalysisReport } from "../reports/analysis_report";
import { AnalysisReportContent } from "./AnalysisReportRoute";
import { BotReportPanel } from "./lab-report/BotReportPanel";
import {
  AnalysisHowToReadSection,
  AnalysisProvenanceSection,
  BotHowToReadSection,
  BotProvenanceSection,
  ReportStatePanel,
} from "./lab-report/ReportSupport";

import styles from "./ReportRoute.module.css";

type LoadState<T> =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "loaded"; report: T }
  | { status: "error"; message: string };

type ReportView = "ranking" | "search" | "analysis";

const REPORT_VIEWS: Array<{ id: ReportView; label: string }> = [
  { id: "ranking", label: "Ranking" },
  { id: "search", label: "Search" },
  { id: "analysis", label: "Analysis" },
];

export function LabReportRoute() {
  useDocumentTitle("Lab Report");

  const [searchParams] = useSearchParams();
  const view = parseReportView(searchParams.get("tab"));
  const matchPath = searchParams.get("match");
  const analysisMatchPath = view === "analysis" ? matchPath : null;
  const [analysisState, setAnalysisState] = useState<LoadState<PublishedAnalysisReport>>({
    status: "idle",
  });
  const [botState, setBotState] = useState<LoadState<PublishedBotReport>>({ status: "idle" });

  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);
  useEffect(() => {
    if (view === "analysis" || botState.status !== "idle") {
      return;
    }

    setBotState({ status: "loading" });
    loadPublishedBotReport()
      .then((report) => {
        if (mounted.current) {
          setBotState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (mounted.current) {
          setBotState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
  }, [view]);

  useEffect(() => {
    if (view !== "analysis" || analysisState.status !== "idle") {
      return;
    }

    setAnalysisState({ status: "loading" });
    loadAnalysisReport()
      .then((report) => {
        if (mounted.current) {
          setAnalysisState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (mounted.current) {
          setAnalysisState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
  }, [view]);

  let content: ReactNode;

  if (view === "analysis") {
    if (analysisState.status === "error") {
      content = <ReportStatePanel title="Report unavailable" message={analysisState.message} />;
    } else if (analysisState.status !== "loaded") {
      content = <ReportStatePanel title="Loading report" message="Fetching the published data." />;
    } else {
      content = (
        <>
          <AnalysisReportContent
            report={analysisState.report}
            initialMatchPath={analysisMatchPath}
          />
          <AnalysisHowToReadSection />
          <AnalysisProvenanceSection report={analysisState.report} />
        </>
      );
    }
  } else if (botState.status === "error") {
    content = <ReportStatePanel title="Report unavailable" message={botState.message} />;
  } else if (botState.status !== "loaded") {
    content = <ReportStatePanel title="Loading report" message="Fetching the published data." />;
  } else {
    content = (
      <>
        <BotReportPanel report={botState.report} view={view} />
        <BotHowToReadSection view={view} />
        <BotProvenanceSection report={botState.report} view={view} />
      </>
    );
  }

  return (
    <div className={styles.developerReport}>
      <header className="dev-heading">
        <p className="dev-eyebrow">Experiments & analysis</p>
        <h1>Inside the bot lab</h1>
        <p className="dev-lead">
          Compare behavior under a recorded budget, then inspect the decisions.
        </p>
        {view !== "analysis" && botState.status === "loaded" && (
          <p className="dev-meta">
            {botState.report.run.rules.variant === "renju" ? "Renju" : "Freestyle"} ·{" "}
            {botState.report.board_size} × {botState.report.board_size} ·{" "}
            {botState.report.run.games_per_pair} games per pair · Recorded{" "}
            {botState.report.provenance?.generated_at_utc?.slice(0, 10) ?? "date unavailable"}
          </p>
        )}
        {view === "analysis" && analysisState.status === "loaded" && (
          <p className="dev-meta">
            Bounded analysis · maximum depth {analysisState.report.model.max_depth} · Recorded{" "}
            {analysisState.report.provenance?.generated_at_utc?.slice(0, 10) ?? "date unavailable"}
          </p>
        )}
      </header>
      <nav className="dev-views" aria-label="Lab views">
        {REPORT_VIEWS.map((option) => (
          <Link
            key={option.id}
            to={
              option.id === "ranking"
                ? "/lab/"
                : `/lab/?tab=${option.id}${option.id === "analysis" && matchPath ? `&match=${encodeURIComponent(matchPath)}` : ""}`
            }
            aria-current={view === option.id ? "page" : undefined}
            id={`lab-report-tab-${option.id}`}
          >
            {option.label}
          </Link>
        ))}
      </nav>
      <p className={styles.reportNote}>
        Published reference runs describe their recorded engine revisions. Current engine behavior
        can differ; see Provenance for dates and settings.
      </p>
      {botState.status === "loaded" && botState.report.provenance?.git_dirty ? (
        <p className={styles.warning}>Development run: generated from a dirty git worktree.</p>
      ) : null}
      {content}
    </div>
  );
}
function parseReportView(value: string | null): ReportView {
  return value === "search" || value === "analysis" ? value : "ranking";
}
