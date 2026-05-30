# Bot: SearchBot

- **File:** `gomoku-bot-lab/gomoku-bot/src/search/mod.rs`
- **Legacy name string:** `"baseline"`
- **Purpose:** Configurable alpha-beta search bot and the reference search
  implementation everything else gets compared against.

---

## Algorithm overview

Negamax with alpha-beta pruning and iterative deepening. The bot searches deeper
on each iteration, keeping the best result found so far, and cuts off when the
time budget is exhausted or a forced win/loss is detected. Time-budgeted search
checks the deadline inside the alpha-beta loop, so eval tournaments can compare
fixed-depth configs with a practical per-move cap.

The web game currently runs fixed-depth Wasm search without a product-side time
cap. Lab tournaments keep explicit timing policies: strict CPU-per-move for
focused continuity checks, and pooled CPU budgeting for curated published
reports that should better approximate a hard bot spending longer on difficult
positions while keeping average cost bounded.

```
for depth in 1..=max_depth:
    (score, move) = negamax(board, depth, -∞, +∞)
    if time_budget exceeded or abs(score) >= WIN:
        break
return best_move
```

Iterative deepening gives move ordering for free: the best move from depth N is tried first at depth N+1, which significantly improves alpha-beta cutoffs.

---

## Explicit config

`SearchBot` is built from `SearchBotConfig`. The compatibility constructors
still exist for legacy `baseline` specs and tests:

- `SearchBot::new(depth)` creates a custom fixed-depth search bot.
- `SearchBot::with_time(ms)` creates a custom time-budgeted search bot.

`gomoku-bot` intentionally exposes explicit engine knobs rather than owning
product presets:

| Config field | Meaning |
|---|---|
| `max_depth` | Fixed maximum iterative-deepening depth |
| `max_tt_entries` | Optional transposition-table entry cap; `None` means unbounded |
| `time_budget_ms` | Optional per-move wall-clock budget |
| `cpu_time_budget_ms` | Optional per-move Linux thread CPU-time budget |
| `candidate_radius` | Distance around existing stones used to generate candidate moves, or current-player stones for asymmetric candidate sources |
| `candidate_opponent_radius` | Optional opponent-stone radius for asymmetric candidate sources |
| `null_cell_culling` | Optional lab filter for generated candidates that cannot make five for either side in any direction |
| `safety_gate` | Root safety gate: `current_obligation` or `none` |
| `move_ordering` | Alpha-beta move ordering: `tt_first_board_order`, current `tactical`, or lab-only `tactical_full` |
| `child_limit` | Optional lab-only cap on the ordered non-root child frontier searched by alpha-beta |
| `static_eval` | Leaf board evaluator: default `line_shape_eval` or lab-only `pattern_eval` |
| `threat_view_mode` | Threat-view backend: default `rolling`, validation-only `rolling_shadow`, or fallback `scan` |

Search traces expose explicit pipeline stages: `candidate_source`,
`null_cell_culling`, `legality_gate`, tactical annotation counters,
`safety_gate`, and `move_ordering`. Candidate sources currently cover symmetric
near-all radii (`near_all_rN`) and lab-only asymmetric current-player/opponent
radii (`near_self_rN_opponent_rM`). Optional null-cell culling runs after
candidate generation and removes geometric dead cells that cannot participate
in any five-cell line for either color. There is one legality gate
(`exact_rules`), one shared local-threat view, one root safety gate
(`current_obligation`, or `none` for ablations), the default
`tt_first_board_order`, the current local-potential `tactical` ordering, the
older `tactical_full` comparison path, and an optional `child_limit` lab cap.
Static eval
defaults to the
global line-shape evaluator; `pattern_eval` is a lab-only alternative that
scores five-cell windows with Renju-aware completion/extension squares.
Local-threat annotation is policy-backed by `SearchThreatPolicy` in
`gomoku-bot::tactical`: the raw detector records shape facts, while the search
policy decides ordering score and must-keep status. Forbidden-only Renju Black
raw shapes do not receive tactical ordering or safety-gate credit, and mixed
legal/forbidden shapes keep only legal continuations for active threat strength.
Renju forbidden-move checks are owned by core rules; see
[`renju_rules.md`](renju_rules.md) and the
[`Renju Corpus`](../corpora/renju_corpus.md). Core first applies a cheap
necessary-condition guard: a forbidden candidate must have at least two black
stones on one of the four local axes before the detector runs.

