# Bot: SearchBot

- **File:** `gomoku-bot-lab/gomoku-bot/src/search.rs`
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
| `time_budget_ms` | Optional per-move wall-clock budget |
| `cpu_time_budget_ms` | Optional per-move Linux thread CPU-time budget |
| `candidate_radius` | Distance around existing stones used to generate candidate moves, or current-player stones for asymmetric candidate sources |
| `candidate_opponent_radius` | Optional opponent-stone radius for asymmetric candidate sources |
| `safety_gate` | Root safety gate: `current_obligation` or `none` |
| `move_ordering` | Alpha-beta move ordering: `tt_first_board_order`, lab-only `tactical_first`, `tactical_lite`, or `priority_first` |
| `child_limit` | Optional lab-only cap on the ordered non-root child frontier searched by alpha-beta |
| `static_eval` | Leaf board evaluator: default `line_shape_eval` or lab-only `pattern_eval` |
| `corridor_portals` | Optional corridor-extension controls, disabled by default |
| `threat_view_mode` | Threat-view backend: default `rolling`, validation-only `rolling_shadow`, or fallback `scan` |

Search traces expose explicit pipeline stages: `candidate_source`,
`legality_gate`, tactical annotation counters, `safety_gate`, and
`move_ordering`. Candidate sources currently cover symmetric near-all radii
(`near_all_rN`) and lab-only asymmetric current-player/opponent radii
(`near_self_rN_opponent_rM`). There is one legality gate (`exact_rules`), one
shared local-threat view, one root safety gate (`current_obligation`, or `none`
for ablations), four move-ordering modes (`tt_first_board_order` default,
`tactical_first`, `tactical_lite`, and `priority_first` lab-only), and an optional
`child_limit` lab cap. Static eval
defaults to the
global line-shape evaluator; `pattern_eval` is a lab-only alternative that
scores five-cell windows with Renju-aware completion/extension squares.
Local-threat annotation is policy-backed by `SearchThreatPolicy` in
`gomoku-bot::tactical`: the raw detector records shape facts, while the search
policy decides ordering score and must-keep status. Forbidden-only Renju Black
raw shapes do not receive tactical ordering or safety-gate credit, and mixed
legal/forbidden shapes keep only legal continuations for active threat strength.
Renju forbidden-move checks still use exact core rules, but core first applies a
cheap necessary-condition guard: a forbidden candidate must have at least two
black stones on one of the four local axes before the exact detector runs.

That Renju guard is deliberately not exposed as a bot component. It is a
correctness-preserving core legality optimization, not a playing-style knob:
the exact legality result is unchanged, while the measured candidate-legality
hot path is cheaper.

The lab tools primarily use explicit search specs over these fields:

| Spec | Max depth | Candidate source | Safety gate | Intent |
|---|---:|---|---|---|
| `search-d1` | 1 | `near_all_r2` | `current_obligation` | easy/beginner lane |
| `search-d3` | 3 | `near_all_r2` | `current_obligation` | current default baseline |
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
| Legality gate | `exact_rules` | no suffix | Exact core legality filtering, including Renju forbidden moves. |
| Root safety gate | `current_obligation` | `+no-safety` | Filters legal root candidates against immediate/imminent obligations already on the board. |
| Move ordering | `tt_first_board_order` | `+tactical-first`, `+priority-first`, `+tactical-lite` | How legal child moves are ordered before alpha-beta explores them. |
| Child width | uncapped | `+child-cap-N`, `+tactical-cap-N`, `+tactical-lite-cap-N`, `+priority-cap-N` | Caps non-root children after ordering; root still considers every legal/safe candidate. |
| Static eval | `line_shape_eval` | `+pattern-eval` | How leaf board positions are scored. |
| Threat view | `rolling` | `+rolling-frontier`, `+rolling-frontier-shadow`, `+scan-threat-view` | How tactical facts are answered for safety, ordering, win/block checks, and corridor queries. |
| Corridor proof | disabled | `+corridor-proof-cN-dM-wW`, legacy `+leaf-corridor-dM-wW`, `+leaf-proof-cN` | Optional after-search corridor proof over selected root candidates. |
| Corridor portal | disabled | `+corridor-own-dN-wM`, `+corridor-opponent-dN-wM` | Retired portal experiment; kept only for direct historical comparison. |

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

