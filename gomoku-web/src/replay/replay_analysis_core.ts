import { WasmBoard, WasmReplayAnalyzer } from "../core/wasm_bridge";
import { movesFromMoveCells, type SavedMatchStatus, type SavedMatchV2 } from "../match/saved_match";

const CORE_REPLAY_SCHEMA_VERSION = 1;
const CORE_REPLAY_BOARD_SIZE = 15;
const CORE_REPLAY_WIN_LENGTH = 5;
const CORE_REPLAY_HASH_ALGORITHM = "xorshift64";
const CORE_REPLAY_HASH_SEED = "16045690984503098046";
const MOVE_FILES = "ABCDEFGHIJKLMNO";

interface CoreReplayRules {
  board_size: typeof CORE_REPLAY_BOARD_SIZE;
  variant: SavedMatchV2["ruleset"];
  win_length: typeof CORE_REPLAY_WIN_LENGTH;
}

interface CoreReplayHashAlgo {
  algorithm: typeof CORE_REPLAY_HASH_ALGORITHM;
  seed: string;
}

interface CoreReplayMove {
  hash: string;
  mv: string;
  time_ms: number;
}

interface CoreReplay {
  black: string;
  hash_algo: CoreReplayHashAlgo;
  moves: CoreReplayMove[];
  result: "black_wins" | "draw" | "white_wins";
  rules: CoreReplayRules;
  schema_version: typeof CORE_REPLAY_SCHEMA_VERSION;
  white: string;
}

export interface ReplayAnalysisOptions {
  maxDepth?: number;
  maxScanPlies?: number | null;
}

function moveNotation(row: number, col: number): string {
  const file = MOVE_FILES[col];
  if (!file) {
    throw new Error("Saved match contains a move outside the core replay board.");
  }

  return `${file}${row + 1}`;
}

function replayResult(status: SavedMatchStatus): CoreReplay["result"] {
  switch (status) {
    case "black_won":
      return "black_wins";
    case "draw":
      return "draw";
    case "white_won":
      return "white_wins";
  }
}

function assertUnsignedIntegerToken(value: string, label: string): void {
  if (!/^\d+$/.test(value)) {
    throw new Error(`Core replay ${label} must be an unsigned integer token.`);
  }
}

function stringifyReplayWithExactIntegers(replay: CoreReplay): string {
  const tokens = [
    replay.hash_algo.seed,
    ...replay.moves.map((move) => move.hash),
  ];

  for (const [index, token] of tokens.entries()) {
    assertUnsignedIntegerToken(token, index === 0 ? "seed" : "hash");
  }

  let json = JSON.stringify(replay);
  json = json.replace(`"${CORE_REPLAY_HASH_SEED}"`, CORE_REPLAY_HASH_SEED);

  for (const move of replay.moves) {
    json = json.replace(`"${move.hash}"`, move.hash);
  }

  return json;
}

export function savedMatchToReplayJson(match: SavedMatchV2): string {
  const board = WasmBoard.createWithVariant(match.ruleset);

  try {
    const moves = movesFromMoveCells(match.move_cells).map((move) => {
      const result = board.applyMove(move.row, move.col) as { error?: string | null };
      if (result.error) {
        throw new Error(`Saved match cannot be replayed by core rules: ${result.error}`);
      }

      const hash = board.hashString();
      return {
        hash,
        mv: moveNotation(move.row, move.col),
        time_ms: 0,
      };
    });

    return stringifyReplayWithExactIntegers({
      black: match.player_black.display_name,
      hash_algo: {
        algorithm: CORE_REPLAY_HASH_ALGORITHM,
        seed: CORE_REPLAY_HASH_SEED,
      },
      moves,
      result: replayResult(match.status),
      rules: {
        board_size: CORE_REPLAY_BOARD_SIZE,
        variant: match.ruleset,
        win_length: CORE_REPLAY_WIN_LENGTH,
      },
      schema_version: CORE_REPLAY_SCHEMA_VERSION,
      white: match.player_white.display_name,
    });
  } finally {
    board.free();
  }
}

export function replayAnalysisOptionsJson(options: ReplayAnalysisOptions = {}): string {
  return JSON.stringify({
    ...(options.maxDepth === undefined ? {} : { max_depth: options.maxDepth }),
    ...(options.maxScanPlies === undefined ? {} : { max_scan_plies: options.maxScanPlies }),
  });
}

export function createReplayAnalyzer(
  match: SavedMatchV2,
  options: ReplayAnalysisOptions = {},
): WasmReplayAnalyzer {
  return WasmReplayAnalyzer.createFromReplayJson(
    savedMatchToReplayJson(match),
    replayAnalysisOptionsJson(options),
  );
}
