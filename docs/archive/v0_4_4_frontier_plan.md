# v0.4.4 Rolling Frontier Plan

Purpose: make local threat facts cheap enough for corridor-aware bot work by
adding a correctness-first rolling frontier behind the existing `ThreatView`
seam.

## Why This Exists

`0.4.3` proved that corridor search can be shared by replay analysis and bot
experiments, but the first live portal variants remained too expensive. The
problem is now structural: the bot asks more and more local-threat questions,
while the implementation still rediscovers those facts through scans.

`0.4.4` should not start by tuning another portal suffix. It should build the
derived threat index that makes those suffixes plausible later.

## Goal

Add a rolling threat frontier that can answer the current search-facing
`ThreatView` queries with the same semantics as `ScanThreatView`, then validate
it in shadow mode before allowing any hot search path to use it for behavior.

Success means:

- scan-backed and rolling-backed threat answers match on tactical fixtures,
  random apply/undo sequences, and Renju forbidden cases;
- the rolling implementation survives exact undo back to previous states;
- shadow mode can compare both views without changing bot choices;
- metrics make query count, update count, and mismatch failures visible;
- corridor/search behavior remains unchanged unless an explicit lab flag opts
  into the rolling view.

## Non-Goals

- Do not replace `Board`. `Board` remains the authority for stones, turn,
  result, rule config, legality, apply, and undo.
- Do not expose player-facing bot settings in this slice.
- Do not promote corridor portal variants as product bots.
- Do not optimize storage layout before semantic parity is proven.
- Do not design the frontier around HTML report needs. Reports can keep using
  scan-backed or batch helpers until the hot search contract is stable.

## Architecture

The split should stay explicit:

- `Board`: authoritative game state and exact legality.
- `ThreatView`: read-only query contract used by search and corridor logic.
- `ScanThreatView`: reference implementation backed by current scanners.
- `RollingThreatFrontier`: derived cache synchronized with board apply/undo.

Design decision: rolling stays alongside scan instead of replacing it. Scan is
the semantic reference, the safe fallback, and the simplest way to diagnose a
rolling mismatch. Rolling is an alternate implementation behind the same
`ThreatView` contract, enabled only by explicit lab/config flags until it proves
both parity and useful cost shape. This keeps a little extra plumbing around,
but it pays for itself by making future frontier work reversible, shadowable,
and benchmarkable.

The frontier stores normalized tactical facts, not gameplay state. A useful
fact should carry enough information to compare, filter, and update without
re-reading report-specific structures:

```text
ThreatFact {
  player,
  kind,
  origin,
  defense_squares,
  rest_squares,
  legal_forcing_continuations,
  forbidden_black_squares
}
```

The first implementation can keep the existing `LocalThreatFact` shape and add
normalization helpers around it. If later profiling shows memory layout matters,
compact keys or bitsets can come after parity.

## Update Model

The safe path is incremental:

1. Add stable sorting/dedup helpers for threat facts and view query outputs.
2. Add a full-rebuild cached frontier that implements `ThreatView` from a
   snapshot board. This is a reference cache, not the final performance win.
3. Add rolling apply/undo with conservative invalidation around the last move.
4. Add differential tests that compare `ScanThreatView` and the frontier after
   every apply and after every undo.
5. Add search shadow mode that computes both answers and fails fast on mismatch
   while still using scan answers for behavior.
6. Only after shadow mode is clean, add explicit lab flags that let normal
   search tactical ordering and then corridor queries use the rolling view.

Conservative invalidation is acceptable. Missing or stale facts are not. Most
local shape facts are axis-local: a move can only affect shape facts and simple
annotations on the four Gomoku axes crossing that move. Renju Black tactical
annotations are broader because continuation filtering asks whether a
hypothetical forcing continuation leads to an immediate Black win elsewhere, so
the first safe rolling pass keeps Black Renju annotations globally refreshed
until that continuation cache is split out.

## Renju Rules

Renju must stay explicit because legality changes tactical meaning:

- raw shape facts and legal forcing continuations are separate concepts;
- Black forbidden continuations are not active threats;
- White threats whose natural Black replies are forbidden remain meaningful;
- Black-only forbidden squares must survive parity tests and diagnostics.

The frontier may ask `Board` for exact legality. The optimization target is
avoiding repeated threat scans, not replacing core rules.

## Shadow Mode

Shadow mode is the main safety guard:

- scan answers drive behavior;
- rolling answers are computed for the same query;
- mismatches should include board FEN, query name, player/attacker, scan facts,
  rolling facts, and last move when available;
- tournament/eval smoke runs should be able to enable shadow mode without
  changing standings.

The first shadow consumers should be the current `ThreatView` queries:

- `search_annotation_for_move(mv)`;
- `active_corridor_threats(attacker)`;
- `has_move_local_corridor_entry(attacker, mv)`;
- `defender_reply_moves(attacker, actual_reply)`;
- `attacker_move_rank(attacker, mv)`.

