# Performance Tuning

Purpose: current performance process and active optimization backlog for
`gomoku-bot-lab`.

Historical tuning notes live in
[`../archive/performance_history.md`](../archive/performance_history.md). This
file should stay short and current.

## Current State

- Rolling `ThreatView` is the default search backend.
- Scan-backed modes remain for fallback and parity validation.
- Pattern eval uses the rolling `PatternFrame` path in normal rolling mode.
- Tactical ordering keeps global immediate win/block checks, then gates more
  expensive annotations through local tactical potential.
- Corridor portal and leaf-extension experiments are retired.
- Current published reports come from the curated tournament command in
  [`tournament.md`](../reference/ops/tournament.md), then preset-triangle replay
  analysis.

## Benchmark Rules

1. Benchmark in release mode.
2. Use fixed scenario corpora before noisy self-play timings.
3. Compare medians/ranges, not a single fastest run.
4. Pair speed work with correctness checks.
5. Promote only stable commands/results into curated reports.
6. Keep scratch telemetry in ignored `gomoku-bot-lab/outputs/`.

## Useful Commands

```sh
cd gomoku-bot-lab
cargo test -p gomoku-core --test bench_scenarios
cargo bench -p gomoku-core --bench board_perf -- --noplot
cargo bench -p gomoku-bot --bench search_perf -- --noplot
cargo run -p gomoku-eval -- tactical-scenarios
cargo run -p gomoku-eval -- lethal-scenarios
```

For strength or report claims, use the tournament runbook instead of ad hoc
benchmarks.

## Active Backlog

- Re-check whether any remaining scan-backed paths matter in default rolling
  mode before removing them.
- Keep pattern-eval and tactical-ordering changes behavior-neutral unless a
  tournament explicitly proves the tradeoff.
- Add focused perf counters only when they answer a current tuning question;
  remove or demote stale counters from published reports.
- Avoid adding new lab suffixes for experiments unless the axis is likely to
  survive into repeated comparisons.
