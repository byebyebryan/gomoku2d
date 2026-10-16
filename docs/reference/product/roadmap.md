# Roadmap

This roadmap tracks current sequencing. Completed phase detail lives in
[`release_history.md`](../../archive/release_history.md); this file should stay
short enough to answer "what is next and why?"

## Current State: October `0.5.x` Review Complete

`v0.5.4` marked the July public-release reconciliation checkpoint. The product
carries a complete local loop across play, configurable bots and hints, saved
history, Replay Analysis, and the public Lab,
Rules, Guide, and Visuals surfaces. The repository has current ownership maps,
bounded tests, clean release operations, and archived historical context.

The closeout did not hide another bot-research or redesign phase. Curated report
data and analyzer semantics remain unchanged; the work clarified ownership,
fixed demonstrated resilience gaps, and validated the shipped product on
desktop and mobile.

The October post-pause review reopened `0.5.x` and is now complete on `main`.
It repaired cloud/session and persistence failures, rule and payload boundaries,
analyzer-confidence presentation, bot recovery, keyboard access, accessibility,
worker ownership, and route recovery. The systematic code review completed
targeted history and lifecycle cleanup while retaining the main architecture.

Bryan chose to keep the existing UI and copy. Only the Rules/Guide mobile layout
refinement was selected from the visual study; broader mocks are not an accepted
backlog. The [October closeout](../../archive/reconciliation_2026_10/README.md)
links the decisions, code review, measurements, and verified runtime baseline.

Manual account/device/accessibility/human checks remain explicitly deferred at
Bryan's request. These deferrals do not hold the review or next design checkpoint
open and are not recorded as passes. A new patch release, media refresh, and
deployment remain separate decisions; the public release is still `v0.5.4`.

The bounded [unattended follow-up](../../archive/reconciliation_2026_10/v0_5_unattended_evidence.md)
is also complete: two-client Auth/Firestore application acceptance in all three
engines, a repaired no-op sync cooldown, and broader production Wasm performance,
active cancellation and recovery evidence. Exact implementation CI and all 45
fresh-state race repetitions passed. The chosen UI, closed Renju review, and
existing manual acceptance deferrals remain in force. Return to release
packaging and the next design checkpoint; no further review phase is implied.

Historical execution and findings:

- [October review and closeout](../../archive/reconciliation_2026_10/README.md)
- [`v0.5.4 Reconciliation Closeout Plan`](../../archive/v0_5_4_reconciliation_plan.md)
- [`v0.5.4 Reconciliation Findings`](../../archive/v0_5_4_findings.md)
- [`v0.5 Public Release Plan`](../../archive/v0_5_public_release_plan.md)

## Following Reconciliation: `v0.6` Design Checkpoint

Choose the next player benefit at a fresh design
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
