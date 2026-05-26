import { useEffect } from "react";
import { Link } from "react-router-dom";

import type { BoardOverlay, BoardViewModel } from "../board/board_model";
import { Board } from "../components/Board/Board";
import type { CellStone } from "../game/types";

import styles from "./GuideRoute.module.css";

type GuideCell = "black" | "counter" | "empty" | "entry" | "reply" | "white" | "win";

const GUIDE_BOARD_SIZE = 7;

const IMMEDIATE_THREAT: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "white", "black", "black", "black", "black", "win"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const IMMINENT_THREAT: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "reply", "black", "black", "black", "reply", "empty"],
  ["empty", "empty", "empty", "counter", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const FORK_THREAT: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "black", "black", "black", "black", "win", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const CORRIDOR: GuideCell[][] = [
  ["empty", "entry", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "black", "white", "black", "reply", "empty", "empty"],
  ["empty", "empty", "white", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "black", "win", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const LEGEND: Array<{ kind: GuideCell; label: string }> = [
  { kind: "black", label: "attacker stone" },
  { kind: "white", label: "defender stone" },
  { kind: "win", label: "immediate target" },
  { kind: "reply", label: "imminent reply" },
  { kind: "counter", label: "counter threat" },
  { kind: "entry", label: "last escape" },
];

export function GuideRoute() {
  useEffect(() => {
    document.title = "Gomoku2D Guide";
  }, []);

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">How to read the board</p>
              <h1 className={styles.title}>Guide</h1>
              <p className={styles.summary}>
                Good Gomoku is threat reading: stop the move that wins now, spot the
                move that wins next, and notice when one reply is no longer enough.
              </p>
            </div>
            <nav className={styles.links} aria-label="Guide links">
              <Link className="uiAction uiActionNeutral" to="/">
                <span className="uiActionLabel">Home</span>
              </Link>
              <Link className="uiAction uiActionNeutral" to="/rules/">
                <span className="uiActionLabel">Rules</span>
              </Link>
              <Link className="uiAction uiActionPrimary" to="/match/local">
                <span className="uiActionLabel">Play</span>
              </Link>
            </nav>
          </div>
        </header>

        <section className={styles.legend} aria-label="Guide diagram legend">
          {LEGEND.map((item) => (
            <span key={item.label} className={styles.legendItem}>
              <span className={`${styles.cell} ${styles[item.kind]}`} aria-hidden="true" />
              {item.label}
            </span>
          ))}
        </section>

        <div className={styles.lessonGrid}>
          <GuideLesson
            accent="danger"
            title="Immediate Threat"
            kicker="Fours"
            model={guideModelFromCells(IMMEDIATE_THREAT)}
            text="A four is one move away from five. If the opponent has a real four, block it now unless you can win immediately."
          />
          <GuideLesson
            accent="reply"
            title="Imminent Threat"
            kicker="Threes"
            model={guideModelFromCells(IMMINENT_THREAT)}
            text="An open or real broken three is not a win yet, but it can become a four next. These are the pink reply hints in game and replay analysis."
          />
          <GuideLesson
            accent="danger"
            title="Lethal Threat"
            kicker="Forks"
            model={guideModelFromCells(FORK_THREAT)}
            text="A fork creates more threats than one move can cover: open four, 4+4, 4+3, or 3+3. Once this appears, the defender is already losing."
          />
          <GuideLesson
            accent="entry"
            title="Forced Corridor"
            kicker="Replay analysis"
            model={guideModelFromCells(CORRIDOR)}
            text="A corridor is a chain of must-answer threats. The analyzer walks backward through it to find the last move that could still escape."
          />
        </div>

        <section className={styles.readerPanel}>
          <div>
            <p className="uiSectionLabel">Replay reading order</p>
            <h2>Start from the end, then walk back.</h2>
          </div>
          <ol>
            <li>
              <strong>Win:</strong> the final five or lethal result.
            </li>
            <li>
              <strong>Onset:</strong> the first frame where the loser can no longer
              fully answer the threat.
            </li>
            <li>
              <strong>Last escape:</strong> the last earlier move that could avoid
              the losing corridor.
            </li>
          </ol>
        </section>
      </div>
    </main>
  );
}

function GuideLesson({
  accent,
  kicker,
  model,
  text,
  title,
}: {
  accent: "danger" | "entry" | "reply";
  kicker: string;
  model: BoardViewModel;
  text: string;
  title: string;
}) {
  return (
    <article className={`${styles.lesson} ${styles[`${accent}Accent`]}`}>
      <div>
        <p className={styles.kicker}>{kicker}</p>
        <h2>{title}</h2>
      </div>
      <div className={styles.diagram} aria-label={`${title} diagram`} role="img">
        <Board model={model} />
      </div>
      <p>{text}</p>
    </article>
  );
}

function guideModelFromCells(source: GuideCell[][]): BoardViewModel {
  const cells: CellStone[][] = source.map((row) =>
    row.map((cell) => {
      if (cell === "black") {
        return 0;
      }
      if (cell === "white") {
        return 1;
      }
      return null;
    }),
  );
  const overlays: BoardOverlay[] = [];

  for (const [rowIndex, row] of source.entries()) {
    for (const [colIndex, cell] of row.entries()) {
      const position = { row: rowIndex, col: colIndex };

      if (cell === "win") {
        overlays.push({ cell: position, kind: "hint", role: "immediateThreat" });
      } else if (cell === "reply") {
        overlays.push({ cell: position, kind: "hint", role: "imminentThreat" });
      } else if (cell === "counter") {
        overlays.push({ cell: position, kind: "hint", role: "counterThreat" });
      } else if (cell === "entry") {
        overlays.push({
          cell: position,
          highlight: "corridorEntry",
          kind: "analysis",
          marker: "escape",
          side: "white",
        });
      }
    }
  }

  return {
    boardSize: GUIDE_BOARD_SIZE,
    forbiddenMoves: [],
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
