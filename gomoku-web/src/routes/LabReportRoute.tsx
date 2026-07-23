import { useEffect, useState, type ReactNode } from "react";
import { useSearchParams } from "react-router-dom";

import {
  loadPublishedBotReport,
  type PublishedBotReport,
} from "../reports/bot_report";
import {
  loadAnalysisReport,
  type PublishedAnalysisReport,
} from "../reports/analysis_report";
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

const baseUrl = import.meta.env.BASE_URL;

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
  const [searchParams, setSearchParams] = useSearchParams();
  const view = parseReportView(searchParams.get("tab"));
  const analysisMatchPath = view === "analysis" ? searchParams.get("match") : null;
  const [analysisState, setAnalysisState] = useState<LoadState<PublishedAnalysisReport>>({
    status: "idle",
  });
  const [botState, setBotState] = useState<LoadState<PublishedBotReport>>({ status: "idle" });

  useEffect(() => {
    document.title = "Gomoku2D Lab Report";
  }, []);

  useEffect(() => {
    if (view === "analysis" || botState.status !== "idle") {
      return;
    }

    let cancelled = false;
    setBotState({ status: "loading" });
    loadPublishedBotReport()
      .then((report) => {
        if (!cancelled) {
          setBotState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setBotState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [view]);

  useEffect(() => {
    if (view !== "analysis" || analysisState.status !== "idle") {
      return;
    }

    let cancelled = false;
    setAnalysisState({ status: "loading" });
    loadAnalysisReport()
      .then((report) => {
        if (!cancelled) {
          setAnalysisState({ status: "loaded", report });
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setAnalysisState({
            status: "error",
            message: error instanceof Error ? error.message : String(error),
          });
        }
      });
    return () => {
      cancelled = true;
    };
  }, [view]);

  const setView = (nextView: ReportView) => {
    const nextParams = new URLSearchParams();
    if (nextView !== "ranking") {
      nextParams.set("tab", nextView);
    }
    const currentMatch = searchParams.get("match");
    if (nextView === "analysis" && currentMatch) {
      nextParams.set("match", currentMatch);
    }
    setSearchParams(nextParams);
  };

  let content: ReactNode;

  if (view === "analysis") {
    if (analysisState.status === "error") {
      content = <ReportStatePanel title="Lab Report" message={analysisState.message} />;
    } else if (analysisState.status !== "loaded") {
      content = <ReportStatePanel title="Lab Report" message="Loading report…" />;
    } else {
      content = (
        <>
          <AnalysisReportContent report={analysisState.report} initialMatchPath={analysisMatchPath} />
          <AnalysisHowToReadSection />
          <AnalysisProvenanceSection report={analysisState.report} />
        </>
      );
    }
  } else if (botState.status === "error") {
    content = <ReportStatePanel title="Lab Report" message={botState.message} />;
  } else if (botState.status !== "loaded") {
    content = <ReportStatePanel title="Lab Report" message="Loading report…" />;
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
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div className={styles.headerCopy}>
              <p className="uiPageEyebrow">Gomoku2D lab</p>
              <h1 className={styles.title}>Lab Report</h1>
              <p className={styles.subtitle}>
                Inner workings of a Gomoku bot: rankings, search telemetry, and replay
                analysis.
              </p>
            </div>
            <nav className={styles.links} aria-label="Report links">
              <a className="uiAction uiActionNeutral" href={baseUrl}>
                <span className="uiActionLabel">Home</span>
              </a>
              <a className="uiAction uiActionNeutral" href={`${baseUrl}visuals/`}>
                <span className="uiActionLabel">Visuals</span>
              </a>
            </nav>
          </div>
          <ReportTabs value={view} onChange={setView} />
          {botState.status === "loaded" && botState.report.provenance?.git_dirty ? (
            <p className={styles.warning}>Development run: generated from a dirty git worktree.</p>
          ) : null}
        </header>

        {content}
      </div>
    </main>
  );
}

function ReportTabs({
  value,
  onChange,
}: {
  value: ReportView;
  onChange: (view: ReportView) => void;
}) {
  return (
    <div className={styles.viewToggle} aria-label="Lab report sections" role="tablist">
      {REPORT_VIEWS.map((option) => (
        <button
          key={option.id}
          type="button"
          id={`lab-report-tab-${option.id}`}
          aria-controls={`lab-report-panel-${option.id}`}
          aria-selected={value === option.id}
          className={value === option.id ? styles.activeToggle : undefined}
          onClick={() => onChange(option.id)}
          role="tab"
        >
          {option.label}
        </button>
      ))}
    </div>
  );
}

function parseReportView(value: string | null): ReportView {
  return value === "search" || value === "analysis" ? value : "ranking";
}
