# Roadmap

This roadmap tracks current sequencing. Completed phase detail lives in
[`release_history.md`](../../archive/release_history.md); this file should stay
short enough to answer "what is next and why?"

## Current Line: `v0.5.4` Reconciliation Closeout

`v0.5.3` established a stable public-alpha checkpoint, but the resumed
project-wide review found that declaring the `0.5` line complete was premature.
The release remains a valid shipped snapshot; `v0.5.4` finishes the original
reconciliation goal without rewriting that history.

Goal: leave the product and repository in a deliberately reviewed state before
starting another feature line.

Current work:

- audit large Rust, report, and frontend ownership centers before refactoring;
- remove or explicitly retain stale compatibility and experimental paths;
- reconcile tests, suite runtime, dependencies, CI, generated artifacts, and
  active versus parked documentation;
- run a fresh desktop/mobile product walkthrough and fix concrete usability,
  presentation, accessibility, loading, and error-state issues;
- reconcile the README, metadata, and release-facing artifacts around a
  product-first story without making a devlog or process essay a release gate;
- preserve behavior unless a review finding identifies a real defect.

Release bar:

- every review finding is fixed, deliberately retained, or explicitly deferred
  with an owner and reason;
- active docs describe current behavior and `docs/working/` contains only
  genuinely active material;
- full Rust, wasm, web, rules, report, and browser validation passes;
- `v0.5.4` is a clean closeout, not a hidden online, bot-research, or redesign
  release.

Detailed execution lives in
[`v0.5.4 Reconciliation Closeout Plan`](../../working/v0_5_4_reconciliation_plan.md).

## Next Line: `v0.6` Online Product Expansion Planning

After `v0.5.4`, the next step is a fresh design checkpoint, not immediate
implementation. It should settle:

- the smallest useful online loop, likely direct challenge before discovery or
  matchmaking;
- authoritative game state and where the Rust rules engine runs server-side;
- reconnect, timeout, abandonment, and concurrent-session behavior;
- how local, signed-in private, trusted online, and explicitly public records
  remain distinct;
- hosting, operational cost, abuse boundaries, and deployment ownership.

Likely work:

- direct challenge or live PvP flow;
- trusted match authority using the Rust rules engine server-side;
- server-verified match history;
- explicit replay publish/share flow;
- public replay pages;
- lightweight public identity only where sharing requires it.

Eventual release bar:

- two people can reliably play a full online game;
- the product clearly distinguishes local history, signed-in private history,
  trusted online history, and explicitly published public replays.

## Parking Lot

These are plausible but should not quietly enter the current line:

- puzzle generation from analyzed games;
- "save this game" challenges from losing replay positions;
- bot personalities beyond current Easy/Normal/Hard controls;
- stronger server-side bot endpoint if browser-side wasm becomes too slow;
- broader theme/skin work after the public story is stable.

## Non-goals For Now

- native mobile apps;
- chat/social feeds;
- SSR/server-rendered app shell;
- monetization;
- ranked/esports scope before basic online play is valuable;
- forcing cloud or online into the default local flow.