## Checkpoints

1. Docs and roadmap alignment.
2. Threat fact normalization plus scan parity tests.
3. Full-rebuild frontier plus fixture parity tests.
4. Rolling apply/undo frontier plus random sequence parity tests.
5. Search shadow mode plus metrics.
6. Optional opt-in rolling consumer for normal tactical ordering.
7. Optional opt-in rolling consumer for corridor portal entry/reply paths.

Stop after checkpoint 4 if parity is not exact. A wrong frontier is worse than
a slow scan.

## First Implementation Checkpoint

The first code checkpoint intentionally favors contracts over performance:

- `RebuildThreatFrontier` implements `ThreatView` from a snapshot board.
- `RollingThreatFrontier` owns apply/undo discipline but currently rebuilds its
  cached view after each move.
- `ThreatViewMode::RollingShadow` compares rolling-backed portal entry and
  tactical ordering answers against scan-backed answers while scan still drives
  behavior.
- `ThreatViewMode::Rolling` can drive portal entry checks and tactical ordering
  as a lab-only opt-in.
- Lab specs parse `+rolling-frontier-shadow` and `+rolling-frontier`.
- Search traces record scan query time, frontier rebuild/update time, and
  frontier query time for the threat-view checks that are active in the selected
  mode.
- `SearchState` keeps the recursive search board, Zobrist hash, and optional
  frontier synchronized through apply/undo. Plain scan mode does not maintain a
  frontier; rolling shadow/rolling modes do.

This was not the final rolling invalidation model yet. It was a safe seam for
measuring correctness and wiring cost before replacing rebuilds with localized
fact updates. The frontier timing at this checkpoint was rebuild-backed by
design; it was a baseline for the seam, not an estimate of the eventual
localized update model.
The next frontier checkpoint is full-parity localized fact invalidation inside
`RollingThreatFrontier`; the search-context stack is now the bridge that makes
that work measurable in real recursion. Partial/recent-frontier shortcuts stay
out of scope until the full rolling model matches scan behavior.

## Incremental Checkpoint

The second code checkpoint replaces the full board snapshot undo stack with
frontier deltas:

- `RollingThreatFrontier` now applies and undoes the same move as `Board`
  instead of cloning a previous board snapshot.
- Per-origin local threat facts are refreshed along the four full Gomoku axes
  crossing the changed move.
- Tactical annotations are cached per side so turn changes do not invalidate the
  whole cache by themselves.
- White annotations and non-Renju/simple annotations are refreshed on affected
  axes; Black Renju annotations are still globally refreshed for correctness.
- Global active-threat lists remain scan-backed because the per-origin cache and
  canonical global threat list intentionally deduplicate at different
  granularities.

Focused parity tests cover apply/undo, deterministic Renju sequences, full
annotation comparisons, per-origin entry checks, and same-axis invalidation.
Shadow tournament smoke with `search-d3+tactical-cap-8`,
`+rolling-frontier-shadow`, and `+rolling-frontier` over `3 pairs x 8 games`
with `1000 ms` CPU budget reached zero shadow mismatches.

The cost shape improved but was not solved. In that 24-game smoke, rolling query
time stayed tiny, but frontier update time was still roughly `290s` aggregated
for the rolling entrants because Black Renju annotations remained globally
refreshed. Budget exhaustion dropped versus the rebuild-backed checkpoint but
was still around `25-28%`, while scan stayed near `0%`. Treat this as a
correctness checkpoint, not a promotion candidate.

## Lazy Black Renju Checkpoint

The next checkpoint split Black Renju continuation effectiveness out of eager
annotation refresh:

- search annotations are cached as raw local shape facts per side;
- both Black and White raw annotations now refresh only on affected Gomoku axes;
- public rolling-frontier queries lazily apply the exact Black Renju
  continuation-effectiveness filter before returning an annotation;
- `SearchThreatPolicy` now has player-explicit raw/effective annotation helpers,
  which removes the old clone-and-mutate-current-player path for normal
  annotation lookup.

A focused 24-game smoke with `search-d3+tactical-cap-8`,
`+rolling-frontier-shadow`, and `+rolling-frontier` reached zero shadow
mismatches and zero budget exhaustion. Frontier update cost dropped from roughly
`250-280 us/update` to roughly `56-57 us/update`; query cost rose from tens of
nanoseconds to about `1 us/query`, which is the expected tradeoff after moving
Black Renju filtering to lookup time.

Rolling is still a lab-only mode, but this checkpoint changes the bottleneck:
the remaining gap is no longer global Black Renju annotation refresh. The next
frontier work should profile whether lazy Black Renju query filtering,
scan-backed active corridor threat lists, or apply/undo bookkeeping is the
dominant cost.

## Dirty Annotation Checkpoint

The next optimization keeps raw tactical annotations cached, but stops eagerly
refreshing them during frontier apply/undo:

- apply/undo still refreshes per-origin local move facts eagerly because those
  facts back `has_move_local_corridor_entry`;
