# Roadmap

This roadmap tracks current sequencing. Completed phase detail lives in
[`release_history.md`](../../archive/release_history.md); this file should stay
short enough to answer "what is next and why?"

## Current Line: `v0.6` Online Product Expansion Planning

`v0.5.3` closes the public-release reconciliation line. The product, lab
reports, explanation pages, repository, and release media now form a coherent
public alpha; the optional process-story material is no longer a release gate.

Goal: turn the cloud foundation and product identity into online human play and
trusted/shareable records.

The next step is a fresh design checkpoint, not immediate implementation. It
should settle:

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

Release bar:

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
