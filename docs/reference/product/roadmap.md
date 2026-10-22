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
open and are not recorded as passes. The prepared `0.5.5` patch packages the
completed review and refreshed media. Its [release record](../../archive/v0_5_5_release_preparation.md)
keeps tagging/deployment separate from preparation; see
[GitHub Releases](https://github.com/byebyebryan/gomoku2d/releases/latest) for the
currently published build.

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

Multiplayer is deferred from v0.6 as of October 21. Its implementation and UX
effort, uncertain usage, and difficulty demonstrating it with another player
make it a weaker immediate fit for this demo/experiment. The next player-facing
slice remains under discussion. Preserve the chosen visual language and
existing screen roles when choosing that slice.

## Deferred Multiplayer Research

The [P2P research](../backend/p2p_multiplayer.md) preserves the preferred backend
direction if multiplayer is revisited: direct WebRTC gameplay, validation in
both Rust/Wasm clients, and a small backend role for finding/joining peers and
signaling. Optimize for minimal backend usage as gameplay volume grows; ordinary
moves should produce no server or database operations. Nothing is implemented
or deployed.

An initial PoC would accept direct-connection failures on some networks. TURN
and database move relaying remain separate decisions. An established game should
remain playable without signaling while its peer connection remains usable.
P2P results would be casual history, not server-verified records.

UI/UX questions to resume with the feature:

- entry, invitation, waiting, joining, and rules/colors/readiness;
- clear player turns, move acknowledgement, and connection status;
- connection failure, reconnect, leaving, and interrupted sessions;
- completion, rematch, replay, and optional private history continuity;
- identity requirements and the treatment of hints, undo, and other assists.

Future multiplayer acceptance should cover a complete two-player game on
supported network paths, honest failure/recovery states, and measured signaling
usage. Specific criteria follow its UI/UX and protocol design; no player capacity
or connectivity success rate is claimed yet. Ranked/trusted match authority and
wider discovery are outside the preferred PoC.

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
