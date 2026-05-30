# Roadmap

This roadmap tracks current sequencing. Completed phase detail lives in
[`release_history.md`](../../archive/release_history.md); this file should stay
short enough to answer "what is next and why?"

## Current Line: `v0.5` Public Release Reconciliation

Goal: turn the heavy `0.4` lab/analyzer foundation into a clean public alpha.

`0.5` is intentionally not another broad research line. It is about explaining
the product, reducing repo noise, converting reports/static pages into durable
web surfaces, and preparing a first public-facing release package.

Done so far:

- compact report JSON and a web-rendered `/lab/` report viewer;
- `/rules/`, `/guide/`, `/visuals/`, SPA Privacy/Terms, and Source link;
- replay-analysis surfacing through the product flow;
- docs/repo/API cleanup passes after the `0.4` lab work.

Remaining likely slices:

- finish docs and artifact hygiene so source remains readable;
- polish public copy and screenshots around the "lab under the board" story;
- keep report pages understandable without turning them into telemetry dumps;
- prepare README, release notes, screenshots, and public packaging.

Release bar:

- local play, settings, replay analysis, reports, sign-in, mobile, and
  no-config fallback all smoke cleanly;
- active docs describe current behavior without archaeology;
- generated artifacts do not dominate the repo;
- public pages explain the unusual features without requiring external docs.

## Next Product Line: `v0.6` Online Product Expansion

Goal: turn the cloud foundation and product identity into online human play and
trusted/shareable records.

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
