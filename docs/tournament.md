# Tournament Eval

`gomoku-eval tournament` is the bot-lab round-robin harness. It is for comparing
bot configs under repeatable conditions, not for modeling the full product game
flow or official human tournament opening rules.

## Schedule

The harness receives a comma-separated bot list and runs every unordered pair.
For each pair it creates `games-per-pair` jobs. Even values are important:

- game `0` uses bot A as black and bot B as white
- game `1` reuses the same opening with colors swapped
- game `2` advances to the next opening and repeats the color swap

Jobs run in parallel, then results are sorted back into deterministic match
order before sequential Elo, pairwise records, and reports are built.

For ranking runs, prefer running from `gomoku-bot-lab/`:

```sh
mkdir -p reports

cargo run --release -p gomoku-eval -- tournament \
  --bots search-d2,search-d3,search-d5 \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 48 \
  --threads 22 \
  --report-json reports/latest.json
```

## Opening Policies

`centered-suite` is the default. It exists because the previous `random-legal`
opening mode picked uniformly from every legal board point, which often produced
scattered, low-value moves and color-dominated games. `random-legal` remains
available as a noisy stress mode, but should not be used for ranking.

The centered suite is intentionally modest:

- fixed first move at board center
- 4-ply templates
- all moves stay within the central `5x5`
- legal under Renju
- 4 base templates expanded by the 8 square symmetries
- 32 total openings

With `--games-per-pair 64`, every bot pair sees all 32 openings once with both
color assignments.

`--seed` rotates the suite start. It does not invent new random shapes. This
keeps ranking runs varied but still comparable.

## Base Templates

Templates are listed as offsets from center `(0, 0)`. On the default 15x15 board,
center is `H8`.

| Template | Relative offsets | Default orientation |
|----------|------------------|---------------------|
| `base-0` | `(0,0) (0,1) (1,0) (1,1)` | `H8 I8 H9 I9` |
| `base-1` | `(0,0) (1,0) (0,2) (-1,1)` | `H8 H9 J8 I7` |
| `base-2` | `(0,0) (1,1) (-1,1) (2,0)` | `H8 I9 I7 H10` |
| `base-3` | `(0,0) (-1,1) (1,0) (0,-2)` | `H8 I7 H9 F8` |

Each template is transformed through the 8 square symmetries: identity,
rotations, horizontal/vertical reflections, and diagonal mirrors. The generated
suite is code-defined in `gomoku-bot-lab/gomoku-eval/src/opening.rs`.

## Reading Results

Pairwise results are reported as `bot A wins - draws - bot B wins`. This is the
primary view for direct comparisons.

Color splits are still important. If both games for many opening pairs are won by
the same color, the opening suite is still carrying part of the result. Treat
that as an eval-harness signal, not as a bot-strength conclusion.

Shuffled Elo is report-local. It helps reduce sequential update noise, but it is
not a persistent rating system.

## Report Process

Scratch tournament JSON and HTML belong in `gomoku-bot-lab/outputs/`, which is
ignored. Curated published reports belong in `gomoku-bot-lab/reports/`.

For published reports:

1. Commit code and docs first.
2. From `gomoku-bot-lab/`, run the full tournament from a clean working tree.
3. Confirm report provenance has `"git_dirty": false`.
4. Commit report JSON/HTML as a follow-up report commit.

## Known Limitations

The centered suite is hand-curated, not solved. It is a better baseline than
whole-board random openings, but future eval should track opening IDs directly
and retire templates that remain heavily color-dominated under stronger
reference bots.
