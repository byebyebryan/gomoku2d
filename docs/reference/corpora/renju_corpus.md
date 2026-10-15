# Renju Corpus

Purpose: index the golden Renju legality cases used to validate Gomoku2D's
forbidden-move oracle.

This is a fixture index, not a generated board dump. Detailed extracted boards,
OCR/manual notes, and external reference validation live under
`gomoku-bot-lab/external/renjunet-advanced-examples/`.

## Runner

```sh
cd gomoku-bot-lab
cargo run -p gomoku-eval -- renju-rules \
  --report-json outputs/renju-rule-fixtures.json
```

The external RenjuNet/Piskvork reference check is:

```sh
python external/renjunet-advanced-examples/check_refs.py
```

## Corpus Summary

| Group | Count | Source | Purpose |
|---|---:|---|---|
| Core handwritten | 7 | Project/RIF-derived | Basic rules plus directional four-completion regression |
| RenjuNet advanced | 23 | Extracted from `renju.net/advanced` and checked against Piskvork | Real three/four edge cases, blocked branches, forbidden continuations, triple-combo variants |

Current expected status: Gomoku2D passes all 30 cases and all 240 rotated/reflected
legality/application probes; RenjuNet extracted labels
match the external Piskvork reference on all 23 advanced cases.

## Core Cases

| Case | Candidate | Expected | Purpose |
|---|---|---|---|
| `black_exact_five_legal` | Black `I8` | legal | Exact five wins before forbidden checks matter. |
| `black_overline_forbidden` | Black `E1` | forbidden | Six-in-row without exact five is forbidden. |
| `black_double_four_forbidden` | Black `H8` | forbidden | Two real fours are forbidden. |
| `black_double_three_forbidden` | Black `H8` | forbidden | Two real threes are forbidden. |
| `black_four_plus_three_legal` | Black `H8` | legal | `4+3` is legal for Black. |
| `unrelated_five_does_not_complete_a_four` | Black `H8` | legal | A completion's unrelated vertical five cannot make a horizontal overline into a second four. |
| `white_double_three_unrestricted` | White `H8` | legal | White has no forbidden-shape restrictions. |

## Advanced Coverage

The RenjuNet-derived cases cover:

- apparent double-threes where one branch is blocked;
- apparent double-threes where a continuation is itself forbidden;
- apparent double-fours where one four is not real;
- valid double-three/double-four/triple-combo forbiddens;
- exact examples from the public advanced tutorial, split into single-question
  fixtures.

When adding or changing cases, update fixture JSON and reference extraction
notes first, then regenerate the Gomoku2D report through the runner above.

## Legality Authority

The rules model is documented in [`../lab/renju_rules.md`](../lab/renju_rules.md).
Do not infer expected results from raw shape counts in this corpus doc. A case
is golden only after it is represented in fixture data and validated by the
external reference path or an explicit project rule decision.

The directional regression was checked against Piskvork at
`f76a43afb67861883c86f8bd22b1a4957c27f068`: H8 is legal (`reason=0`). Black is
E8/F8/G8/J8/I4/I5/I6/I7; White is A1/A3/A5/A7/C1/C3/C5/C7. After H8, EFGH has
a valid completion at D8. Completing FGH.J at I8 makes the horizontal six
EFGHIJ; its simultaneous vertical five does not make FGH.J another four.
The existing external checker wrapper was compiled against that pinned source
with this case as input; no reference implementation code was copied into core.