That Renju guard is deliberately not exposed as a bot component. It is a
correctness-preserving core legality optimization, not a playing-style knob:
the exact legality result is unchanged, while the measured candidate-legality
hot path is cheaper.

The lab tools primarily use explicit search specs over these fields:

| Spec | Max depth | Candidate source | Safety gate | Intent |
|---|---:|---|---|---|
| `search-d1` | 1 | `near_all_r2` | `current_obligation` | easy/beginner lane |
| `search-d3` | 3 | `near_all_r2` | `current_obligation` | current default search spec |
| `search-d5` | 5 | `near_all_r2` | `current_obligation` | uncapped depth reference |
| `search-d5+tactical-cap-8` | 5 | `near_all_r2` | `current_obligation` | efficient hard-side candidate |
| `search-d7+tactical-cap-8` | 7 | `near_all_r2` | `current_obligation` | stronger but slower hard-side candidate |

### Config axes

Lab specs are additive: `search-d5+tactical-cap-16+pattern-eval` starts from
depth `5`, then changes move ordering/child cap, then changes static eval.
The important point is that similarly named suffixes can belong to different
pipeline axes.

| Axis | Default | Suffixes / controls | What it changes |
|---|---|---|---|
| Search budget | fixed depth from `search-dN` | CLI `--search-time-ms`, `--search-cpu-time-ms` | Iterative-deepening stopping condition. |
| Candidate source | `near_all_r2` | `+near-all-r1`, `+near-all-r2`, `+near-all-r3`, `+near-self-rN-opponent-rM` | Which empty cells enter the search before root safety or child ordering. |
| Null-cell cull | disabled | `+null-cull` | Optional post-generation filter for cells boxed out from every possible five for both colors. |
| Legality gate | `exact_rules` | no suffix | Exact core legality filtering, including Renju forbidden moves. |
| Root safety gate | `current_obligation` | `+no-safety` | Filters legal root candidates against immediate/imminent obligations already on the board. |
| Move ordering | `tt_first_board_order` | `+tactical-full` for old full annotation; `+tactical-cap-N` selects current tactical ordering with a cap | How legal child moves are ordered before alpha-beta explores them. |
| Child width | uncapped | `+child-cap-N`, `+tactical-cap-N`, `+tactical-full-cap-N` | Caps non-root children after ordering; root still considers every legal/safe candidate. |
| Static eval | `line_shape_eval` | `+pattern-eval` | How leaf board positions are scored. |
| Threat view | `rolling` | `+rolling-frontier`, `+rolling-frontier-shadow`, `+scan-threat-view` | How tactical facts are answered for safety, ordering, win/block checks, and corridor queries. |
| Corridor proof | disabled | `+corridor-proof-cN-dM-wW` | Optional after-search corridor proof over selected root candidates. |

The current suffix list does **not** include `+pattern-eval-scan`. Conceptually
that would be a static-eval implementation backend, separate from threat view.
Today, `+pattern-eval` uses the cached `PatternFrame` when the search is on the
rolling threat-view path; forcing `+scan-threat-view` also forces the older full
pattern scan as an implementation consequence. That coupling is why
`pattern-eval` versus `pattern-eval+scan-threat-view` can behave the same in
fixed-depth parity tests while reading like different features in report names.
If scan-vs-frame pattern comparisons remain common, the next cleanup should add
an explicit `+pattern-eval-scan` suffix rather than overloading
`+scan-threat-view`.

`+null-cull` is intentionally a lab axis, not a default. It is safe in the
geometric sense: a culled cell has no direction where either side could form a
five-cell line through that point. The rolling frontier caches the same fact for
search nodes; scan mode recomputes it from the board. The trace metrics split
root/search checks and culled counts so sweeps can tell whether the filter is
actually reducing useful breadth or just adding overhead.

