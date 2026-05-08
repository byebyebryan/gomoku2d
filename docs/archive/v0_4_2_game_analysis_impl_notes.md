# `v0.4.2` Game Analysis Implementation Notes

Status: archived implementation/history notes extracted from
[`../game_analysis.md`](../game_analysis.md) during `v0.4.2` release prep.

Canonical current docs:

- [`../corridor_search.md`](../corridor_search.md) — strategic corridor-search
  model.
- [`../game_analysis.md`](../game_analysis.md) — current replay analyzer
  contract and report shape.

## Why This Was Archived

`game_analysis.md` had grown into a mix of product contract, corridor-search
design, report schema, implementation telemetry, and temporary work notes. The
root doc should describe the current replay-analysis contract. This archive
keeps the implementation history and useful experiment notes without making the
canonical doc read like a changelog.

## Completed Lab Slices

1. Locked terminology, proof statuses, model bounds, and output shape.
2. Added finished-game prefix fixtures covering immediate wins, short forced
   lines, conversion errors, missed defenses, missed wins, unknown results, and
   Renju legality edges.
3. Built a CLI/lab analyzer that finds final win, proof intervals, last chance,
   and a bounded principal line for finished games.
4. Added proof-detail output and visual HTML report rendering for debugging.
5. Added batch replay analysis for replay directories and compact tournament
   reports.
6. Published a curated top-two analysis report from the current bot report under
   `/analysis-report/`.

## Lab Implementation Surface

The first implementation lives in `gomoku-eval` and is intentionally narrow:

- `gomoku_eval::analysis` defines model/result types and the bounded proof
  walker.
- `gomoku-eval analyze-replay --input <replay.json>` emits JSON analysis.
- `gomoku-eval analyze-replay-batch --replay-dir <dir>` analyzes replay JSON
  directories and emits grouped JSON/HTML reports.
- `gomoku-eval analyze-report-replays --report <report.json>` samples compact
  tournament-report matches, reconstructs replay objects in memory, and analyzes
  them without first writing replay JSON files.
- `gomoku-eval analysis-fixtures` runs curated replay fixtures and prints
  expected-vs-actual labels.

The active reply model exposes corridor-valid replies: direct threat defenses,
imminent-threat replies, defender immediate wins, counter-threats, and
forbidden cost squares in branch evidence. It is not a full threat-space search.

## Visual Proof Frames

`--include-proof-details` adds previous-prefix and final-forced-start proof
snapshots, reply classification, principal-line notation, compact board
snapshots, and visual decision frames.

Important conventions:

- Visual frames render pre-move decision states backward from the winning ply
  through the final forced interval.
- Do not add a separate `after ply N` boundary frame; every visual frame should
  use the `before ply X` convention.
- Actual replay moves use rings and are not probed/labeled as alternate branch
  outcomes.
- Outer hints explain why a square is shown: immediate win, immediate threat,
  imminent threat response, counter threat, or corridor-entry denial.
- Marker characters explain alternate reply outcomes: `L` forced loss, `E`
  confirmed escape, `P` possible escape, `!` immediate loss, `?` unknown.
- Proof branch evidence such as aggregate cost squares, forbidden costs, and
  principal-line moves stays in textual proof snapshots so the board does not
  imply nested branch moves are current gameplay hints.

## Useful Checkpoints And Telemetry

- After the single-depth corridor refactor, an 8-game top-two smoke run passed
  with `8 analyzed / 8 total` and `0 failed`. Before the loss-category pass,
  the root-detail split was `7` missed defenses and `1` draw/ongoing entry.
- After the inclusive-span loss-category pass, a top-two 64-game audit passed
  with `64 analyzed / 64 total` and `0 failed` in about `61s` wall time. It
  classified the sample as `8` mistakes, `25` tactical errors, `27` strategic
  losses, `3` unclear entries, and `1` draw/ongoing entry.
- A later implementation snapshot passed with `64 analyzed / 64 total` and
  `0 failed` in about `49s` total elapsed time. It classified the decisive
  sample as `54` strategic losses, `5` missed defenses, and `4` unclear
  proof-limit entries, plus `1` draw/ongoing entry. Treat this as pre-refactor
  telemetry only.
