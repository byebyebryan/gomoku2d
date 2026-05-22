import { applyWasmMove, createWasmBoard, readWasmWinningCells } from "../core/wasm_bridge";
import type { CellPosition } from "../game/types";
import { movesFromMoveCells, type SavedMatchV2 } from "../match/saved_match";

export function winningCellsFromCore(match: SavedMatchV2): CellPosition[] {
  const board = createWasmBoard(match.ruleset);

  try {
    for (const move of movesFromMoveCells(match.move_cells)) {
      const result = applyWasmMove(board, move.row, move.col);
      if (result?.error) {
        return [];
      }
    }

    return readWasmWinningCells(board).map((cell) => ({ ...cell }));
  } finally {
    board.free();
  }
}
