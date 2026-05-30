# Game Analysis

Purpose: define how finished-game replay analysis applies corridor search.

This doc owns replay-specific attribution and UI/report meaning. Tactical facts
live in [`tactical_shapes.md`](tactical_shapes.md), lethal onset in
[`lethal_threats.md`](lethal_threats.md), and corridor proof semantics in
[`corridor_search.md`](corridor_search.md).

Source of truth in code:

- shared analyzer: `gomoku-bot-lab/gomoku-analysis/`
- report exporter: `gomoku-bot-lab/gomoku-eval/src/analysis*`
- wasm bridge: `gomoku-bot-lab/gomoku-wasm/`

## Product Question

Given a finished decisive replay, explain the final collapse:

- when the winner first had a forced corridor;
- when the loser first faced lethal onset;
- the losing side's latest escape;
- whether the losing side missed a direct response, missed lethal prevention,
  missed an earlier escape, or was simply forced inside the modeled corridor.

This is not best-move analysis and not a full solver. If the bounded model sees
a legal alternative it cannot prove losing, that is a possible escape.

## Replay Flow

The analyzer walks backward from the terminal move along the actual replay:

1. identify the final forced interval ending at the winning move;
2. identify lethal onset when available;
3. derive the setup corridor from forced start through onset;
4. inspect losing-side decision points backward;
5. enumerate named legal alternatives using corridor candidate rules;
6. stop at the latest escape, possible escape, or model boundary.

Winner-side actual moves are normally kept as the actual proof spine. The
important question is what the losing side could have done differently.

## Key Spans

| Span | Meaning |
|---|---|
| Terminal move | Actual final winning move. |
| Lethal onset | First frame where the loser has no legal reply avoiding terminal or known-lethal continuation. |
| Setup corridor | Forced sequence that led into onset. |
| Lethal tail | Conversion after onset. |
| Last escape | Latest losing-side frame with a legal move that exits the detected setup corridor. |

## Failure Labels

Labels describe the losing side's failure mode inside the analyzer model:

| Label | Meaning |
|---|---|
| Missed response | Loser played outside the required immediate/imminent response set. |
| Missed lethal prevention | Right before onset, loser had a legal way to avoid onset but chose another valid response. |
| Missed escape | Earlier in the setup corridor, loser had a legal escape from the detected corridor. |
| Forced loss | No modeled legal escape was found before the scan boundary. |
| Unclear | The analyzer hit a guard or model boundary before a concrete explanation. |

Do not use corridor length alone as a mistake classifier. That old shortcut was
removed because it conflated tactical miss, forced sequence length, and proof
budget.

## Browser And Report Contract

The static report and browser replay analyzer should agree on candidate
semantics:

- immediate tier suppresses imminent tier;
- multiple threats in the same tier all survive;
- actual losing-side moves are shown as actual moves, not reprobed alternatives;
- forbidden Black candidates can be marked but are not legal alternatives;
- remaining legal non-actual candidates are probed and listed;
- evidence cells may be shown to explain why a candidate is a threat.

The report is the richer debugging surface. The browser UI should stay concise:
status text, timeline spans, board hints, and replay navigation.

## Model Settings

Current replay analysis uses:

- reply policy: `corridor_replies`;
- `max_depth`: corridor proof depth, not broad minimax depth;
- `max_scan_plies`: backward replay scan window.

Every published report must include these model settings so proof claims remain
bounded and reproducible.

## Validation

Use both code paths when changing analysis semantics:

```sh
cd gomoku-bot-lab
cargo test -p gomoku-analysis
cargo run --release -p gomoku-eval -- analyze-report-replays \
  --report ../reports/lab/bot-report.json \
  --selector preset-triangle \
  --published-report-json outputs/analysis-smoke.json \
  --max-depth 4 \
  --max-scan-plies 64
```

For UI-facing changes, compare the browser replay annotations against the report
for the same move list before trusting the wasm bridge.
