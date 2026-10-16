// Run with node --experimental-wasm-modules. Validate fixtures with the same
// freshly built core used by Vite, without adding a test entry to the app.
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { WasmBoard } from "gomoku-wasm";

/** @typedef {{id: string, variant: 'freestyle' | 'renju', matchIndex: number, prefix?: number}} Entry */
/** @typedef {{match_index: number, move_count: number, move_cells: number[], result: 'black_won' | 'white_won' | 'draw'}} Game */
const manifestBytes = await readFile(new URL("../perftests/workloads.json", import.meta.url));
/** @type {{source: string, sourceSha256: string, positions: Entry[], replays: Entry[]}} */
const manifest = JSON.parse(manifestBytes.toString());
const sourceBytes = await readFile(new URL(`../${manifest.source}`, import.meta.url));
/** @param {import('node:crypto').BinaryLike} bytes */
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
if (sha256(sourceBytes) !== manifest.sourceSha256) throw new Error("Profile source changed; review the frozen manifest");
/** @type {{matches: Game[]}} */
const source = JSON.parse(sourceBytes.toString());

/** @param {Entry} entry @param {number | undefined} [prefix] */
function validate(entry, prefix) {
  const game = source.matches.find((candidate) => candidate.match_index === entry.matchIndex);
  if (!game || game.move_cells.length !== game.move_count) throw new Error(`Missing or malformed game: ${entry.id}`);
  const limit = prefix ?? game.move_count;
  if (prefix !== undefined && (prefix < 1 || prefix >= game.move_count)) throw new Error(`Invalid prefix: ${entry.id}`);
  const board = WasmBoard.createWithVariant(entry.variant);
  try {
    for (let ply = 0; ply < limit; ply += 1) {
      if (board.result() !== "ongoing") throw new Error(`Early finish: ${entry.id} at ${ply}`);
      const cell = game.move_cells[ply];
      const result = JSON.parse(board.applyMove(Math.floor(cell / 15), cell % 15));
      if (result.error) throw new Error(`Illegal ${entry.id} ply ${ply + 1}: ${result.error}`);
    }
    const expected = prefix === undefined ? { black_won: "black", white_won: "white", draw: "draw" }[game.result] : "ongoing";
    if (board.result() !== expected) throw new Error(`Wrong result: ${entry.id}: ${board.result()} != ${expected}`);
    return prefix === undefined
      ? { ...entry, moveCells: game.move_cells, result: game.result }
      : { ...entry, fen: board.toFen(), legalMoves: JSON.parse(board.legalMoves()) };
  } finally { board.free(); }
}

console.log(JSON.stringify({
  manifestSha256: sha256(manifestBytes), sourceSha256: manifest.sourceSha256,
  wasmSha256: sha256(await readFile(new URL("./gomoku_wasm_bg.wasm", import.meta.resolve("gomoku-wasm")))),
  positions: manifest.positions.map((entry) => validate(entry, entry.prefix)),
  replays: manifest.replays.map((entry) => validate(entry)),
}));
