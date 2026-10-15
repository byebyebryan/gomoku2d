# Roadmap

This roadmap tracks current sequencing. Completed phase detail lives in
[`release_history.md`](../../archive/release_history.md); this file should stay
short enough to answer "what is next and why?"

## Current State: `v0.5` Reconciliation Reopened

`v0.5.4` marked the July public-release reconciliation checkpoint. The product
carries a complete local loop across play, configurable bots and hints, saved
history, Replay Analysis, and the public Lab,
Rules, Guide, and Visuals surfaces. The repository has current ownership maps,
bounded tests, clean release operations, and archived historical context.

The closeout did not hide another bot-research or redesign phase. Curated report
data and analyzer semantics remain unchanged; the work clarified ownership,
fixed demonstrated resilience gaps, and validated the shipped product on
desktop and mobile.

The October post-pause review reopened `0.5.x`. The first repair batch addressed
demonstrated cloud sync, persistence, analyzer-confidence, bot-recovery, and
keyboard-access gaps. The active [deep reconciliation and refinement plan](../../working/v0_5_reconciliation.md)
now covers the whole product: product assumptions, rules/tactics, architecture
and data contracts, cloud continuity, UX/visual/accessibility refinement,
browser performance, test quality, and repository/release operations.

Existing decisions and recent fixes receive fresh scrutiny. Review first,
choose coherent refinements, implement bounded batches, then establish whole-
product acceptance before closing `0.5.x` or starting the next feature cycle.

Historical execution and findings:

- [`v0.5.4 Reconciliation Closeout Plan`](../../archive/v0_5_4_reconciliation_plan.md)
- [`v0.5.4 Reconciliation Findings`](../../archive/v0_5_4_findings.md)
- [`v0.5 Public Release Plan`](../../archive/v0_5_public_release_plan.md)

## Following Reconciliation: `v0.6` Design Checkpoint

After the reopened `v0.5` pass, choose the next player benefit at a fresh design
checkpoint. Online expansion remains the existing candidate. If selected, settle:

- the smallest useful online loop, likely direct challenge before discovery or
  matchmaking;
- authoritative game state and where the Rust rules engine runs server-side;
- reconnect, timeout, abandonment, and concurrent-session behavior;
- how local, signed-in private, trusted online, and explicitly public records
  remain distinct;
- hosting, operational cost, abuse boundaries, and deployment ownership.

Candidate online work:

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
