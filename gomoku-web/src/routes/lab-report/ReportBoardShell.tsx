import { Fragment, type CSSProperties, type ReactNode } from "react";

import styles from "../ReportRoute.module.css";

export function ReportBoardShell({
  ariaLabel,
  boardKind,
  boardSize,
  cellSize,
  labelSize,
  renderCell,
}: {
  ariaLabel: string;
  boardKind: "analysis" | "finished";
  boardSize: number;
  cellSize: number;
  labelSize: number;
  renderCell: (cell: number, notation: string) => ReactNode;
}) {
  const gridSpan = Math.max(0, boardSize - 1) * cellSize + 1;
  const columnLabels = Array.from({ length: boardSize }, (_, col) =>
    String.fromCharCode("A".charCodeAt(0) + col),
  );
  const style = {
    "--proof-cell-size": `${cellSize}px`,
    "--proof-label-size": `${labelSize}px`,
    "--proof-grid-span": `${gridSpan}px`,
    gridTemplateColumns: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
    gridTemplateRows: `var(--proof-label-size) repeat(${boardSize}, var(--proof-cell-size)) var(--proof-label-size)`,
  } as CSSProperties;

  return (
    <div
      aria-label={ariaLabel}
      className={styles.proofBoard}
      data-proof-board={boardKind === "analysis" ? "analysis" : undefined}
      data-report-board={boardKind === "finished" ? "finished" : undefined}
      role="img"
      style={style}
    >
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {columnLabels.map((label) => (
        <div className={`${styles.proofCoordinate} ${styles.proofCoordinateTop}`} key={`col-${label}`}>
          {label}
        </div>
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {Array.from({ length: boardSize }, (_, row) => (
        <Fragment key={`row-${row}`}>
          <div className={`${styles.proofCoordinate} ${styles.proofCoordinateLeft}`}>{row + 1}</div>
          {Array.from({ length: boardSize }, (_, col) => {
            const cell = row * boardSize + col;
            const notation = `${String.fromCharCode("A".charCodeAt(0) + col)}${row + 1}`;
            return renderCell(cell, notation);
          })}
          <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
        </Fragment>
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
      {Array.from({ length: boardSize }, (_, col) => (
        <div
          className={`${styles.proofCoordinate} ${styles.proofCorner}`}
          key={`bottom-${col}`}
          aria-hidden="true"
        />
      ))}
      <div className={`${styles.proofCoordinate} ${styles.proofCorner}`} aria-hidden="true" />
    </div>
  );
}
