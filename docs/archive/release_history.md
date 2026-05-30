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
