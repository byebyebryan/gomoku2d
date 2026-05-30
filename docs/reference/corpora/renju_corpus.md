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
| Core handwritten | 6 | Project/RIF-derived | Basic exact-five, overline, double-four, double-three, legal `4+3`, White unrestricted behavior |
| RenjuNet advanced | 23 | Extracted from `renju.net/advanced` and checked against Piskvork | Real three/four edge cases, blocked branches, forbidden continuations, triple-combo variants |

Current expected status: Gomoku2D passes all 29 cases; RenjuNet extracted labels
match the external Piskvork reference on all 23 advanced cases.

## Core Cases

| Case | Candidate | Expected | Purpose |
|---|---|---|---|
| `black_exact_five_legal` | Black `I8` | legal | Exact five wins before forbidden checks matter. |
| `black_overline_forbidden` | Black `E1` | forbidden | Six-in-row without exact five is forbidden. |
| `black_double_four_forbidden` | Black `H8` | forbidden | Two real fours are forbidden. |
| `black_double_three_forbidden` | Black `H8` | forbidden | Two real threes are forbidden. |
| `black_four_plus_three_legal` | Black `H8` | legal | `4+3` is legal for Black. |
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