The retired opponent-reply safety probes are intentionally no longer accepted
as lab suffixes; the current safety gate only filters the legal root candidates
by obligations already present on the board. It preserves own immediate wins
first, then direct replies to opponent immediate wins, then
direct replies or counter-fours against opponent imminent threats. It does not
generate candidates, run opponent-reply search, reorder moves, or cap the root.
The retired `+corridor-q` leaf-quiescence experiment proved the shared corridor
module could be called from search, but it was too expensive to keep as a lab
axis. The later portal suffixes are also retired as candidate bot knobs and are
no longer parser surface. Portal variants remain historical report evidence
only; use `+corridor-proof-cN-dM-wW` for the current after-search corridor
proof experiment.

The old split corridor suffixes `+leaf-corridor-dM-wW` and `+leaf-proof-cN`
are also retired and no longer accepted by the lab parser. Historical reports
may still contain those names, but new runs must use the single
`+corridor-proof-cN-dM-wW` suffix.

These specs are not durable product identity, and they are not character bots
yet. They exist so the lab can benchmark stable configs before deciding whether
UI presets like aggressive or defensive are real enough to expose. The older
`fast`/`balanced`/`deep` aliases still parse for compatibility, but they are not
the current anchor set.

### Product Control Surface

`0.4.5` exposed the bot lab through product controls, not raw parser surface.
The shipped surface has two layers:

- tested presets for normal play;
- advanced Bot Lab controls for players who want to inspect and shape the bot.

Presets resolve to explicit lab specs, but the UI label describes the player
experience:

| Product layer | Example UI label | Backing concept |
|---|---|---|
| Preset | `Easy` | shallow search with the current safety gate |
| Preset | `Normal` | everyday bot with stronger board-shape scoring |
| Preset | `Hard` | deeper/capped tactical search backed by the anchor report |
| Advanced | `Depth` | `search-dN` |
| Advanced | `Width` | child frontier cap such as `tactical-cap-N` |
| Advanced | `Pattern scoring` | `+pattern-eval` |
| Advanced | `Corridor proof` | fixed `+corridor-proof-c16-d8-w4` profile in v1 |
| Transparency | `Generated lab spec` | exact reproducible parser string |

Persist advanced config as product state, not as a raw lab spec string. The v1
custom shape is intentionally narrow: depth `1/3/5/7`, width `none/8/16`,
pattern scoring on/off, and corridor proof on/off. If future lab work changes
what these fields mean, bump the config version instead of silently changing old
settings.

Avoid exposing implementation and validation axes as normal user choices:

- `+rolling-frontier-shadow`
- `+scan-threat-view`
- `+no-safety`
- retired corridor portal / leaf-extension suffixes
- tournament-only CPU budget controls
- candidate radius/source
- raw thinking budget

The advanced layer can show the generated lab spec for reproducibility, but the
primary copy should be human-scale. For example, show `Hard + Pattern +
Corridor Proof` first, then show
`search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4` as the
secondary lab spec.

The current product presets remain intentionally simple:

- `Easy`: `search-d1`
- `Normal`: `search-d3+pattern-eval`
- `Hard`: `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4`

Keep `search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4` as an
advanced/reference lane rather than a fourth default preset for now. It is
strong and slightly cheaper than the top D7 corridor lane, but the D5/D7
corridor pair is close enough that exposing both as a clean difficulty ladder
would overstate the report evidence.

## Corridor Integration

`gomoku-bot` now owns a replay-independent corridor module alongside
`SearchBot`. The earlier standalone `CorridorBot`, leaf-quiescence suffixes,
and portal suffixes are retired. Do not include portal variants in anchors,
sweeps, product difficulty ladders, settings UI, or new docs commands.

The only current bot-facing corridor integration is candidate proof:

1. Run normal iterative deepening with corridor proof disabled.
2. If normal search completes max depth with budget remaining, prove selected
   root candidates from the normal-search ranking.
3. `+corridor-proof-cN-dM-wW` controls candidate count, proof depth, and reply
   width. The current baseline is `+corridor-proof-c16-d8-w4`.
4. Proof returns only proven win, proven loss, or unknown. Unknown cannot
   outrank the normal-search score; only terminal proof can confirm, reject, or
   replace a normal-search candidate.

