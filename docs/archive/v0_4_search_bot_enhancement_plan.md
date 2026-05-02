# `v0.4.0` Search Bot Enhancement Plan

Status: ad-hoc implementation plan. This captures the current bot-lab work loop
so the commit boundaries and evaluation gates stay clear.

## Goal

Evolve the existing `SearchBot` into a measurable experimental bot without
forking a separate `AdvancedSearchBot` yet. The current baseline must remain
reproducible through config, while tactical features can be enabled one by one
for ablation testing.

## Design Direction

Keep one `SearchBot` implementation and extend `SearchBotConfig` with explicit
advanced toggles:

- `tactical_candidates`
- `tactical_move_ordering`
- `tactical_eval`

Baseline constructors and aliases keep those toggles off. Lab specs opt into
features with suffixes such as:

- `search-d3`
- `search-d3+candidates`
- `search-d3+ordering`
- `search-d3+eval`
- `search-d3+all`

This avoids duplicating the search loop while still allowing tournament reports
to compare each feature in isolation.

## Phases

### Phase 1: Freeze Baseline Behavior

Lock down current `SearchBot` behavior before tactical changes affect move
choice.

- Keep `SearchBot::new(depth)` and `SearchBotConfig::custom_depth(depth)` as
  frozen baseline config.
- Add tests that baseline tactical toggles default to off.
- Ensure trace output records all config fields so reports explain which knobs
  were active.
- Keep current web practice bot behavior unchanged.

### Phase 2: Add Experimental Config And Tactical Analyzer Skeleton

Add the scaffolding required for ablation tests without changing search results.

- Extend `SearchBotConfig` with the tactical toggles.
- Extend lab spec parsing so explicit depth specs and feature suffixes resolve
  into configs.
- Add an internal tactical analyzer skeleton.
- First analyzer fields:
  - legal move
  - immediate win
  - immediate block
- Do not wire analyzer output into candidate generation, ordering, or eval yet.

### Phase 3: Tactical Candidates

When `tactical_candidates` is enabled, keep radius-based candidates but
force-add tactically important moves that radius filtering might miss.

- Start with immediate wins and immediate blocks.
- Keep baseline candidate generation unchanged when the flag is off.
- Add curated sparse-position tests where the tactical move is outside the
  normal radius.
- Measure branching-factor impact through node counts.

### Phase 4: Tactical Move Ordering

When `tactical_move_ordering` is enabled, rank root and child candidates by
tactical urgency before alpha-beta search.

Suggested priority:

1. Immediate win.
2. Immediate block.
3. Creates major threat.
4. Blocks major threat.
5. Normal positional move.
6. Suspicious or low-value move.

Initial implementation can use only the analyzer fields that exist at the time.
Later analyzer work can refine the priority list.

### Phase 5: Tactical Eval

When `tactical_eval` is enabled, augment or replace the current contiguous-run
eval with feature-aware scoring.

Start conservative:

- Terminal win/loss scores stay unchanged.
- Own immediate/near-forcing threats score high.
- Opponent immediate/near-forcing threats score slightly higher defensively.
- Broken-three and double-threat features can be added after the first measured
  pass.

This phase is tuning-heavy and should not be merged just because tests pass.

## Intended Commit Boundaries

### Commit 1: Config Plumbing And Baseline Guardrails

Includes:

- New config toggle fields.
- Baseline constructors/presets with toggles off.
- Trace output including toggles.
- Lab spec parser support for feature suffixes.
- Tests for baseline defaults and parser behavior.

Expected behavior change: none.

### Commit 2: Tactical Analyzer Skeleton

Includes:

- Internal tactical analyzer type/helper.
- Tests for immediate win and immediate block detection.
- No integration with candidate generation, ordering, or eval.

Expected behavior change: none.

Current local slice combines Commit 1 and Commit 2 in code, but they can still
be split if we want separate history.

### Commit 3: Tactical Candidates

Includes:

- Candidate expansion when `tactical_candidates` is enabled.
- Focused tests for tactical move inclusion outside the normal radius.
- Small benchmark or tournament sanity check.

Expected behavior change: only for configs with `tactical_candidates = true`.

### Commit 4: Tactical Move Ordering

Includes:

- Candidate ordering when `tactical_move_ordering` is enabled.
- Tests that priority ordering is stable for immediate wins/blocks.
- Ablation tournament comparing baseline vs ordering-enabled configs.

Expected behavior change: only for configs with `tactical_move_ordering = true`.

### Commit 5: Tactical Eval

Includes:

- Eval changes behind `tactical_eval`.
- Tests for tactical score direction, not exact brittle values where possible.
- Ablation tournament comparing baseline, individual features, and full config.

Expected behavior change: only for configs with `tactical_eval = true`.

## Evaluation Gates

Before moving from one behavioral commit to the next:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler`
- `npm --prefix gomoku-web run build`

For commits 3-5, also run at least a small ablation tournament. After commit 5,
run a clean full tournament report and publish/update the report only from a
clean code commit.

## Risks

- Tactical candidates can increase branching factor enough to erase strength
  gains.
- Tactical ordering can improve pruning but also bias the bot into shallow
  tactical tunnel vision.
- Tactical eval is the highest-risk phase because tuning can pass unit tests
  while making play feel worse.
- If toggles make `search.rs` too hard to reason about, revisit splitting into
  a separate bot or extracting modules before adding more features.

