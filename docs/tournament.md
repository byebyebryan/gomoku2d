# Tournament Eval

`gomoku-eval tournament` is the bot-lab scheduling harness. It is for comparing
bot configs under repeatable conditions, not for modeling the full product game
flow or official human tournament opening rules.

## Schedule

The harness supports three pairing workflows:

| Schedule | Use For | Pairing Rule |
|----------|---------|--------------|
| `round-robin` | Curated/reference reports and final comparison sets | Every unordered bot pair from `--bots` |
| `head-to-head` | One focused question, such as line eval vs pattern eval | Exactly two bots from `--bots` |
| `gauntlet` | Testing one or more candidates without quadratic growth | `--candidate` or `--candidates` against each bot in `--anchors` |

For each scheduled pair it creates `games-per-pair` jobs. Even values are
important:

- game `0` uses bot A as black and bot B as white
- game `1` reuses the same opening with colors swapped
- game `2` advances to the next opening and repeats the color swap

Jobs run in parallel, then results are sorted back into deterministic match
order before sequential Elo, pairwise records, and reports are built.

Full round-robin scales quadratically:

```text
pairs = n * (n - 1) / 2
```

Use `head-to-head` or `gauntlet` while tuning. Promote only promising candidates
into the next full round-robin/reference report.

For ranking runs, prefer running the curated reference set from
`gomoku-bot-lab/`. The current set covers the depth ladder, tactical-cap
variants, and pattern-eval ablations without keeping no-safety bots in the
published baseline.

```sh
mkdir -p reports

cargo run --release -p gomoku-eval -- tournament \
  --bots search-d1,search-d3,search-d3+pattern-eval,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval,search-d3+pattern-eval+corridor-proof-c16-d8-w4,search-d5+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --seed 63 \
  --threads 22 \
  --report-json reports/latest.json
```

Long parallel tournaments print progress to stderr as games complete, including
elapsed time and ETA. When running in tmux, pipe both streams through `tee` if
you want a durable progress log:

```sh
cargo run --release -p gomoku-eval -- tournament ... \
  --report-json outputs/scratch.json 2>&1 | tee outputs/scratch.log
```

Focused head-to-head:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --schedule head-to-head \
  --bots search-d5+tactical-cap-8,search-d5+tactical-cap-8+pattern-eval \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --report-json outputs/head-to-head.json
```

Candidate gauntlet:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --schedule gauntlet \
  --candidate search-d5+tactical-cap-4+pattern-eval \
  --anchors search-d3,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --anchor-report reports/latest.json \
  --games-per-pair 64 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --report-json outputs/gauntlet.json
```

Batch gauntlet:

```sh
cargo run --release -p gomoku-eval -- tournament \
  --schedule gauntlet \
  --candidates search-d5+tactical-cap-4+pattern-eval,search-d5+tactical-cap-16+pattern-eval,search-d7+tactical-cap-4+pattern-eval,search-d7+tactical-cap-16+pattern-eval \
  --anchors search-d3,search-d5+tactical-cap-8+pattern-eval,search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4 \
  --anchor-report reports/latest.json \
  --games-per-pair 32 \
  --opening-policy centered-suite \
  --opening-plies 4 \
  --search-cpu-time-ms 1000 \
  --max-moves 120 \
  --report-json outputs/sweep-a-gauntlet.json
```

Gauntlet ratings should be treated as working calibration, not permanent truth:
anchor ratings come from the latest clean reference tournament, while the
gauntlet result is useful only under the same rule, opening policy, budget,
match caps, and code revision.

Batch gauntlets play candidates against anchors only. They intentionally do not
play candidate-vs-candidate games; promote surviving candidates to a focused
head-to-head or the next curated round-robin when that comparison matters.

`--anchor-report` points at a full round-robin report, normally
`gomoku-bot-lab/reports/latest.json`. The gauntlet report embeds the requested
anchor standings from that source, including source config and git provenance,
anchor-vs-anchor pair summaries, and aggregate anchor search-cost summaries, so
scratch gauntlets can be read without maintaining a separate rating cache. The
HTML report marks comparison rows by `current` versus `reference`: current rows
come from the gauntlet JSON itself; reference rows come from the embedded anchor
report and do not include per-match boards in the scratch report. The source
report must be `round-robin`; a gauntlet or head-to-head report is not accepted
as an anchor reference. The command also validates the source report against the
current rule config, opening policy/plies, search budgets, and max move/game
caps before running.

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

## Report Source Data

Tournament JSON records enough source data to explain the report without
re-running the tournament:

- each match stores compact `cell_index_v1` moves plus opening metadata
- centered-suite openings store opening index, suite index, base template, and
  transform index
- standings and per-side match stats split alpha-beta nodes from root
  safety-gate work
- candidate width is reported separately from child width after ordering and
  optional caps
- tactical annotation, legality, transposition-table, beta-cut, and reached
  depth counters are preserved for later report rendering

Generated candidate width can stay high for capped bots because the cap applies
after candidate generation, legality filtering, and ordering. Use `Child width`
in the report to see the actual searched non-root frontier.

## Report Process

Scratch tournament JSON and HTML belong in `gomoku-bot-lab/outputs/`, which is
ignored. Curated published reports belong in `gomoku-bot-lab/reports/`.

For published reports:

1. Commit code and docs first.
2. From `gomoku-bot-lab/`, run the full tournament from a clean working tree.
3. Confirm report provenance has `"git_dirty": false`.
4. Commit report JSON/HTML as a follow-up report commit.

If the tournament JSON is still the desired data source and only report
presentation changed, re-render the HTML from the existing clean
`reports/latest.json` instead of rerunning the tournament. The JSON remains the
source of truth for ranking and anchor ratings; the HTML is a derived view.

## Known Limitations

The centered suite is hand-curated, not solved. It is a better baseline than
whole-board random openings, and reports now track opening IDs directly. The
next eval-harness improvement is to use those IDs to identify and retire
templates that remain heavily color-dominated under stronger reference bots.