- After the scan-cap refactor, the top-two 64-game checkpoint with default
  `max_scan_plies=64` passed with `64 analyzed / 64 total` and `0 failed`. It
  resolved every decisive game: `3` mistakes, `25` tactical errors, `35`
  strategic losses, and `1` draw/ongoing game. The longest corridor was
  `match_1735`, which needed `41` analyzed prefixes to classify a forced
  interval from ply `53` to `92`. A cap of `32` still left that game as
  `outside_scan_window`, so `64` keeps a power-of-two headroom above this case.

## Notable Debugging Lessons

- In the first top-two sample, the ply-14 frame marked `G4`, `G7`, and `G9` as
  imminent-defense replies that all ended in forced loss, with `G7` also marked
  as the actual replay move. Offensive counter-threat replies were marked
  separately.
- `I11` still lost because Black answered at `I10` and re-entered the narrow
  forced line.
- `I10` was the harder sibling: White occupied the square that was the actual
  final Black move, Black had to answer at `I11`, and the proof had to
  rediscover a longer forced line.
- Decision-critical deepening was tried, but the cost/benefit was poor: the
  first 8-game sample grew from about `16s` to about `62s`, `match_1729` `I10`
  still ended as `possible_escape`, and the 64-game run was expensive enough to
  interrupt. Keep proof-detail audits at base depth until corridor search has
  better pruning, memoization, or a narrower transition model.

## Retired Or Rejected Directions

- Before the corridor-exit pivot, the 64-game sampled checkpoint produced
  `63` proof-limit hits and `1` draw/ongoing game. The old bounded-scan retry
  reduced scan cutoffs, but its "one chunk plus another" semantics were
  confusing and could waste work.
- Narrow-reply experiments were either too narrow to explain real replay
  prefixes or too expensive when they widened into local-threat scans.
- A stricter double-threat-only trigger was fast but did not improve the sampled
  report. A broader one-or-two-threat trigger improved coverage but became
  expensive.
- Adding `BrokenFour` facts and diagnostic `BrokenThree` facts did not change
  the same 8-game smoke result materially.
- Temporarily treating `BrokenThree` as forcing was much slower and was narrowed
  back to diagnostic-only.
- Raising broad corridor depth is not the next practical move. The 8-game smoke
  at corridor depth `3` still left `7` unresolved entries and took roughly
  `190s` wall-clock / `626s` summed per-entry time, versus about `2.4s` /
  `7.4s` for depth `2`.
- Increasing the old forced-extension-only budget did not help the smoke matrix.
  The dominant issue was defender breadth and corridor model quality, not a
  second independent budget.

## Historical Commands

```bash
cargo run -p gomoku-eval -- analyze-replay \
  --input outputs/replays/match_001.json \
  --output outputs/analysis_001.json \
  --max-depth 4

cargo run -p gomoku-eval -- analysis-fixtures \
  --report-json outputs/analysis_fixtures.json \
  --report-html outputs/analysis_fixtures.html \
  --max-depth 4

cargo run -p gomoku-eval -- analyze-replay-batch \
  --replay-dir outputs/replays \
  --report-json outputs/analysis_batch.json \
  --report-html outputs/analysis_batch.html \
  --max-depth 4

cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report reports/latest.json \
  --entrant-a search-d7+tactical-cap-8+pattern-eval \
  --entrant-b search-d5+tactical-cap-8+pattern-eval \
  --sample-size 8 \
  --report-json outputs/analysis/top2_smoke.json \
  --report-html outputs/analysis/top2_smoke.html \
  --max-depth 4 \
  --max-scan-plies 8
```

Use the report-sampled 8-game smoke path while tuning analyzer output or proof
logic. Run a full 64-game head-to-head analysis only for checkpoint reports.
`--max-scan-plies 8` was the practical override for fast iteration; the CLI
default is `64`, which became the `v0.4.2` checkpoint setting.