- search annotations now carry per-side dirty flags instead of cloned previous
  annotation values in the undo delta;
- apply marks affected-axis annotations dirty instead of recomputing raw
  annotations immediately;
- undo restores the previous dirty flags;
- query-time lookup uses the cached raw annotation when clean, and recomputes a
  raw annotation from the current board when dirty before applying the Black
  Renju effectiveness filter;
- terminal boards fall back to scan-equivalent empty annotations so a clean,
  off-axis cached pre-terminal annotation cannot leak after a winning move.

This keeps the scan/shadow safety net intact while testing whether annotation
refresh, rather than move-fact maintenance or remaining root scans, is the real
rolling-frontier bottleneck. The 24-game smoke result confirmed that direction:
rolling update cost dropped from roughly `57 us/update` to roughly
`7.3 us/update`; rolling query cost rose from roughly `1.2 us/query` to roughly
`1.6 us/query`; and the rolling entrant moved from roughly `160 ms/move` to
roughly `72 ms/move`, effectively matching the scan baseline for the smoke.
Shadow mismatches and budget exhaustion stayed at zero.

## Tactical-Only Frontier Checkpoint

The next checkpoint split the rolling frontier into feature modes:

- `Full` keeps both tactical annotations and per-origin move facts, which is
  still required for corridor portal entry checks.
- `TacticalOnly` keeps tactical annotations and dirty-axis tracking, but skips
  per-origin move-fact maintenance. Normal search uses this mode when corridor
  portals are disabled.
- Corridor move-fact queries in `TacticalOnly` fall back to the scan reference
  path, so the mode is safe even if a caller accidentally asks a corridor
  question.
- Search metrics now split frontier update/query cost into delta capture,
  move-fact maintenance, dirty marking, clean annotation lookup, dirty
  annotation recompute, and fallback lookup. The report aggregator carries
  those fields through standings and side stats.

The 24-game Renju smoke with `search-d3+tactical-cap-8`,
`+rolling-frontier-shadow`, and `+rolling-frontier` at `1000 ms/move` reached
zero shadow mismatches and zero budget exhaustion. With the safety gate enabled,
scan averaged `71.9 ms/move`; tactical-only rolling averaged `59.1 ms/move`.
The rolling entrant still paid `13.0s` of frontier annotation lookup time and
`0.47s` of update time across the run, but avoided `19.8s` of repeated scan
query time.

A second 24-game smoke with the same specs plus `+no-safety` isolated the pure
search-annotation path. Scan averaged `29.7 ms/move`; tactical-only rolling
averaged `21.9 ms/move`. Move-fact update counts stayed at `0` in both rolling
smokes, confirming that normal search no longer pays for corridor-specific
facts while portals are off.

This is the first rolling-frontier checkpoint that is a net speed win for
normal tactical search. It should still remain lab-only until a larger anchor
tournament confirms behavior parity under the full report workload. The next
optimization question is no longer "can rolling beat scan at all?" but "can
dirty annotation recompute and remaining root scan queries be reduced without
reintroducing eager move-fact cost?"

## Search-State Memo Checkpoint

Dirty annotation write-back is not safe as a direct frontier-cache mutation
unless undo also restores the previous raw annotation value. A child search can
dirty, recompute, and clean an annotation, then undo back to a parent state that
would incorrectly see the child annotation as clean. To avoid reintroducing that
apply/undo burden, the next checkpoint caches dirty recompute results in
`SearchState` instead:

- memo key: current incremental board hash, current player, and candidate move;
- memo value: effective tactical annotation for that exact state;
- only dirty recomputes are memoized, because clean frontier lookups are already
  cheap;
- apply/undo does not need to restore memo entries because the board hash is
  part of the key;
- report metrics now include memo annotation query count/time so repeated dirty
  recomputes are visible.

Focused smoke results stayed behavior-neutral and showed a real cost drop:

- D3 safety-enabled paired smoke: rolling averaged `39.9 ms/move` versus scan at
  `49.3 ms/move`. Rolling still paid `2.49s` of scan-backed root safety checks,
  but dirty recompute dropped to `1.89s` plus `0.018s` of memo-hit time.
- D3 no-safety paired smoke: rolling averaged `18.8 ms/move` versus scan at
  `26.7 ms/move`.
- D5 cap8 paired smoke: rolling averaged `159.5 ms/move`, essentially tied with
  scan at `159.7 ms/move`, while searching more nodes and avoiding budget
  exhaustion in the small sample.

The root safety target is now the `current_obligation` gate. Turning safety off
stays a diagnostic; the endpoint is a cheap first-order filter that can use
scan, rolling, or rolling-shadow threat views. Because the safety pass runs
before the root `SearchState` exists and needs active existing-threat facts,
rolling safety builds a root-only full frontier instead of reusing the
tactical-only search frontier. The retired opponent-reply probes should stay out
of the parser unless a future experiment reintroduces them under a new name.
