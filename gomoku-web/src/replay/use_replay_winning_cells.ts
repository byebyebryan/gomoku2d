import { useEffect, useState } from "react";

import type { CellPosition } from "../game/types";
import type { SavedMatchV2 } from "../match/saved_match";

export function useReplayWinningCells(match: SavedMatchV2 | null, moveIndex: number): CellPosition[] {
  const [coreWinningCells, setCoreWinningCells] = useState<CellPosition[]>([]);

  useEffect(() => {
    let cancelled = false;

    if (!match || moveIndex !== match.move_count) {
      setCoreWinningCells([]);
      return () => {
        cancelled = true;
      };
    }

    void import("./local_replay_core")
      .then(({ winningCellsFromCore }) => {
        if (!cancelled) {
          setCoreWinningCells(winningCellsFromCore(match));
        }
      })
      .catch(() => {
        if (!cancelled) {
          setCoreWinningCells([]);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [match, moveIndex]);

  return coreWinningCells;
}
