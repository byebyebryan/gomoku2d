# RenjuNet Advanced Extraction

Purpose: preserve the extracted RenjuNet advanced forbidden-move examples,
convert them into fixture data, and validate the expected labels with external
reference code.

Primary source: <https://www.renju.net/advanced/>

This lives under `gomoku-bot-lab/external/` rather than top-level `docs/`
because it behaves like a small extraction/reference project: source data,
machine-readable fixture data, generated reference results, and a checker
wrapper. The promoted human-readable corpus is
[`../../../docs/reference/corpora/renju_corpus.md`](../../../docs/reference/corpora/renju_corpus.md).

## Files

| File | Role |
| --- | --- |
| [`extracted_boards.md`](extracted_boards.md) | Human-readable extracted boards from the RenjuNet advanced tutorial images. |
| [`extracted_boards.json`](extracted_boards.json) | Machine-readable extracted board material before fixture slicing. |
| [`extraction_notes.md`](extraction_notes.md) | Source-image ordering, split-case notes, manual correction caveats, and fixture-count rationale. |
| [`fixtures.json`](fixtures.json) | Machine-readable curated fixture cases used by reference checkers and the executable Renju rule corpus. |
| [`validation.md`](validation.md) | Current external reference-check summary against Piskvork. |
| [`piskvork_check.json`](piskvork_check.json) | Raw Piskvork checker results. |
| [`check_refs.py`](check_refs.py) | Reproducible dev-only checker driver. |

The original tutorial images are not vendored here. The extracted boards are
the durable source material we need for implementation and review; the source
URL stays attached for provenance.

## Coordinate Convention

Boards use Gomoku2D tactical-scenario coordinates:

- `A1` is the lower-left point.
- `O15` is the upper-right point.
- `B` is a black stone.
- `W` is a white stone.
- `?` is the probe point being checked as a Black Renju move.

The fixture boards are full 15x15 diagrams. `focus window` metadata records
which source-image slice the case came from; it is not part of the rule input.

## Extraction Notes

The extracted board material started from OCR/OpenCV-assisted image extraction,
then was manually corrected against the RenjuNet tutorial examples. Some source
images contain several independent situations, so those were split into
separate fixture candidates.

Fixture candidates intentionally clear source stones outside the focus window,
then add count-balancing filler stones outside the focus window. Fillers are
kept away from the probe row, column, and diagonals so they should not affect
the local forbidden proof. The goal is to keep each board usable as a legal
Black-to-move fixture while preserving the local shape under review.

Synthetic fixtures are called out in
[`../../../docs/reference/corpora/renju_corpus.md`](../../../docs/reference/corpora/renju_corpus.md). They
represent tutorial proof frames such as "after placing H11" or "after removing
G8" when the source image explains a continuation rather than a standalone
board.

## Validation Method

Run the tracked checker driver from the repo root:

```sh
python gomoku-bot-lab/external/renjunet-advanced-examples/check_refs.py
```

The script clones Piskvork into ignored lab output, pins it to the recorded
commit, compiles a tiny C++ wrapper, and runs the same `fixtures.json` through:

- Piskvork Renju `forbid(board, pos, size)`

Reference build products go under ignored lab output:

```text
gomoku-bot-lab/outputs/renjunet-advanced-examples-ref-check-build/
```

The validation result files in this directory are intentionally tracked so a
future fixture-label change can diff the external checker behavior. Piskvork is
enough for the default reference lane because SlowRenju uses the same visible
forbidden-check lineage and did not add independent evidence for this corpus.

Gomoku2D checks are intentionally not part of this wrapper. The extracted cases
are external golden input; Gomoku2D consumes them through `gomoku-eval
renju-rules`.

Important implementation detail: the Piskvork line checker encodes white stones
as `-1`, not `2`. Using `2` can produce false overline/double shape results
because the reference code uses line sums.

### Python Oracle Note

Do not transliterate Piskvork into a tracked Python oracle. It is GPL-licensed;
a line-by-line or close structural port would contaminate this MIT repo. It
would also turn external executable evidence into another implementation we must
maintain and validate.

A pure Python checker is still feasible later, but it should be one of:

- a clean-room implementation from RIF/RenjuNet semantics and the fixture pack;
- a validated port from a permissively licensed source such as `renju_forbid`,
  if we decide its behavior is suitable;
- an ignored local convenience script used only for investigation.

For now, the tracked source of external evidence remains the small wrapper
driver that compiles the external checker outside the repo.

## Current Result

Current checked set:

- 23 fixture cases.
- Piskvork matches expected labels: 23/23.
- External reference disagreements: 0/23.

Piskvork agreement is useful executable evidence, not a formal proof. If a
future case disagrees, prefer RIF/RenjuNet text and direct tutorial examples
over checker output.

## How To Use This Pack

Use this pack for:

- validating extracted RenjuNet fixture labels against external code;
- reviewing apparent-three/apparent-four edge cases;
- feeding the promoted golden corpus in `docs/renju_corpus.md`;
- comparing future extraction changes against known tutorial examples.

Do not put Gomoku2D implementation checks in this folder. They belong in the
eval/core test surfaces that consume the corpus.
