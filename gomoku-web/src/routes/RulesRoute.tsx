import { useEffect } from "react";
import { Link } from "react-router-dom";

import type { BoardOverlay, BoardViewModel } from "../board/board_model";
import { Board } from "../components/Board/Board";
import type { CellStone } from "../game/types";

import styles from "./RulesRoute.module.css";

type RuleCell =
  | "black"
  | "blackWin"
  | "candidate"
  | "empty"
  | "forbidden"
  | "winningMove"
  | "white";

const RULE_BOARD_SIZE = 7;

const FREESTYLE_FIVE: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "blackWin", "blackWin", "blackWin", "blackWin", "blackWin", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_OVERLINE: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["black", "black", "black", "black", "black", "forbidden", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_DOUBLE_FOUR: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "black", "black", "forbidden", "black", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_DOUBLE_THREE: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "black", "black", "empty", "forbidden", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "black", "empty", "empty", "empty", "empty"],
  ["empty", "black", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_LEGAL_FOUR_THREE: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "black", "black", "winningMove", "black", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_FORBIDDEN_BLOCK_TRAP: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "black", "empty"],
  ["empty", "empty", "empty", "empty", "black", "black", "empty"],
  ["black", "white", "white", "white", "white", "forbidden", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const BOXED_BRANCH: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "black", "black", "candidate", "white", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "black", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const BOXED_BRANCH_AFTER: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "black", "black", "black", "white", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "black", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const ILLEGAL_BRANCH: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "black", "black", "candidate", "empty", "empty", "empty"],
  ["empty", "empty", "black", "empty", "black", "empty", "empty"],
  ["empty", "black", "empty", "empty", "black", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "black", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const ILLEGAL_BRANCH_AFTER: RuleCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "black", "black", "black", "forbidden", "empty", "empty"],
  ["empty", "empty", "black", "empty", "black", "empty", "empty"],
  ["empty", "black", "empty", "empty", "black", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "black", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

export function RulesRoute() {
  useEffect(() => {
    document.title = "Gomoku2D Rules";
  }, []);

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">How to play</p>
              <h1 className={styles.title}>Rules</h1>
            </div>
            <nav className={styles.links} aria-label="Rules links">
              <Link className="uiAction uiActionNeutral" to="/">
                <span className="uiActionLabel">Home</span>
              </Link>
              <Link className="uiAction uiActionNeutral" to="/guide/">
                <span className="uiActionLabel">Guide</span>
              </Link>
              <Link className="uiAction uiActionPrimary" to="/match/local">
                <span className="uiActionLabel">Play</span>
              </Link>
            </nav>
          </div>
        </header>

        <div className={styles.content}>
          <section className={`${styles.panel} ${styles.freestylePanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Freestyle</p>
              <h2>Five or more wins.</h2>
            </div>
            <div className={styles.freestyleBody}>
              <RuleDiagram
                label="Freestyle five in a row diagram"
                model={ruleModelFromCells(FREESTYLE_FIVE)}
              />
              <p>
                Black moves first. Either side wins by making a line of five or more.
              </p>
            </div>
          </section>

          <section className={`${styles.panel} ${styles.renjuPanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Renju</p>
              <h2>Black plays under restrictions.</h2>
              <p>
                To reduce first-move advantage, Black cannot play overlines,
                double-four, or double-three. White does not have these restrictions.
              </p>
            </div>
            <RuleFramePanel
              label="Renju forbidden move examples"
              frames={[
                { label: "Overline", model: ruleModelFromCells(RENJU_OVERLINE) },
                { label: "Double-four", model: ruleModelFromCells(RENJU_DOUBLE_FOUR) },
                { label: "Double-three", model: ruleModelFromCells(RENJU_DOUBLE_THREE) },
              ]}
            />
            <article className={`${styles.renjuInsert} ${styles.legalThreatInsert}`}>
              <RuleDiagram
                label="Renju legal four-three threat diagram"
                model={ruleModelFromCells(RENJU_LEGAL_FOUR_THREE)}
              />
              <div>
                <h3>Four + Three</h3>
                <p>
                  The only lethal combo Black is allowed to play is four + three.
                </p>
              </div>
            </article>
            <article className={`${styles.renjuInsert} ${styles.trapInsert}`}>
              <RuleDiagram
                label="Renju forbidden block trap diagram"
                model={ruleModelFromCells(RENJU_FORBIDDEN_BLOCK_TRAP)}
              />
              <div>
                <h3>Trap</h3>
                <p>
                  White can set up a threat where Black&apos;s required block is forbidden
                  by Renju.
                </p>
              </div>
            </article>
          </section>

          <section className={`${styles.panel} ${styles.threatPanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Complex Renju</p>
              <h2>Real double-four and double-three.</h2>
              <p>
                A Black move is not forbidden just because it makes a four + four or
                three + three shape. The real question is whether that shape would
                force a win for Black.
              </p>
              <p className={styles.note}>
                That means some double-four and double-three shapes are legal when one
                or both threats are blocked or cannot be materialized.
              </p>
            </div>
            <div className={styles.complexThreatGrid}>
              <RuleSequenceExample
                label="Boxed three branch non-threat diagram"
                frames={[
                  { label: "Question", model: ruleModelFromCells(BOXED_BRANCH) },
                  { label: "Result", model: ruleModelFromCells(BOXED_BRANCH_AFTER) },
                ]}
                title="Blocked branch"
                text="The horizontal three is blocked on one side, so it is not a real second threat."
                tone="quiet"
              />
              <RuleSequenceExample
                label="Renju illegal three continuation non-threat diagram"
                frames={[
                  { label: "Question", model: ruleModelFromCells(ILLEGAL_BRANCH) },
                  { label: "Result", model: ruleModelFromCells(ILLEGAL_BRANCH_AFTER) },
                ]}
                title="Forbidden continuation"
                text="The horizontal three is nullified because materializing it would require a forbidden Black move."
                tone="quiet"
              />
            </div>
          </section>
        </div>
      </div>
    </main>
  );
}

function RuleDiagram({ label, model }: { label: string; model: BoardViewModel }) {
  return (
    <div className={styles.diagram} aria-label={label} role="img">
      <Board model={model} />
    </div>
  );
}

function RuleFramePanel({
  frames,
  label,
}: {
  frames: Array<{ label: string; model: BoardViewModel }>;
  label: string;
}) {
  return (
    <article className={`${styles.example} ${styles.sequenceExample} ${styles.quietExample} ${styles.frameOnlyExample}`}>
      <div className={`${styles.frameGrid} ${styles.tripleFrameGrid}`} aria-label={label}>
        {frames.map((frame) => (
          <div key={frame.label} className={styles.frame}>
            <RuleDiagram label={`${label} ${frame.label.toLowerCase()} frame`} model={frame.model} />
            <p className={styles.frameLabel}>{frame.label}</p>
          </div>
        ))}
      </div>
    </article>
  );
}

function RuleSequenceExample({
  frames,
  label,
  text,
  title,
  tone = "default",
}: {
  frames: Array<{ label: string; model: BoardViewModel }>;
  label: string;
  text: string;
  title: string;
  tone?: "default" | "quiet";
}) {
  const toneClass = tone === "quiet" ? styles.quietExample : "";

  return (
    <article className={`${styles.example} ${styles.sequenceExample} ${toneClass}`}>
      <div className={styles.frameGrid} aria-label={label}>
        {frames.map((frame) => (
          <div key={frame.label} className={styles.frame}>
            <RuleDiagram label={`${label} ${frame.label.toLowerCase()} frame`} model={frame.model} />
            <p className={styles.frameLabel}>{frame.label}</p>
          </div>
        ))}
      </div>
      <div>
        <h3>{title}</h3>
        <p>{text}</p>
      </div>
    </article>
  );
}

function ruleModelFromCells(source: RuleCell[][]): BoardViewModel {
  const cells: CellStone[][] = source.map((row) =>
    row.map((cell) => {
      if (cell.startsWith("black")) {
        return 0;
      }
      if (cell.startsWith("white")) {
        return 1;
      }
      return null;
    }),
  );
  const forbiddenMoves = [];
  const overlays: BoardOverlay[] = [];

  for (const [rowIndex, row] of source.entries()) {
    for (const [colIndex, cell] of row.entries()) {
      const position = { row: rowIndex, col: colIndex };

      if (cell === "blackWin") {
        overlays.push({ cell: position, kind: "winningLine" });
      } else if (cell === "candidate") {
        overlays.push({ cell: position, kind: "analysis", marker: "question" });
      } else if (cell === "winningMove") {
        overlays.push({ cell: position, kind: "hint", role: "winning" });
      } else if (cell === "forbidden") {
        forbiddenMoves.push(position);
      }
    }
  }

  return {
    boardSize: RULE_BOARD_SIZE,
    forbiddenMoves,
    interaction: { kind: "readonly" },
    overlays,
    position: {
      cells,
      currentPlayer: 1,
      lastMove: null,
      moves: [],
      showSequenceNumbers: false,
      status: "playing",
    },
  };
}
