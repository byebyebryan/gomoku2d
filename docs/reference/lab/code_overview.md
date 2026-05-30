# Bot Lab Code Overview

This is the orientation map for `gomoku-bot-lab`. It explains crate ownership,
public APIs, and where to start for common changes. It is not a full Rust API
reference.

## Workspace Shape

`gomoku-bot-lab` is the Rust side of Gomoku2D: rules, bot search, replay
analysis, evaluation reports, native CLI tooling, and the wasm bridge.

```text
gomoku-core
  -> gomoku-lab-support
  -> gomoku-bot
      -> gomoku-analysis
      -> gomoku-eval
      -> gomoku-cli
      -> gomoku-wasm
```

`gomoku-core` is the rules foundation. Every other crate should depend on it,
not reimplement board, win, replay, or Renju legality behavior.
`gomoku-lab-support` is fixture/scenario support for lab consumers; `gomoku-core`
has a benchmark-only dev-dependency back to it.

## Crate Map

| Crate | Main files | Ownership |
|---|---|---|
| `gomoku-core` | `src/board.rs`, `src/rules.rs`, `src/renju.rs`, `src/replay.rs` | Board state, move legality, win detection, Renju forbidden checks, FEN, replay JSON |
| `gomoku-bot` | `src/lib.rs`, `src/search/*`, `src/tactical/mod.rs`, `src/frontier.rs`, `src/corridor.rs` | Bot trait, search bot, tactical facts, rolling threat view, corridor/proof helpers |
| `gomoku-analysis` | `src/lib.rs`, `src/replay.rs`, `src/trace.rs`, `src/annotations.rs`, `src/failure.rs`, `src/onset.rs` | Replay traceback, setup-corridor model, lethal onset, failure classification, per-frame annotations |
| `gomoku-eval` | `src/cli.rs`, `src/tournament.rs`, `src/report/*`, `src/analysis_batch/*`, `src/scenario.rs` | CLI harness, tournaments, reports, corpora/scenario checks, curated artifacts |
| `gomoku-lab-support` | `src/scenarios.rs` | Shared benchmark/tactical boards for tests, reports, and perf work |
| `gomoku-cli` | `src/main.rs` | Native playable match runner and replay exporter |
| `gomoku-wasm` | `src/lib.rs` | Wasm bridge for board, bot, threat snapshots, and replay analysis |

## Core Rules API

Start in `gomoku-core` for anything about legal play.

Important types:

- `RuleConfig` and `Variant`
- `Board`
- `Move`
- `GameResult`
- `Replay`

Important behavior:

- `Board::new(...)`
- `Board::apply_move(...)`
- `Board::is_legal_for(...)`
- `Board::current_player()`
- `Board::result()`
- `Board::to_fen()` / `Board::from_fen(...)`
- `Replay::to_json()` / `Replay::from_json(...)`

Renju forbidden checks live in `renju.rs`. They are deliberately deeper than
simple shape matching; they must stay aligned with the Renju corpus and external
validation notes.

## Bot API

The trusted in-process bot API is the `Bot` trait in `gomoku-bot/src/lib.rs`:

```rust
pub trait Bot: Send {
    fn name(&self) -> &str;
    fn choose_move(&mut self, board: &Board) -> Move;
    fn trace(&self) -> Option<serde_json::Value> { None }
}
```

This is for trusted native bots inside eval, CLI, and wasm. Do not treat it as a
remote/untrusted bot protocol. Future API-backed bots should use an adapter
layer, not this trait directly.

Current exported bot/config types:

- `RandomBot`
- `SearchBot`
- `SearchBotConfig`
- `CandidateSource`
- `MoveOrdering`
- `StaticEvaluation`
- `ThreatViewMode`
- `SafetyGate`
- `NullCellCulling`
- `CorridorProofConfig`

`lab_spec.rs` parses lab strings such as `search-d7+tactical-cap-8+pattern-eval`.
Product presets should resolve to known config/specs; they are not the same as
the lab parser contract.

## Search Bot Pipeline

`gomoku-bot/src/search/` is split by responsibility:

| Module | Role |
|---|---|
| `config.rs` | Search config, defaults, and public knobs |
| `state.rs` | Search-time board/frontier/TT state and apply/undo lifecycle |
| `candidates.rs` | Candidate generation, legality filtering, child caps |
| `evaluation.rs` | Static evaluation and pattern evaluation |
| `timing.rs` | CPU budget and timing helpers |
| `metrics.rs` | Trace/metric counters |
| `corridor_proof.rs` | Optional root candidate proof pass |
| `mod.rs` | `SearchBot` orchestration and tests |

Current search model:

- negamax with alpha-beta pruning
- iterative deepening
- transposition table with an optional bounded entry cap
- tactical ordering
- rolling threat-view backend
- pattern evaluation
- optional corridor-proof pass, currently a small root proof layer rather than a
  general portal search

## Tactical And Threat APIs

`gomoku-bot/src/tactical/mod.rs` is the shared tactical fact source. Search,
analysis, wasm hints, and report rendering should consume it rather than each
owning shape logic.

