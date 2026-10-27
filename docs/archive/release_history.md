# Release History

Historical phase notes. This file preserves the broad release arc without making
the active roadmap carry every completed detail.

## `v0.1` POC

Validated the stack: Rust rules, a basic bot loop, wasm bridge, Phaser browser
board, and GitHub Pages deployment.

It proved that the old core idea could become a browser product without
throwing away native game logic.

Historical state: [`progress_v0.1.md`](progress_v0.1.md).

## `v0.2` Frontend Foundation And Local Play

Turned the prototype into a real local-first frontend product:

- React app shell and route structure;
- Phaser reduced to board rendering;
- local profile, local settings, local history, replay viewer, and replay
  branching;
- desktop and mobile match layouts;
- screenshot review, visual references, and release hygiene.

The durable lesson was the React/Phaser/Rust boundary: agents can work in one
layer without constantly breaking another when ownership is clear.

## `v0.3` Backend Foundation And Cloud Continuity

Added optional cloud continuity without putting cloud in front of the local game:

- Firebase Auth with Google sign-in;
- Firestore `profiles/{uid}` for owner-scoped profile/settings/history;
- local profile-to-cloud promotion;
- capped private cloud history;
- reset/delete barriers and rules tests;
- Privacy/Terms pages and production sign-in readiness.

The durable lesson was the trust-lane split: local guest history, signed-in
private cloud history, future trusted online history, and explicit public
sharing are different products and must not be collapsed.

## `v0.4` Lab-Powered Product Identity

Made the Rust lab visible as a product differentiator:

- explicit `search-*` bot specs, scenario diagnostics, tournaments, reports,
  and search metrics;
- configurable Easy/Normal/Hard bots with controlled advanced settings;
- rolling threat facts and pattern-frame caching as default hot-path support;
- corridor search as the shared strategic model for replay analysis;
- in-product replay analyzer with setup corridor, lethal onset, last escape,
  and mistake-aware labels;
- exact Renju forbidden-move checking backed by extracted/reference fixtures;
- curated bot and analysis reports published as product surfaces.

The durable lesson was that raw bot-strength tuning had diminishing returns,
while explainable play became the stronger product direction. Corridor search
did not work as a broad live bot shortcut under browser-scale compute, but it
became the right foundation for replay explanation and tactical vocabulary.

Detailed historical plans remain in the archive:

- [`v0_4_plan.md`](v0_4_plan.md)
- [`v0_4_search_bot_enhancement_plan.md`](v0_4_search_bot_enhancement_plan.md)
- [`v0_4_3_corridor_bot_plan.md`](v0_4_3_corridor_bot_plan.md)
- [`v0_4_4_frontier_plan.md`](v0_4_4_frontier_plan.md)
- [`v0_4_5_bot_controls_plan.md`](v0_4_5_bot_controls_plan.md)
- [`v0_4_6_replay_analysis_plan.md`](v0_4_6_replay_analysis_plan.md)
- [`v0_4_7_lethal_threat_plan.md`](v0_4_7_lethal_threat_plan.md)

## `v0.5` Public Release Reconciliation

Through `v0.5.4`, turned the lab-heavy `0.4` foundation into a coherent,
maintainable public product:

- replaced generated report HTML with compact curated JSON rendered by the
  first-class `/lab/` web surface;
- added concise Rules, Guide, Visuals, Privacy, and Terms pages inside the app;
- made replay analysis easier to discover from the normal product flow;
- refreshed gameplay, analysis, lab, visual, and social-preview media;
- reconciled web/lab APIs, tests, docs, generated artifacts, CI, and release
  runbooks after the long analyzer research line;
- preserved private process-story evidence without making a devlog a condition
  of shipping.
- completed a repository-wide ownership, test, dependency, operations, and
  fresh-player review without changing the promoted bot or analyzer model;
- closed demonstrated worker/cache resilience gaps and left native, Wasm, web,
  rules, dependency, and browser validation green.

The durable lesson was operational: a personal process story can strengthen a
release, but it cannot be the release gate. The product and repository should be
able to ship first, with longer-form writing remaining an optional follow-up.

`v0.5.3` remains the first public-alpha checkpoint. `v0.5.4` completes the
reconciliation line and leaves `v0.6` to begin from a fresh product-design
checkpoint rather than inherited cleanup.

Historical plan: [`v0_5_public_release_plan.md`](v0_5_public_release_plan.md).
Closeout plan:
[`v0_5_4_reconciliation_plan.md`](v0_5_4_reconciliation_plan.md).

`v0.5.5` packaged the October correctness, recovery, accessibility and mobile
Rules/Guide closeout. Its [preparation record](v0_5_5_release_preparation.md)
retains the packaging evidence and manual acceptance limits.

## `v0.6` Shareable Replays And Public Profiles

Published as `v0.6.1`, directly after `v0.5.5`: self-contained finished-game
links, explicit private replay saves, ordinary continuation games, and opt-in
public profiles with dated statistics and bounded game catalogs. Sharing and
publishing use the existing UI patterns; the release also fixes board colours
across Canvas and WebGL.

Public sharing remains separate from private history and cloud continuity.
Gameplay and analysis stay in the browser, and multiplayer remains deferred.
The [release record](v0_6_1_release.md) records the verified deployment,
automated production acceptance, pending real signed-in sharing round trip,
and retained human/device deferrals.

`v0.6.2` refined public profiles, mobile controls and replay return, then added
Guide sharing help and account-free examples. Its
[release record](v0_6_2_release.md) retains the production and acceptance evidence.

`v0.6.3` established the static Dev site: Visual design inspectors, current screen
references, a milestone overview and screen histories, prepared design reviews,
and the existing Bot lab. It presents source, review findings and actual working
tools while preserving historical capture identities and their limits. The
[release record](v0_6_3_release.md) separates exact-source publication and
production automation from the still-open account/device/human checks.
