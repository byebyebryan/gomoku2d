import { WasmBoard } from "../core/wasm_bridge";
import type { CellPosition } from "../game/types";
import { movesFromMoveCells } from "../match/saved_match";
import type { GuestSavedMatch } from "../profile/guest_profile_store";

export function winningCellsFromCore(match: GuestSavedMatch): CellPosition[] {
  const board = WasmBoard.createWithVariant(match.variant);

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