The retired opponent-reply safety probes are intentionally no longer accepted
as lab suffixes; the current safety gate only filters the legal root candidates
by obligations already present on the board. It preserves own immediate wins
first, then direct replies to opponent immediate wins, then
direct replies or counter-fours against opponent imminent threats. It does not
generate candidates, run opponent-reply search, reorder moves, or cap the root.
The retired `+corridor-q` leaf-quiescence experiment proved the shared corridor
module could be called from search, but it was too expensive to keep as a lab
axis. The later portal suffixes are also retired as candidate bot knobs. Only
the base disabled `+corridor-own-dN-wM` and `+corridor-opponent-dN-wM` suffixes
remain parseable for direct comparison against the failed portal shape; the
rank/top-N, proof-only, and static-exit suffixes are historical report evidence
only.

These specs are not durable product identity, and they are not character bots
yet. They exist so the lab can benchmark stable configs before deciding whether
UI presets like aggressive or defensive are real enough to expose. The older
`fast`/`balanced`/`deep` aliases still parse for compatibility, but they are not
the current anchor set.

## Corridor Integration

`gomoku-bot` now owns a replay-independent corridor module alongside
`SearchBot`. The earlier standalone `CorridorBot` bridge is retired. The live
search integrations are also retired as candidate presets:

- `+corridor-q` was leaf quiescence. It probed too many leaves that fell back to
  ordinary static evaluation.
- `+corridor-own-dN-wM` and `+corridor-opponent-dN-wM` tested selective
  child-move portal extension, including side-specific attacking and defensive
  controls.
- `+corridor-min-rank-N` and `+corridor-top-n-N` tried to restrict portal work
  to ordered high-value candidates.
- `+corridor-proof-only` avoided resume churn by using only terminal corridor
  proofs and falling back to the original child search on non-terminal exits.

These passes produced useful instrumentation, but not a useful bot knob. Resume
portals multiplied normal searches from corridor exits and distorted scores.
Proof-only portals were safer, but still spent hundreds of branch probes and
fallbacks per move for too few terminal proofs. In the latest `32` game
head-to-head checks, D3 proof-only lost `13-19` to base D3 while costing roughly
`176 ms/move` versus `60 ms/move`; D5+tactical-cap8 proof-only lost `15-17` to
base D5+tactical-cap8 while costing roughly `175 ms/move` versus `116 ms/move`.

Keep only the base `+corridor-own-dN-wM` and `+corridor-opponent-dN-wM`
suffixes as disabled lab evidence. Do not include portal variants in anchors,
sweeps, product difficulty ladders, or settings UI. The rank/top-N, proof-only,
and static-exit suffixes are historical controls from old reports, not current
parser surface.