Key concepts:

- immediate threats: winning moves and four replies
- imminent threats: forcing-three replies
- counter-threats: creating an immediate threat while responding to an
  imminent threat
- lethal threats: open-four / combo coverage where one reply cannot cover all
  routes
- evidence stones: existing stones that explain why a hinted cell matters

`frontier.rs` owns the rolling threat view used by search and wasm snapshots.
Scan-backed paths remain useful as correctness references and fallback, but new
hot-path features should try to consume rolling facts first.

## Corridor And Replay Analysis

`gomoku-analysis` owns replay analysis. It is shared by static reports and wasm
replay analysis.

Main exports:

- `analyze_replay(...)`
- `ReplayAnalysisSession`
- `replay_frame_annotations_for_analysis(...)`
- `visible_defender_reply_candidates(...)`
- `defender_reply_candidates(...)`
- `analyze_defender_reply_options(...)`
- `analysis_options_from_json(...)`
- `ReplayAnalysisStepEnvelope`

The model walks backward from a decisive replay, identifies lethal onset and the
setup corridor before it, and classifies failure modes such as missed response,
missed lethal prevention, and missed escape. It assumes the core/tactical
modules provide the authoritative rule and threat facts.

## Eval And Report APIs

`gomoku-eval` is both a library crate and a CLI app. The CLI dispatcher is in
`src/cli.rs`; report and scenario logic live in modules so tests can call them
without shelling out.

Important commands:

- `tournament`
- `report-json`
- `analyze-report-replays`
- `analyze-replay-batch`
- `tactical-scenarios`
- `lethal-scenarios`
- `analysis-fixtures`
- `renju-rules`

Curated artifacts:

- `reports/lab/bot-report.json`
- `reports/lab/analysis-report.json`

Scratch artifacts belong in ignored `gomoku-bot-lab/outputs/`.

## Wasm Bridge API

`gomoku-wasm` is a bridge only. It should translate data across the JS boundary;
it should not own game logic.

Current exported classes/functions:

- `init()`
- `WasmBoard`
- `WasmBoard.createWithVariant(...)`
- `WasmBoard.applyMove(row, col) -> JSON`
- `WasmBoard.threatSnapshot() -> JSON`
- `WasmBoard.winningCells() -> JSON`
- `WasmBoard.toFen()` / `WasmBoard.fromFenWithVariant(...)`
- `WasmBot.createFromSpec(specJson)`
- `WasmBot.chooseMove(board) -> JSON`
- `WasmReplayAnalyzer.createFromReplayJson(replayJson, optionsJson)`
- `WasmReplayAnalyzer.step(maxWorkUnits) -> JSON`

The TypeScript side validates these JSON payloads in
`gomoku-web/src/core/wasm_bridge.ts` and `gomoku-web/src/replay/*protocol.ts`.
Any wasm payload change must update both sides.

## Validation Corpora

The lab has two kinds of test data:

- code-defined scenarios in `gomoku-lab-support/src/scenarios.rs`
- documented corpora under `docs/reference/corpora/`

Use scenario/corpus tests to validate behavior categories. Avoid adding
single-replay debug tests unless the replay is converted into a general
condition or promoted to a curated fixture with a clear reason.

## Common Change Map

| Change | Start here | Also check |
|---|---|---|
| Change move legality or Renju rules | `gomoku-core/src/renju.rs`, `board.rs` | Renju corpus, wasm threat snapshots, search legality metrics |
| Add or tune a search config | `gomoku-bot/src/search/config.rs`, `lab_spec.rs` | web `bot_config.ts`, wasm `parse_bot_spec`, report labels |
| Change tactical shape semantics | `gomoku-bot/src/tactical/mod.rs` | tactical corpus, corridor analysis, wasm hints, report annotations |
| Change rolling threat facts | `gomoku-bot/src/frontier.rs` | scan parity tests, search perf benches, wasm threat snapshots |
| Change replay-analysis logic | `gomoku-analysis/src/*` | analysis fixtures, report generation, wasm `ReplayAnalysisSession`, web overlays |
| Change report presentation | `gomoku-eval/src/report/*`, `analysis_batch/*` | web report publishing scripts and Playwright report smoke |
| Add a new CLI command | `gomoku-eval/src/cli.rs` | ops docs, tests in CLI module |
| Change wasm payloads | `gomoku-wasm/src/lib.rs` | web bridge validators, TypeScript protocol types, wasm tests |

## Verification

For Rust-side changes:

```sh
cd /home/bryan/code/gomoku2d/gomoku-bot-lab
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

For wasm-boundary changes:

```sh
cd /home/bryan/code/gomoku2d
/home/bryan/.cargo/bin/wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd /home/bryan/code/gomoku2d/gomoku-web
npm run typecheck
npm test
GOMOKU_BASE_PATH=/ npm run build
```

For report/model changes, refresh or smoke the relevant report command before
shipping. The release runbook owns the full curated report flow.