Reports render the suffix as `Corridor Proof`; raw JSON and docs commands keep
the full spelling for reproducibility. Historical portal and leaf-extension
evidence lives in [`corridor_search.md`](corridor_search.md#bot-search-role)
and [`performance_tuning.md`](../../working/performance_tuning.md).

`0.4.4` promoted the rolling-frontier implementation behind the `ThreatView`
contract as the default threat-view backend after focused scan-vs-rolling
controls and shadow parity checks. Search keeps board, hash, and the optional
frontier synchronized through one recursive `SearchState`; scan remains
available through `+scan-threat-view` for fallback and comparisons:

- `+rolling-frontier-shadow` records scan-vs-frontier parity for tactical
  ordering annotations, current-obligation root safety, and any remaining
  corridor diagnostic queries while scan-backed answers still drive behavior. It
  also records scan time, frontier rebuild/update time, and frontier query time
  for those checks.
- `+rolling-frontier` explicitly selects the default frontier-backed answer for
  tactical ordering annotations, root win/block checks, and any remaining
  corridor diagnostic queries.
- `+scan-threat-view` forces the scan-backed threat view for fallback and
  comparison runs.

The shadow suffix is a validation and instrumentation mode, not a promoted bot
config. Incremental frontier deltas, lazy Renju filtering, per-state dirty
annotation memoization, indexed immediate-win lookup, and the simplified
`current_obligation` safety gate moved focused smoke results from "useful but
slower" to a net rolling speed win for normal tactical search. Scan-vs-rolling
controls should judge parity with relaxed or no per-move budget; normal
`1000 ms/move` runs are cost/strength samples and may shift because iterative
deepening completes different work under the clock. Scan remains the fallback
implementation behind the same `ThreatView` contract, so parity checks, safe
rollback, and targeted benchmarks stay cheap. Non-shadow rolling search is
expected to keep
`threat_view_scan_queries == 0`; scan queries in rolling work should be limited
to shadow comparison, tests, report/replay analysis, or explicit fallback paths.

Search traces include both the result and the config. Abridged example:

```json
{
  "config": {
    "max_depth": 3,
    "time_budget_ms": null,
    "cpu_time_budget_ms": null,
    "candidate_radius": 2,
    "candidate_opponent_radius": null,
    "candidate_source": "near_all_r2",
    "legality_gate": "exact_rules",
    "safety_gate": "current_obligation",
    "move_ordering": "tt_first_board_order",
    "child_limit": null,
    "search_algorithm": "alpha_beta_id",
    "static_eval": "line_shape_eval",
    "corridor_proof": {
      "enabled": false,
      "max_depth": 0,
      "max_reply_width": 0,
      "proof_candidate_limit": 0
    }
  },
  "depth": 3,
  "nominal_depth": 3,
  "effective_depth": 3,
  "nodes": 1234,
  "safety_nodes": 56,
  "corridor": {
    "search_nodes": 0,
    "branch_probes": 0,
    "max_depth_reached": 0,
    "width_exits": 0,
    "depth_exits": 0,
    "neutral_exits": 0,
    "terminal_exits": 0
  },
  "total_nodes": 1290,
  "metrics": {
    "root_candidate_generations": 1,
    "search_candidate_generations": 80,
    "root_legality_checks": 20,
    "search_legality_checks": 400,
    "root_tactical_annotations": 56,
    "search_tactical_annotations": 0,
    "child_limit_applications": 0,
    "root_child_limit_applications": 0,
    "search_child_limit_applications": 0,
    "child_cap_hits": 0,
    "root_child_cap_hits": 0,
    "search_child_cap_hits": 0,
    "root_child_moves_before_total": 0,
    "search_child_moves_before_total": 0,
    "root_child_moves_after_total": 0,
    "search_child_moves_after_total": 0,
    "corridor_nodes": 0,
    "corridor_branch_probes": 0,
    "corridor_width_exits": 0,
    "corridor_depth_exits": 0,
    "corridor_neutral_exits": 0,
    "corridor_terminal_exits": 0,
    "corridor_plies_followed": 0,
    "corridor_own_plies_followed": 0,
    "corridor_opponent_plies_followed": 0,
    "corridor_max_depth": 0
  },
  "score": 200,
  "budget_exhausted": false
}
```

`nodes` counts alpha-beta search nodes. `safety_nodes` counts root
current-obligation filtering work, not alpha-beta-equivalent nodes. The safety
gate is first-order: it inspects the already-generated legal root candidates
against current immediate and imminent obligations, optionally using the rolling
frontier in `rolling` or `rolling_shadow` threat-view mode. `total_nodes` is the
aggregate used by eval reporting. Root/search candidate and legality metrics
are split so pipeline-stage costs can be compared independently. Tactical
annotation metrics count reusable local-threat classification work separately
from candidate generation and alpha-beta nodes.
Child-cap metrics count ordered non-root frontier size before and after the
optional `child_limit`; root cap metrics stay zero because root is intentionally
uncapped, and all cap metrics are zero for default uncapped configs.
Tournament reports preserve this split as generated candidate width versus
post-ordering child width, so capped bots no longer look uncapped just because
their candidate generator still sees the same broad board frontier.
Corridor-proof metrics follow the same rule: split ordinary alpha-beta depth
from corridor extra plies, and report effective depth as a derived reach metric
rather than renaming the bot's nominal `max_depth`.
Node budgets are not enforced yet; this is currently a trace and tournament
metric.

## Tournament openings

`gomoku-eval tournament` defaults to `--opening-policy centered-suite` with
`--opening-plies 4`. The suite contains 32 deterministic, center-local Renju-safe
opening templates. In a 64-games-per-pair run, each bot pair sees each opening
once with both color assignments. This replaced the older `random-legal` mode,
which chose each opening move uniformly from the whole legal board and often
created scattered, color-dominated positions. Keep `--opening-policy
random-legal` only for noisy stress checks, not ranking. See
[`tournament.md`](../ops/tournament.md) for the harness schedule and base templates.

## Current Lab Read

The detailed experiment logs live in the archive and in
[`performance_tuning.md`](../../working/performance_tuning.md). The durable
lessons for current work are shorter:

- Keep one `SearchBot` implementation. Behavior-changing strategies should
  survive tournaments before they become separate bot identities.
- Remove failed knobs instead of keeping dormant parser surface. Dead toggles
  make reports harder to interpret.
- Depth is still the main way the bot sees long play, but Gomoku's breadth
  makes raw depth expensive without tactical ordering and caps.
- Tactical facts are search-efficiency data first. They should feed safety,
  move ordering, child caps, and narrow proof passes before they feed global
  board value.
- Static eval remains globally board-oriented. Pattern eval is the useful
  shipped lab axis; broad tactical leaf eval and recent-frontier-only leaf eval
  are retired paths.
- Corridor portal and leaf-extension search did not promote. Candidate proof is
  the only current corridor-in-search branch.
- The rolling `ThreatView` is the default hot-path backend; scan mode remains
  the fallback/comparison implementation.

The current curated anchor set promotes pattern eval, tactical caps, and
candidate-proof lanes instead of old line-eval middle anchors. The published
report is the source of truth for current standings and cost. Product presets
remain deliberately simple:

- `Easy`: `search-d1`
- `Normal`: `search-d3+pattern-eval`
- `Hard`: `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4`

Keep corridor proof in anchors and advanced controls, but continue treating the
exact proof suffix as lab detail. Reports render
`corridor-proof-c16-d8-w4` as `Corridor Proof`, while raw JSON and docs
commands keep the full spelling for reproducibility.

Tactical annotation is routed through the same `ThreatView` seam as corridor
entry checks. In scan mode, `ScanThreatView` computes the reference answer. In
rolling shadow mode, scan drives behavior while the frontier answer is checked
for parity. In rolling mode, tactical ordering consumes the frontier-backed
annotation. The current `tactical` path keeps immediate win/block checks global,
then fully annotates only local-potential moves before applying the child cap.
Without a child cap it falls back to `tactical_full`.

The focused tactical scenario corpus is documented in
[`tactical_scenarios.md`](../corpora/tactical_scenarios.md). It is layered into `local_*`,
`priority_*`, and `combo_*` cases, with explicit role, layer, intent, and shape
metadata in the JSON report. Use the hard safety-gate cases as regression
guards before tournament ablations; use diagnostic cases to understand behavior
and cost, not as standalone promotion gates.

The tactical shape vocabulary is documented in
[`tactical_shapes.md`](tactical_shapes.md). Shape facts are move-centric records
with a `kind`, `gain_square`, `defense_squares`, and `rest_squares`; this keeps
create, prevent, react, analysis, and eval work tied to the same definitions.

---

## Transposition table

Each position is keyed by a Zobrist hash (64-bit). The table stores:

| Field | Description |
|-------|-------------|
| `depth` | Depth at which this entry was searched |
| `score` | Score found |
| `flag` | `Exact`, `LowerBound`, or `UpperBound` |
| `best_move` | Best move found at this node (used for move ordering) |

On each node, if a TT entry exists at sufficient depth, we return early or tighten the alpha-beta window. The TT move is always tried first in the child loop.

### Zobrist hashing

Hash is computed incrementally — O(1) per node rather than O(board_size²). Each `(row, col, color)` triple has a precomputed random 64-bit value. The turn bit is XORed separately. When making a move, the child hash is:

```
child_hash = parent_hash ^ piece(row, col, color) ^ turn_bit
```

---

## Candidate move generation

Rather than searching all 225 cells, only empty cells within two rows/columns of
any existing stone are considered (`near_all_r2`). This is a square/Chebyshev
radius. The current tournament metrics show a typical generated candidate set
around 90 moves in developed Renju positions; earlier small-position estimates
are no longer a reliable planning number.

On an empty board, the first move is forced to the center.

**Known weakness:** radius 2 can miss long-range threats in sparse positions.
Radius 3 would catch more but grows the branching factor. Candidate radius is
now an explicit lab axis (`near_all_r1`, `near_all_r2`, `near_all_r3`) so future
experiments can trade breadth for reached depth deliberately. The lab also
supports asymmetric current-player/opponent radii (`near_self_rN_opponent_rM`),
currently exposed in specs as `+near-self-rN-opponent-rM`, to test whether
own-stone expansion can stay wider than opponent-stone expansion.

---

## Static evaluation

Called at leaf nodes (depth 0) or terminal positions.

Terminal positions return ±2,000,000 immediately.

For non-terminal positions, the default `line_shape_eval` scores runs of
consecutive same-color stones in all 4 directions (horizontal, vertical,
diagonal ↘, diagonal ↗) for both sides and returns
`my_score - opponent_score`.

### Run scoring

Each run is characterised by its **length** (2–4) and the number of **open ends** (0 = blocked both sides, 1 = half-open, 2 = fully open). Blocked runs (0 open ends) are ignored. The base values:

| Run length | Base score |
|------------|-----------|
| 4 | 10,000 |
| 3 | 1,000 |
| 2 | 100 |

Score per run = `base × open_ends_count`. A fully open four (score 20,000) is treated as near-forcing. An open three (2,000) is a serious threat.

**Known weakness:** the eval doesn't model threat interactions — two simultaneous open threes (a "double-three") aren't scored higher than their sum. A stronger eval would detect these compound threats explicitly.

### Pattern eval experiment

`pattern_eval` is a lab-only alternative selected with `+pattern-eval`. It scans
every five-cell window, scores windows with 2-4 stones and no opponent stones,
and counts only empty completion/extension squares that are legal for the
scored color. That means black Renju overline/double-three/double-four
completion squares are discounted through core legality, without changing the
board's current player during static eval.

Current evidence is mixed but still useful. Earlier 64-game Renju
head-to-heads at `1000 ms` CPU/move with the centered opening suite showed the
strength signal but also the scan cost:

| Pair | Pattern result | Avg move time tradeoff | Budget signal |
|---|---:|---|---|
| `search-d3` vs `search-d3+pattern-eval` | `49-0-15` | `326 ms` vs `39 ms` | pattern exhausted budget on `7.9%` of moves |
| `search-d5+tactical-cap-8` vs same `+pattern-eval` | `39-0-25` | `250 ms` vs `181 ms` | pattern exhausted budget on `1.2%` of moves |
| `search-d7+tactical-cap-8` vs same `+pattern-eval` | `35-3-26` | `581 ms` vs `429 ms` | both spent budget; pattern exhausted `40.9%` |

`0.4.4` keeps `+pattern-eval` as a lab axis, but promotes the implementation
path for rolling-backed pattern eval from full scan to a cached `PatternFrame`.
The frame stores the five-cell window scores and updates affected windows
alongside search apply/undo. Until an explicit `+pattern-eval-scan` suffix
exists, `+scan-threat-view` is also the practical way to force the legacy full
pattern scan for fallback and comparison.

Focused scan-vs-rolling controls over `64` games per pair show the cache is a
performance win without a clear strength regression:

| Pair | H2H score | Scan ms/move | Rolling ms/move | Budget signal |
|---|---:|---:|---:|---|
| `search-d3+pattern-eval` scan vs rolling | `33.5-30.5` | `106.0` | `70.3` | `0.6%` -> `0.2%` |
| `search-d5+tactical-cap-16+pattern-eval` scan vs rolling | `33.5-30.5` | `285.5` | `203.9` | `5.0%` -> `1.1%` |
| `search-d7+tactical-cap-8+pattern-eval` scan vs rolling | `32.0-32.0` | `381.8` | `267.2` | `18.7%` -> `7.5%` |

The small score edges are treated as noise. Fixed-depth parity tests keep scan
and rolling cache choices identical on benchmark scenarios, and a debug
head-to-head smoke recorded `156,214` pattern-frame shadow checks with `0`
mismatches. This is enough to use the cached frame as the normal rolling
implementation. It is not, by itself, a reason to promote `+pattern-eval` as the
product default.

## Tactical Ordering Cost Gate

The latest tactical-ordering profile showed that full tactical ordering spends
too much time annotating candidate-created threats for children that are then
dropped by `child_limit`. In the current pooled anchor report, D5/D7 pattern
lanes still generate roughly `87` candidate moves per search node before
keeping `8` or `16`, while threat-stage time sits around `43-47%` of the
captured stage total. That is lower than the older strict-budget reports, but it
is still the largest named stage.

`+tactical-cap-N` is now the targeted cost-reduction path. It keeps immediate
win/block checks global, then runs full tactical annotation only for moves that
pass a cheap local tactical-potential gate: a candidate-inclusive five-cell line
must contain the candidate plus at least two existing friendly stones and no
opponent stones. Root ordering and uncapped tactical ordering still fall back to
full tactical so root tactics are not hidden. The older full-annotation path is
available as `+tactical-full` / `+tactical-full-cap-N` for direct comparisons.

Focused same-config H2H at `1000 ms` CPU/move, centered-suite openings, `64`
games per pair, seed `79`:

| Pair | Result for tactical | Avg move time | Annotation query change |
|---|---:|---:|---:|
| D5 `tactical-cap-16+pattern` vs D5 `tactical-full-cap-16+pattern` | `33-1-30` | `194 ms` vs `215 ms` | `41M` vs `171M` |
| D5 `tactical-cap-16+pattern+corridor-proof` vs D5 `tactical-full-cap-16+pattern+corridor-proof` | `32-0-32` | `236 ms` vs `263 ms` | `27M` vs `129M` |
| D7 `tactical-cap-8+pattern` vs D7 `tactical-full-cap-8+pattern` | `34-1-29` | `275 ms` vs `320 ms` | `72M` vs `332M` |
| D7 `tactical-cap-8+pattern+corridor-proof` vs D7 `tactical-full-cap-8+pattern+corridor-proof` | `35-0-29` | `308 ms` vs `348 ms` | `61M` vs `284M` |

The tactical cost gate hits the intended mechanism: fewer full tactical annotations and
lower move time. The current local-potential gate is also behaviorally credible
in the focused H2H set: the gated tactical path won two same-config pairs, tied
one, and only traded evenly in D5 proof while still being faster. It is still a
lab candidate until it survives a full anchor tournament.

Rejected intermediate: a broad tactical-potential gate with priority quiet
scoring was tested and reverted. It also reduced annotation count, but it was
too broad and changed quiet ordering enough to search more nodes, so it lost all
four same-config H2Hs against full tactical. The current tactical version avoids
that by keeping full tactical tie semantics for unannotated quiet moves and by
guarding benchmark candidates against false negatives.

---

## Known limitations / future work

- No dedicated threat-space search. The bot can still miss forcing sequences
  that require proof-oriented threat chaining beyond normal alpha-beta depth.
- Lethal/compound threat facts exist for analysis and tactical obligations, but
  they are not a full replacement for global static evaluation.
- Candidate radius 2 may miss some long-range sparse setups.
- No opening book — always searches from scratch on move 1.
- Opening-suite balance is still hand-curated; tournament reports now track
  opening IDs so future eval can retire templates that remain color-dominated
  under stronger reference bots.
- The transposition table supports an optional entry cap, but default lab specs
  currently leave it unbounded unless a product or benchmark run sets a cap.