The next corridor-in-search shape is candidate proof rather than leaf
extension. Normal iterative deepening runs first with corridor proof disabled
and keeps the safe fallback move. If that normal search completes max depth with
budget remaining, the lab suffix `+corridor-proof-cN-dM-wW` proves selected
normal-search root candidates: normal best first, then more candidates from the
normal-search ranking. `cN` is the root proof candidate cap, `dM` is max
corridor proof depth, and `wW` is max reply width. The suffix intentionally
checks the top `N` candidates regardless of normal-search score. Proof returns
only proven win, proven loss, or unknown. Unknown proof cannot outrank
normal-search score; only terminal proof can confirm, replace, or reject a
normal-search candidate. The current lab baseline is
`+corridor-proof-c16-d8-w4`: sixteen root candidates, proof depth eight, and
reply width four. The design is captured in
[`corridor_search.md`](corridor_search.md#candidate-proof-corridor-pass).
Reports render this suffix as `Corridor Proof`; the full CLI spelling remains
in raw JSON and docs commands for reproducibility.

The earlier broad leaf-extension experiments remain useful negative evidence.
Static exit could hurt play; terminal-only extension removed that obvious risk
but still burned most of a 1s/move budget without producing meaningful move
changes. Candidate proof is the first shape that produced stronger play:
`search-d3+corridor-proof-c16-d8-w3` beat base `search-d3` by `42-22` in a
64-game head-to-head while producing `31` proof-driven move changes. A
same-config H2H sweep showed the strongest signal when candidate proof is
paired with pattern eval: `D5 tactical-cap-8 + pattern-eval +
corridor-proof-c16-d8-w3` beat its base `38-26`, while plain capped D5
slightly regressed. Treat candidate proof as a promising lab branch, not a
product default. A later D5 width sweep over `w3/w4/w5/w8` locked `w4` as the
recommended shape: `w4` preserves the useful corridor-proof signal, keeps the
config power-of-two, and avoids `w8`'s extra cost without a clear strength
gain. Proof depth `16/32` was much more expensive.

`0.4.4` promotes the rolling-frontier implementation behind the `ThreatView`
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
    "corridor_portals": {
      "own": {
        "enabled": false,
        "max_depth": 0,
        "max_reply_width": 0
      },
      "opponent": {
        "enabled": false,
        "max_depth": 0,
        "max_reply_width": 0
      }
    },
    "search_algorithm": "alpha_beta_id",
    "static_eval": "line_shape_eval"
  },
  "depth": 3,
  "nominal_depth": 3,
  "effective_depth": 3,
  "corridor_extra_plies": 0,
  "nodes": 1234,
  "safety_nodes": 56,
  "corridor": {
    "search_nodes": 0,
    "branch_probes": 0,
    "max_depth_reached": 0,
    "extra_plies": 0,
    "resume_searches": 0,
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
    "corridor_entry_checks": 0,
    "corridor_entries_accepted": 0,
    "corridor_own_entries_accepted": 0,
    "corridor_opponent_entries_accepted": 0,
    "corridor_nodes": 0,
    "corridor_branch_probes": 0,
    "corridor_resume_searches": 0,
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
Corridor-portal metrics follow the same rule: split ordinary alpha-beta depth
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
[`tournament.md`](tournament.md) for the harness schedule and base templates.

## `v0.4.0` experiment takeaways

The detailed experiment log lives in
[`archive/v0_4_search_bot_enhancement_plan.md`](archive/v0_4_search_bot_enhancement_plan.md).
The canonical lessons are:

- Keep one `SearchBot` implementation for now. A separate `AdvancedSearchBot`
  is not justified until a behavior-changing strategy survives evaluation.
- Failed experimental knobs should be removed instead of kept as dormant config
  fields. Dead toggles make future reports harder to interpret.
- Depth remains the most reliable strength lever. A tactical feature must prove
  that it improves reached depth, runtime, or tournament strength under the same
  budget; fixing one depth-2 fixture is not enough.
- Tactical candidates, immediate-win/block ordering, broad threat extension, and
  broad shape eval all failed their promotion gates. The common failure mode was
  hidden extra work that reduced effective depth or match strength.
- Local-threat static eval has a sharper constraint than ordering/filtering:
  board-value scoring must stay globally consistent. A global tactical leaf eval
  is closer to semantically useful, but it is too expensive under fixed CPU
  budgets. A partial frontier leaf eval is cheaper, but it can overvalue recent
  local threats while ignoring older live threats elsewhere. Tactical facts
  should therefore feed ordering, must-keep child caps, safety gates, or narrow
  corridor probes before they feed static board value.
- `local_create_broken_three` is a diagnostic, not a target. If depth 3 already
  solves a position cleanly, making depth 2 imitate it is only useful when it is
  cheaper than reaching depth 3 normally.
- TSS vocabulary is useful for facts such as gain, cost/defense, and rest
  squares, but the practice bot should not become a full threat-space-search
  solver in this line. Solver-like work belongs in later analysis modules if
  replay review or puzzles need proof-oriented machinery.

The current direction is depth-oriented: improve the normal search cost first,
then use tactical facts only for cheap safety, move ordering, or narrow forced
branches that improve reached depth under the same budget.

On the easier side, depth can be lowered further than the current D3 baseline
without allowing obvious local-threat blunders. A focused tactical sweep showed
`search-d1`, `search-d2`, and `search-d3` all passing the `4/4` hard safety-gate
cases. D1 passed `13/16` total tactical cases, D2 passed `12/16`, and D3 passed
`11/16`; the non-hard cases are diagnostic shape/eval probes, so the key result
is that the safety gate covers immediate local win/block/open-three handling
even at depth 1. A 64-games-per-pair Renju tournament still produced a clear
strength ladder: D1 lost to D2 by `16-4-44`, D1 lost to D3 by `13-0-51`, and D2
lost to D3 by `11-2-51`. That makes D1 a plausible beginner/easy bot, D2 a
casual-but-less-soft bot, and D3 the stable baseline.

`child_limit` is currently a lab knob, not a default. Early tests show it is
most useful when paired with tactical ordering: pre-cleanup tests showed that a
cap without ordering dropped too much important coverage, while tactical-first
ordering with `child_limit` creates a real breadth-for-depth tradeoff. With root
uncapped, the D5 and D7 `tactical-first + child-cap-8` variants both beat
uncapped `search-d3` in a focused Renju tournament, and D7 beat D5. The clearest
same-depth signal so far is a 64-game Renju head-to-head where
`search-d5+tactical-cap-8` beat uncapped `search-d5` by `44-1-19`,
searched far fewer nodes, and reached more completed depth under the same
`1000 ms` CPU budget. A follow-up D9 `tactical-first + child-cap-4` variant
reached deeper on average than D7 cap8 but lost the head-to-head, suggesting
cap4 cuts too much breadth. Later D9 cap8 gauntlets were also negative: at
`1000 ms/move`, D9 cap8 variants still averaged only about depth `5.7`, and a
relaxed `3000 ms/move` budget reduced budget hits without producing meaningful
effective depth. That makes the cap a useful lab axis for harder/slower search
variants, but nominal depth should not be raised further without cheaper
ordering/eval or better forced-branch handling.

A wider 64-games-per-pair Renju tournament with the centered opening suite
across `search-d1`, `search-d3`, `search-d5+tactical-cap-8`, and
`search-d7+tactical-cap-8` confirmed the ladder shape: D1 was clearly
soft, D3 sat in the middle, and the two capped variants occupied the harder
side. Pairwise results were D1/D3 `3-0-61`, D1/D5-cap8 `3-0-61`, D1/D7-cap8
`3-0-61`, D3/D5-cap8 `23-0-41`, D3/D7-cap8 `15-0-49`, and D5-cap8/D7-cap8
`26-1-37`. D7 cap8 is the stronger hard-side bot in this suite, but it spent far
more budget than D5 cap8. Treat D5 cap8 as the efficient hard bot and D7 cap8 as
the slower hard-side variant.

A later 8-entrant reference report added uncapped D5 and the active pattern-eval
variants to that ladder. It proved the useful line-eval conclusions: uncapped D5
is not worth its budget cost, and pattern eval has a real match-strength signal
across D3, D5 cap8, and D7 cap8.

The current curated anchor report promotes the pattern and candidate-proof lanes
instead of keeping old line-eval middle anchors. The run used Renju, the
centered opening suite, `64` games per pair, `1000 ms` Linux CPU time per move,
and clean `0c7657e47994` report provenance. The latest refresh promotes D5
from tactical-cap-8 to tactical-cap-16 for both its pattern and
pattern+corridor-proof lanes. Its standings were:

| Rank | Bot | W-D-L | Read |
|---:|---|---:|---|
| 1 | `search-d5+tactical-cap-16+pattern-eval+corridor-proof-c16-d8-w4` | `274-2-172` | current strongest anchor; high-cost hard proof lane |
| 2 | `search-d3+pattern-eval+corridor-proof-c16-d8-w4` | `274-0-174` | unusually competitive lower-depth proof control |
| 3 | `search-d7+tactical-cap-8+pattern-eval+corridor-proof-c16-d8-w4` | `271-3-174` | close hard proof lane; slightly cheaper than D5 proof |
| 4 | `search-d7+tactical-cap-8+pattern-eval` | `269-3-176` | close non-proof hard control |
| 5 | `search-d5+tactical-cap-16+pattern-eval` | `261-2-185` | promoted D5 non-proof control |
| 6 | `search-d3+pattern-eval` | `248-1-199` | useful mid-strength pattern control |
| 7 | `search-d3` | `149-3-296` | stable default baseline |
| 8 | `search-d1` | `39-0-409` | easy/beginner lane |

The report makes candidate proof a serious lab branch but not a product default.
D5 cap16 proof tops the current report, D7 proof remains close, and D3 pattern
proof is strong enough to stay as a lower-depth strategic control. Keep corridor
proof in anchors so future sweeps measure against it directly; do not expose it
as a user-facing bot label yet. Reports render `corridor-proof-c16-d8-w4` as
`Corridor Proof`, while raw JSON and docs commands keep the full spelling for
reproducibility.

The key assumption is that depth remains the mechanism for seeing long play.
Non-tactical alpha-beta should find winning combinations if it can search deep
enough, but Gomoku's broad candidate set makes that unrealistic without better
breadth control. Local threat facts are therefore search-efficiency data, not a
replacement for search. They should let the bot keep tactically required moves,
order promising moves earlier, stage or cap quiet candidates more safely, and
extend only narrow forcing branches with concrete replies.

Static eval is intentionally still the global line-shape evaluator. The rejected
local-threat eval experiments showed the risk on both sides: broad local-threat
leaf scoring preserves global coverage but consumes too much compute, while a
recent-frontier-only leaf score is cheaper but can create tactical tunnel
vision. For now, tactical facts are consumers of the search pipeline, not a
replacement for globally consistent board evaluation. Partial/recent-frontier
leaf scoring is retired until a full-board rolling model can prove equivalent
coverage.

Tactical annotation is now routed through the same `ThreatView` seam as corridor
entry checks. In normal scan mode, `Board` remains the source of truth and
`ScanThreatView` computes the annotation directly. In rolling shadow mode, the
scan-backed annotation still drives behavior while the frontier-backed answer is
checked for parity. In rolling mode, tactical ordering can consume the
frontier-backed annotation as a lab-only validation path. Tactical-lite is the
middle tier: it asks the same threat-view seam for a compact candidate
tactical-lite rank, but does not run full tactical ordering summaries. Today
that rank only contains first-order corridor-entry strength; latent
second-order potential is intentionally deferred until the first-order rank path
is a clear cost win. Priority ordering is the cheaper sibling: it asks the
threat view only for current immediate-win/block sets, then orders quiet moves
by deterministic heuristics. All ordering modes can pair with `child_limit` to
test whether ordered coverage lets alpha-beta search fewer children without
changing candidate discovery. The frontier now updates through apply/undo. Root
tactical checks now use per-player threat-view annotations for immediate
wins/blocks, and corridor continuation/reply queries use the same threat-view
contract for immediate win checks and local corridor facts instead of a pre-move
attacker-rank scan surface. The next optimization boundary is validating
rolling-backed tactical-lite and full tactical queries against the anchor set
before doing more candidate-frontier maintenance.

For `v0.4.1`, the strategic target is a practice bot that climbs a tactical
ladder:

1. Local competence: never miss obvious immediate wins, single forced blocks,
   or clear four-shape reactions.
2. Casual combo play: recognize compound threats and priority races that casual
   human players often discover through probing.
3. Corridor steering: eventually spend bounded extra depth on narrow lines
   where local threat facts provide the gain move and concrete defender replies.

This keeps the bot aligned with the product. It should become more interesting
and configurable, not just more solver-like. Offensive and defensive styles
should eventually mean different budget allocation: own corridor search versus
opponent corridor prevention.

Positive search optimizations should land in place when they preserve exact
behavior and improve measured hot paths. They should become configurable only
when they represent a real tradeoff: strength versus speed, breadth versus
depth, style, safety, or explainability.

The `0.4.2` follow-up kept that restraint. It used the stronger harness for one
more lab pass before UI, then pivoted toward corridor search once the sweeps
showed that raw tuning knobs were not the most useful next product direction:

1. Tune existing axes first: depth, child cap, candidate source, and pattern
   eval.
2. Prototype bounded corridor search only where local facts provide concrete
   gain and defense replies.
3. Treat style/character as a later budget-allocation mechanism, not as an
   up-front label or eval-weight tweak.

Tactical scenarios remain diagnostics; a change should not be kept just because
it fixes a shallow fixture if it loses reached depth or tournament strength
against the current depth-3 and hard-side capped baselines.

The current `0.4.2` checkpoint therefore treats corridor search as the more
important foundation: inspect why bots win or lose, explain the final forced
sequence, and use that evidence before pushing more knobs into product settings.
The strategic model is documented in [`corridor_search.md`](corridor_search.md);
the replay analyzer contract is documented in
[`game_analysis.md`](game_analysis.md).

`0.4.3` keeps that restraint and stays in the lab before UI. The live-search
attempt showed that corridor portals are not useful as-is, but it left the bot
with shared tactical facts, honest portal metrics, and a threat-view contract.
Treat all corridor behavior as lab aliases or config flags until it survives
tournament, search-cost, and replay-analysis checks. The working plan lives in
[`archive/v0_4_3_corridor_bot_plan.md`](archive/v0_4_3_corridor_bot_plan.md).

The focused tactical scenario corpus is documented in
[`tactical_scenarios.md`](tactical_scenarios.md). It is layered into `local_*`,
`priority_*`, and `combo_*` cases, with explicit role, layer, intent, and shape
metadata in the JSON report. Use the hard safety-gate cases as regression
guards before tournament ablations; use diagnostic cases to understand behavior
and cost, not as standalone promotion gates.

The tactical shape vocabulary is documented in
[`tactical_shapes.md`](tactical_shapes.md). Shape facts are move-centric records
with a `kind`, `gain_square`, `defense_squares`, and `rest_squares`; this keeps
create, prevent, react, and future eval work tied to the same definitions.

### `0.4.2` sweep A read

The first `0.4.2` sweep stayed in the lab and used a batch gauntlet rather than
another full round robin: `8` child-cap / pattern-eval candidates against the
`8` clean `0.4.1` reference anchors, `32` games per candidate-anchor pair,
Renju, centered-suite openings, and `1000 ms` CPU time per move.

The important read is comparative, not absolute. A gauntlet does not play
candidate-vs-candidate or anchor-vs-anchor games, so it is a screening tool for
"worth testing again", not a final product-preset ranking.

Current takeaways:

- Pattern eval is still the main strength signal. The best candidate scores in
  this sweep all used `+pattern-eval`.
- Pattern eval is still not a default. The cost spread is wide: `D5 cap4
  pattern` was cheap enough to remain interesting, while `D7 cap16 pattern`
  spent too much budget.
- `tactical-cap-16` is not a general upgrade. Line-eval cap16 got slower and
  weaker; pattern cap16 had some score upside but did not look clean enough to
  justify the extra cost.
- `tactical-cap-4` is a real candidate, not just a toy narrowing. With tactical
  ordering and safety gates, cap4 often buys useful depth without obviously
  collapsing tactical coverage.
- The most useful next comparison is a smaller survivor run around `D5 cap4`,
  `D5 cap4 pattern`, `D5 cap8 pattern`, `D7 cap4`, `D7 cap4 pattern`, and the
  current `D7 cap8` anchors.

Detailed numbers live in [`performance_tuning.md`](performance_tuning.md).

### `0.4.2` sweep B/C read

The follow-up `0.4.2` sweeps tested candidate-source breadth. Symmetric `r3`
was too expensive to justify, and symmetric `r1` was too limiting as a general
source. The more useful question was asymmetric discovery: keep radius 2 around
the current player's stones while trimming opponent-stone discovery to radius 1
(`+near-self-r2-opponent-r1`).

A full gauntlet tested `D3`, `D5 tactical-cap-8`, and `D7 tactical-cap-8`, with
and without `+pattern-eval`, against the `8` clean reference anchors. The
strongest product conclusion is conservative: do not promote a new anchor from
this sweep. `self2/opponent1` is a useful lab axis, but its value depends on the
rest of the pipeline.

Takeaways:

- Plain `self2/opponent1` is not enough. It helps capped D5/D7 against their
  line-eval baselines, but it still loses badly to the pattern-eval anchors.
- `D3 + self2/opponent1 + pattern-eval` is the most interesting result. It
  tied `D3 + pattern-eval` head-to-head while reducing average move time from
  roughly `277 ms` to `176 ms` in the gauntlet schedule.
- The capped tactical variants are more questionable. Asymmetric candidates
  reduce the pre-ordering frontier, but tactical ordering and child caps still
  do the main pruning work.
- No anchor changes yet. The existing clean reference anchors remain the source
  of truth until a clean survivor run proves a replacement is both stronger and
  worth the extra config complexity.

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

---

## Known limitations / future work

- No dedicated threat-space search — misses forcing sequences that require looking ahead at threats specifically
- Eval doesn't detect double-threat patterns (double-three, four+three)
- Candidate radius 2 may miss some long-range setups
- No opening book — always searches from scratch on move 1
- Opening-suite balance is still hand-curated; tournament reports now track
  opening IDs so future eval can retire templates that remain color-dominated
  under stronger reference bots
- TT grows unbounded (no eviction); for longer matches this could be addressed with a fixed-size table and age-based replacement
