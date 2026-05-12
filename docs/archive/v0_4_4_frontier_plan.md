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

The cost shape is improved but not solved. In that 24-game smoke, rolling query
time stayed tiny, but frontier update time was still roughly `290s` aggregated
for the rolling entrants because Black Renju annotations remain globally
refreshed. Budget exhaustion dropped versus the rebuild-backed checkpoint but
was still around `25-28%`, while scan stayed near `0%`. Treat this as a
correctness checkpoint, not a promotion candidate.

Next likely step: split Black Renju continuation effectiveness out of tactical
annotation refresh so most annotations can stay axis-local.
