# Renju Rules Model

Purpose: document the Renju legality model Gomoku2D currently uses for Black
forbidden moves.

## Sources

Primary rule text:

- RIF international rules: <https://www.renju.net/rifrules/>
- RenjuNet advanced forbidden-move tutorial: <https://www.renju.net/advanced/>

Supporting implementation references:

- Piskvork Renju fork: <https://github.com/wind23/piskvork_renju>
- Rapfi: <https://github.com/dhbloo/rapfi>
- `renju_forbid`: <https://github.com/realjustice/renju_forbid>

Reference code is executable evidence, not source to copy. Piskvork and Rapfi
are GPL-licensed; the project uses small external wrappers for validation rather
than porting their code.

## Semantics

Only Black has forbidden moves. White can win with a five or longer line and
does not have overline, double-four, or double-three restrictions.

For Black, Gomoku2D applies this order:

1. Exact five-in-a-row is a legal win.
2. Otherwise, overline is forbidden.
3. Otherwise, more than one real four is forbidden.
4. Otherwise, more than one real three after RIF 9.3 filtering is forbidden.
5. Otherwise, the move is legal.

This matches the RIF wording that a forbidden shape loses only when Black does
not also make five in a row, and matches the Piskvork `foulr` check order.

## Real Shapes

A raw window count is not enough for Renju forbidden rules.

A real four must have at least one legal completion to exact five. Apparent
fours do not count when every completion is blocked, overline, or forbidden.

A straight four is an unbroken four with two different legal exact-five
completions. For Black, both completions must be legal.

A real three is a line where Black can add one stone, without making five
immediately, to create a legal straight four. Apparent threes are dead when all
extensions fail to create a straight four or land on forbidden points.

Double-three uses the recursive RIF 9.3 test: only threes with at least one
legal path to a straight four count.

## Implementation

`Board` is the rules authority:

- `is_legal_for_color`
- `apply_move`
- `forbidden_moves_for_current_player`
- tactical and wasm threat views that query legal/effective facts

Internally, the old raw shape-count detector has been replaced by a dedicated
Renju legality oracle:

```text
renju_forbidden_reason(board, black_move) -> Option<ForbiddenReason>
```

where:

```text
ForbiddenReason = Overline | DoubleFour | DoubleThree
```

The oracle checks exact five and overline directly. Real four and real three
classification then ask whether their completions/extensions are legal under the
same oracle. Recursive double-three checks use a guard and fail open if they
cannot resolve; false-forbidden is worse than missed-forbidden for normal play,
and unresolved cases should become fixtures.

Cheap necessary-condition guards such as `can_be_renju_forbidden_at` are
performance details only. They must not change oracle output and are not bot
configuration knobs.

## Consumers

Search, corridor analysis, wasm hints, and report rendering may use raw tactical
shape facts for diagnostics, but every Renju-active Black continuation must
cross the legality oracle before it receives tactical credit.

Consumer rules:

- raw Black gains/completions are diagnostics, not legal threats;
- forbidden Black squares can be useful proof evidence for analysis;
- White threats can become stronger when Black's natural reply is forbidden;
- no consumer should locally infer double-three or double-four by raw window
  counts.

## Corpus And Validation

The promoted golden corpus is documented in
[`renju_corpus.md`](renju_corpus.md). It contains:

- 6 handwritten core cases;
- 23 RenjuNet advanced tutorial cases covering apparent three/four edge cases.

Run the Gomoku2D corpus check with:

```sh
cargo run -p gomoku-eval -- renju-rules \
  --report-json outputs/renju-rule-fixtures.json
```

The RenjuNet extraction/reference project lives in
[`../gomoku-bot-lab/external/renjunet-advanced-examples/`](../gomoku-bot-lab/external/renjunet-advanced-examples/).
That folder owns OCR/manual extraction notes, fixture JSON, and external
Piskvork validation. It deliberately does not run Gomoku2D checks; those belong
to the eval/core test surfaces that consume the corpus.

Run the external reference check with:

```sh
python gomoku-bot-lab/external/renjunet-advanced-examples/check_refs.py
```

Current status:

- RenjuNet extracted labels match Piskvork: 23/23.
- Gomoku2D `renju-rules` corpus check passes: 29/29.

## Follow-Up Validation

After changing Renju legality:

- run `cargo test -p gomoku-core`;
- run `cargo run -p gomoku-eval -- renju-rules`;
- run tactical and lethal scenario suites if threat behavior changed;
- run replay-analysis smoke if corridor/analysis behavior changed;
- regenerate bot/analysis reports only if behavior changes materially.
