import { WasmBoard } from "../core/wasm_bridge";
import type { CellPosition } from "../game/types";
import { movesFromMoveCells, type SavedMatchV2 } from "../match/saved_match";

export function winningCellsFromCore(match: SavedMatchV2): CellPosition[] {
  const board = WasmBoard.createWithVariant(match.ruleset);

  try {
    for (const move of movesFromMoveCells(match.move_cells)) {
      const result = board.applyMove(move.row, move.col) as { error?: unknown };
      if (result?.error) {
        return [];
      }
    }

    return (board.winningCells() as CellPosition[]).map((cell) => ({ ...cell }));
  } finally {
    board.free();
  }
}
