import { useEffect } from "react";
import { Link, useLocation } from "react-router-dom";

import { useDocumentTitle } from "../app/useDocumentTitle";
import type { BoardOverlay, BoardViewModel } from "../board/board_model";
import { Board } from "../components/Board/Board";
import type { CellStone } from "../game/types";
import { exampleReplayPath, REPLAY_EXAMPLES } from "../replay/replay_examples";
import { Icon } from "../ui/Icon";

import styles from "./GuideRoute.module.css";

type GuideCell =
  | "black"
  | "counter"
  | "empty"
  | "reply"
  | "white"
  | "win";
type GuideSide = "black" | "white";
type GuidePoint = { col: number; row: number };
type GuideReplayMove = GuidePoint & { side: GuideSide };
type GuideReplayOverlayRole =
  | "counter"
  | "counterEvidence"
  | "escape"
  | "forcedLoss"
  | "immediate"
  | "immediateEvidence"
  | "imminent"
  | "imminentEvidence";

const GUIDE_BOARD_SIZE = 7;
const GUIDE_SEQUENCE_BOARD_SIZE = 7;
const GUIDE_SEQUENCE_ROW_OFFSET = 1;
const GUIDE_SEQUENCE_COL_OFFSET = 5;

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
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const COUNTER_THREAT: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "counter", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "reply", "black", "black", "black", "reply", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const WHITE_FOUR_FOUR: GuideCell[][] = [
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["black", "white", "white", "white", "white", "win", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const THREE_THREE: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "win", "white", "white", "white", "win", "empty"],
  ["empty", "empty", "empty", "white", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const RENJU_FOUR_THREE: GuideCell[][] = [
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["white", "black", "black", "black", "black", "win", "empty"],
  ["empty", "empty", "empty", "black", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "win", "empty", "empty", "empty"],
  ["empty", "empty", "empty", "empty", "empty", "empty", "empty"],
];

const REPORT_SEQUENCE_MOVES: GuideReplayMove[] = [
  { col: 7, row: 7, side: "black" },
  { col: 8, row: 7, side: "white" },
  { col: 7, row: 6, side: "black" },
  { col: 8, row: 6, side: "white" },
  { col: 7, row: 5, side: "black" },
  { col: 7, row: 8, side: "white" },
  { col: 8, row: 5, side: "black" },
  { col: 6, row: 9, side: "white" },
  { col: 7, row: 4, side: "black" },
  { col: 7, row: 3, side: "white" },
  { col: 9, row: 6, side: "black" },
  { col: 6, row: 3, side: "white" },
  { col: 6, row: 5, side: "black" },
  { col: 9, row: 5, side: "white" },
  { col: 8, row: 3, side: "black" },
  { col: 5, row: 6, side: "white" },
  { col: 6, row: 7, side: "black" },
  { col: 5, row: 8, side: "white" },
  { col: 9, row: 4, side: "black" },
  { col: 10, row: 3, side: "white" },
  { col: 8, row: 4, side: "black" },
  { col: 5, row: 10, side: "white" },
  { col: 4, row: 11, side: "black" },
  { col: 5, row: 7, side: "white" },
  { col: 5, row: 9, side: "black" },
  { col: 5, row: 5, side: "white" },
  { col: 5, row: 4, side: "black" },
  { col: 6, row: 4, side: "white" },
  { col: 8, row: 2, side: "black" },
  { col: 8, row: 1, side: "white" },
  { col: 7, row: 2, side: "black" },
  { col: 4, row: 6, side: "white" },
  { col: 3, row: 7, side: "black" },
  { col: 6, row: 1, side: "white" },
  { col: 9, row: 2, side: "black" },
];

const GUIDE_SEQUENCE_CONTEXT_MOVES: GuidePoint[] = [
  { col: 5, row: 6 },
  { col: 6, row: 4 },
  { col: 6, row: 5 },
  { col: 7, row: 4 },
  { col: 8, row: 3 },
  { col: 8, row: 4 },
  { col: 8, row: 5 },
  { col: 8, row: 6 },
  { col: 9, row: 4 },
];

const GUIDE_SEQUENCE_EXTRA_CONTEXT_STONES: GuideReplayMove[] = [
  { col: 10, row: 4, side: "white" },
];

const GUIDE_SEQUENCE_REPLY_1_MOVES: GuidePoint[] = [
  { col: 8, row: 1 },
  { col: 8, row: 2 },
];

const GUIDE_SEQUENCE_REPLY_2_MOVES: GuidePoint[] = [
  { col: 6, row: 1 },
  { col: 7, row: 2 },
];

export function GuideRoute() {
  useDocumentTitle("Guide");
  const { hash, key } = useLocation();
  useEffect(() => {
    if (!["#tactics", "#replays", "#examples"].includes(hash)) return;
    const target = document.getElementById(hash.slice(1));
    target?.scrollIntoView({ block: "start" });
    target?.focus({ preventScroll: true });
  }, [hash, key]);

  return (
    <main className={styles.page}>
      <div className={styles.shell}>
        <header className={styles.hero}>
          <div className={styles.headerRow}>
            <div>
              <p className="uiPageEyebrow">How to play better</p>
              <h1 className={styles.title}>Guide</h1>
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

        <nav className={styles.sectionNav} aria-label="Guide sections">
          <Link className="uiAction uiActionNeutral" to="/guide/#tactics"><Icon name="help" /><span className="uiActionLabel">Tactics</span></Link>
          <Link className="uiAction uiActionNeutral" to="/guide/#replays"><Icon name="share" /><span className="uiActionLabel">Replays &amp; sharing</span></Link>
          <Link className="uiAction uiActionSecondary" to="/guide/#examples"><Icon name="replay" /><span className="uiActionLabel">Examples</span></Link>
        </nav>

        <div className={styles.content}>
          <section className={`${styles.panel} ${styles.mistakePanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Avoid mistakes</p>
              <h2 className={styles.sectionAnchor} id="tactics" tabIndex={-1}>Respond to threats.</h2>
              <p>
                When a threat appears, win now, block it, or answer with something
                stronger.
              </p>
            </div>
            <div className={styles.tileGrid}>
              <GuideTile
                label="Immediate threat diagram: Black has four stones and one winning target."
                model={guideModelFromCells(IMMEDIATE_THREAT)}
                title="Immediate threat"
                tone="danger"
              />
              <GuideTile
                label="Imminent threat diagram: Black has an open three with two replies."
                model={guideModelFromCells(IMMINENT_THREAT)}
                title="Imminent threat"
                tone="imminent"
              />
              <GuideTile
                label="Counter threat diagram: White answers an open three by making a four."
                model={guideModelFromCells(COUNTER_THREAT)}
                title="Counter threat"
                tone="counter"
              />
            </div>
          </section>

          <section className={`${styles.panel} ${styles.tacticsPanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Tactics</p>
              <h2>Make a combo.</h2>
              <p>
                A combo brings several threats together. One reply cannot stop them
                all.
              </p>
            </div>
            <div className={styles.tileGrid}>
              <GuideTile
                label="Four plus four combo diagram: White has two immediate winning targets."
                model={guideModelFromCells(WHITE_FOUR_FOUR)}
                title="4+4 combo"
                tone="danger"
              />
              <GuideTile
                label="Three plus three combo diagram: White has multiple ways to make a lethal four."
                model={guideModelFromCells(THREE_THREE)}
                title="3+3 combo"
                tone="danger"
              />
              <GuideTile
                label="Four plus three combo diagram: Black's legal Renju combo."
                model={guideModelFromCells(RENJU_FOUR_THREE)}
                title="4+3 combo"
                tone="danger"
              />
            </div>
          </section>

          <section className={`${styles.panel} ${styles.strategyPanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Strategy</p>
              <h2>Force a combo.</h2>
              <p>
                Strong players do not wait for combos to appear. They force one
                through a sequence of must-answer threats.
              </p>
            </div>
            <GuideFramePanel
              label="Forced corridor sequence"
              frames={[
                {
                  label: "1. Forced reply 1",
                  model: guideModelFromReplayFrame({
                    currentPlayer: "white",
                    extraStones: GUIDE_SEQUENCE_EXTRA_CONTEXT_STONES,
                    focusMoves: [
                      { col: 8, row: 2, side: "black" },
                      { col: 8, row: 1, side: "white" },
                    ],
                    overlays: [
                      { col: 8, role: "immediate", row: 1 },
                      { col: 8, role: "immediateEvidence", row: 2 },
                      { col: 8, role: "immediateEvidence", row: 3 },
                      { col: 8, role: "immediateEvidence", row: 4 },
                      { col: 8, role: "immediateEvidence", row: 5 },
                    ],
                    prefixPly: 30,
                    visibleMoves: GUIDE_SEQUENCE_CONTEXT_MOVES,
                  }),
                },
                {
                  label: "2. Forced reply 2",
                  model: guideModelFromReplayFrame({
                    currentPlayer: "white",
                    extraStones: GUIDE_SEQUENCE_EXTRA_CONTEXT_STONES,
                    focusMoves: [
                      { col: 7, row: 2, side: "black" },
                      { col: 6, row: 1, side: "white" },
                    ],
                    omittedMoves: [
                      { col: 4, row: 6 },
                      { col: 3, row: 7 },
                    ],
                    overlays: [
                      { col: 6, role: "imminent", row: 1 },
                      { col: 10, role: "imminent", row: 5 },
                      { col: 7, role: "imminentEvidence", row: 2 },
                      { col: 8, role: "imminentEvidence", row: 3 },
                      { col: 9, role: "imminentEvidence", row: 4 },
                    ],
                    prefixPly: 34,
                    visibleMoves: [
                      ...GUIDE_SEQUENCE_CONTEXT_MOVES,
                      ...GUIDE_SEQUENCE_REPLY_1_MOVES,
                    ],
                  }),
                },
                {
                  label: "3. 4+3 combo",
                  model: guideModelFromReplayFrame({
                    currentPlayer: "white",
                    extraStones: GUIDE_SEQUENCE_EXTRA_CONTEXT_STONES,
                    focusMoves: [{ col: 9, row: 2, side: "black" }],
                    omittedMoves: [
                      { col: 4, row: 6 },
                      { col: 3, row: 7 },
                    ],
                    overlays: [
                      { col: 10, role: "immediate", row: 1 },
                      { col: 6, role: "immediate", row: 2 },
                      { col: 10, role: "immediate", row: 2 },
                      { col: 7, role: "immediateEvidence", row: 2 },
                      { col: 8, role: "immediateEvidence", row: 2 },
                      { col: 9, role: "immediateEvidence", row: 2 },
                      { col: 8, role: "immediateEvidence", row: 3 },
                      { col: 7, role: "immediateEvidence", row: 4 },
                      { col: 6, role: "immediateEvidence", row: 5 },
                    ],
                    prefixPly: 35,
                    visibleMoves: [
                      ...GUIDE_SEQUENCE_CONTEXT_MOVES,
                      ...GUIDE_SEQUENCE_REPLY_1_MOVES,
                      ...GUIDE_SEQUENCE_REPLY_2_MOVES,
                    ],
                  }),
                },
              ]}
            />
          </section>

          <section className={`${styles.panel} ${styles.analysisPanel}`}>
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Replay analysis</p>
              <h2>Figure out what went wrong.</h2>
              <p>
                After a match, Analyze opens the replay at the end and works
                backward through the losing side&apos;s decisions. Scrub any move,
                or follow the marked line from combo onset back to an escape candidate.
              </p>
            </div>
            <div className={styles.stepGrid}>
              <div>
                <h3>Combo onset</h3>
                <p>The lethal combo that forces the result within the analyzer&apos;s model.</p>
              </div>
              <div>
                <h3>Setup corridor</h3>
                <p>The forced run into onset: every reply is mandatory, or every alternative still loses.</p>
              </div>
              <div>
                <h3>Last escape</h3>
                <p>
                  E marks a confirmed escape in this analysis. P marks a possible
                  escape: the search has not proved that candidate loses.
                </p>
              </div>
            </div>
          </section>

          <section className={`${styles.panel} ${styles.sharingPanel}`} aria-labelledby="replays">
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Keep and share</p>
              <h2 className={styles.sectionAnchor} id="replays" tabIndex={-1}>Replays &amp; sharing</h2>
            </div>
            <dl className={styles.actionGuide}>
              <div><dt><Icon name="share" />Share replay</dt><dd>Create a game link. Review the names and optional profile links before copying. Anyone with the link can replay it.</dd></div>
              <div><dt><Icon name="save" />Save replay</dt><dd>Keep a private study copy in Saved replays. Saving does not add the original game to your played-game totals.</dd></div>
              <div><dt><Icon name="publish" />Publish replay</dt><dd>Add an eligible game you played to your public profile. Sign in and enable your public profile first.</dd></div>
              <div><dt><Icon name="plus" />Play From Here</dt><dd>Start a new game from an earlier position against your selected bot. When finished, it appears in My games like any other game.</dd></div>
            </dl>
            <details className={styles.sharingDetails}>
              <summary><Icon name="help" /><span>What stays private?</span></summary>
              <dl className={styles.stateGuide}>
                <div><dt>My games</dt><dd>Games you played, including completed continuations.</dd></div>
                <div><dt>Saved replays</dt><dd>Private copies you deliberately kept. Local copies stay in this browser; a signed-in library can sync when connected.</dd></div>
                <div><dt>Public profile</dt><dd>Only the games you publish and the statistics you choose to share. The shared record is a dated My games snapshot, refreshed explicitly.</dd></div>
              </dl>
              <p>Opening a replay does not save or publish it. Removing a public listing does not remove portable links or copies other people already hold.</p>
            </details>
          </section>

          <section className={`${styles.panel} ${styles.examplesPanel}`} aria-labelledby="examples">
            <div className={styles.sectionIntro}>
              <p className="uiSectionLabel">Try it without signing in</p>
              <h2 className={styles.sectionAnchor} id="examples" tabIndex={-1}>Example replays</h2>
              <p>Two bot-generated games. Nothing is saved when you open one.</p>
            </div>
            <ul className={styles.replayExamples} aria-label="Example replays">
              {REPLAY_EXAMPLES.map((example) => (
                <li className={styles.replayExample} key={example.id}>
                  <div className={styles.exampleCopy}>
                    <h3>{example.title}</h3>
                    <p>{example.description}</p>
                    <span className={styles.exampleMeta}>Freestyle · {example.replay.moves.length} moves · {example.replay.viewPly === undefined ? "Final board" : `From move ${example.replay.viewPly}`}</span>
                  </div>
                  <Link aria-label={`Open ${example.title}`} className="uiAction uiActionSecondary" to={exampleReplayPath(example)}>
                    <Icon name="replay" /><span className="uiActionLabel">Open replay</span>
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        </div>
      </div>
    </main>
  );
}

function GuideDiagram({ label, model }: { label: string; model: BoardViewModel }) {
  return (
    <div className={styles.diagram} aria-label={label} role="img">
      <Board decorative model={model} />
    </div>
  );
}

function GuideFramePanel({
  frames,
  label,
}: {
  frames: Array<{ label: string; model: BoardViewModel }>;
  label: string;
}) {
  const frameLayout = frameGridClassName(frames.length);

  return (
    <article className={`${styles.example} ${styles.frameOnlyExample}`}>
      <div className={`${styles.frameGrid} ${frameLayout}`} aria-label={label} role="group">
        {frames.map((frame) => (
          <div key={frame.label} className={styles.frame}>
            <GuideDiagram
              label={`${label}: ${frame.label} board diagram.`}
              model={frame.model}
            />
            <p className={styles.frameLabel}>{frame.label}</p>
          </div>
        ))}
      </div>
    </article>
  );
}

function frameGridClassName(frameCount: number): string {
  if (frameCount === 2) {
    return styles.doubleFrameGrid;
  }
  if (frameCount === 4) {
    return styles.quadFrameGrid;
  }
  return styles.tripleFrameGrid;
}

function GuideTile({
  label,
  model,
  title,
  tone,
}: {
  label: string;
  model: BoardViewModel;
  title: string;
  tone: "counter" | "danger" | "imminent";
}) {
  return (
    <article className={`${styles.guideTile} ${styles[`${tone}Insert`]}`}>
      <GuideDiagram label={label} model={model} />
      <h3>{title}</h3>
    </article>
  );
}

function guideModelFromReplayFrame({
  currentPlayer,
  extraStones = [],
  focusMoves,
  omittedMoves = [],
  overlays,
  prefixPly,
  visibleMoves,
}: {
  currentPlayer: GuideSide;
  extraStones?: GuideReplayMove[];
  focusMoves: GuideReplayMove[];
  omittedMoves?: GuidePoint[];
  overlays: Array<GuidePoint & { role: GuideReplayOverlayRole }>;
  prefixPly: number;
  visibleMoves?: GuidePoint[];
}): BoardViewModel {
  const cells = emptyGuideCells(GUIDE_SEQUENCE_BOARD_SIZE);
  const boardOverlays: BoardOverlay[] = [];
  const omittedMoveKeys = new Set(omittedMoves.map((move) => `${move.row},${move.col}`));
  const visibleMoveKeys = visibleMoves
    ? new Set(visibleMoves.map((move) => `${move.row},${move.col}`))
    : null;

  for (const move of REPORT_SEQUENCE_MOVES.slice(0, prefixPly)) {
    if (omittedMoveKeys.has(`${move.row},${move.col}`)) {
      continue;
    }
    if (visibleMoveKeys && !visibleMoveKeys.has(`${move.row},${move.col}`)) {
      continue;
    }

    placeGuideSequenceStone(cells, move);
  }

  for (const move of extraStones) {
    placeGuideSequenceStone(cells, move);
  }

  for (const overlay of overlays) {
    const cell = localSequenceCell(overlay);

    if (!cell) {
      continue;
    }

    switch (overlay.role) {
      case "counter":
        boardOverlays.push({ cell, kind: "hint", role: "counterThreat" });
        break;
      case "counterEvidence":
        boardOverlays.push({ cell, kind: "evidence", role: "counterThreat" });
        break;
      case "escape":
        boardOverlays.push({
          cell,
          highlight: "corridorEntry",
          kind: "analysis",
          marker: "escape",
          side: currentPlayer,
        });
        break;
      case "forcedLoss":
        boardOverlays.push({ cell, kind: "analysis", marker: "forcedLoss" });
        break;
      case "immediate":
        boardOverlays.push({ cell, kind: "hint", role: "immediateThreat" });
        break;
      case "immediateEvidence":
        boardOverlays.push({ cell, kind: "evidence", role: "immediateThreat" });
        break;
      case "imminent":
        boardOverlays.push({ cell, kind: "hint", role: "imminentThreat" });
        break;
      case "imminentEvidence":
        boardOverlays.push({ cell, kind: "evidence", role: "imminentThreat" });
        break;
    }
  }

  for (const focusMove of focusMoves) {
    const localFocusMove = localSequenceCell(focusMove);

    if (localFocusMove) {
      cells[localFocusMove.row][localFocusMove.col] = null;
      boardOverlays.push({
        cell: localFocusMove,
        kind: "focusStone",
        side: focusMove.side,
      });
    }
  }

  return {
    boardSize: GUIDE_SEQUENCE_BOARD_SIZE,
    forbiddenMoves: [],
    interaction: { kind: "readonly" },
    overlays: boardOverlays,
    position: {
      cells,
      currentPlayer: currentPlayer === "black" ? 1 : 2,
      lastMove: null,
      moves: [],
      showSequenceNumbers: false,
      status: "playing",
    },
  };
}

function guideModelFromCells(source: GuideCell[][]): BoardViewModel {
  const cells: CellStone[][] = source.map((sourceRow) =>
    sourceRow.map((cell) => {
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

function emptyGuideCells(boardSize: number): CellStone[][] {
  return Array.from({ length: boardSize }, () =>
    Array.from({ length: boardSize }, () => null),
  );
}

function localSequenceCell(cell: { col: number; row: number }): { col: number; row: number } | null {
  const row = cell.row - GUIDE_SEQUENCE_ROW_OFFSET;
  const col = cell.col - GUIDE_SEQUENCE_COL_OFFSET;

  if (
    row < 0 ||
    row >= GUIDE_SEQUENCE_BOARD_SIZE ||
    col < 0 ||
    col >= GUIDE_SEQUENCE_BOARD_SIZE
  ) {
    return null;
  }

  return { col, row };
}

function placeGuideSequenceStone(cells: CellStone[][], move: GuideReplayMove): void {
  const cell = localSequenceCell(move);

  if (cell) {
    cells[cell.row][cell.col] = move.side === "black" ? 0 : 1;
  }
}
