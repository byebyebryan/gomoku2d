# Retrospective UI capture pilot

Owner: Bryan. Authorized October 25, 2026 with the luna-goal-loop skill.
This is the bounded v0.2.4 / v0.6.2 pilot, not a bulk history backfill or release.
Implementation and validation are tracked below as work completes.

## Outcome and boundaries

Create comparable retrospective captures of Home, Match, Replay and Profile,
then curate the verified results as one UI-evolution entry in the local developer
area. Original historical media and the frozen 0.6.2 reference stay unchanged.
Older tagged application code, assets and Wasm retain their actual appearance
and behavior; compatibility work belongs to the external capture/build harness.
No new backend, persisted product fields, gameplay changes, dependencies in the
application, push, tag or production deployment is authorized by this pilot.

The current manual demo/emulators and Carbon access services remain owned by
their existing user services. Capture work uses owned detached worktrees,
separate dependencies/builds/ports and guest synthetic data. No real accounts,
private history or production cloud data are capture inputs.

## Capture contract

- Sources: exact v0.2.4 and v0.6.2 tag commits, source-tree and lockfile identities.
- Scenes: Home, populated but idle Match where supported, a finished Replay and
  a nonempty Profile. Use the same names, rules and game moves where supported;
  record schema/route adapters and any meaningful content differences.
- Desktop: 1280 × 900 CSS pixels, DPR 1, desktop Chromium.
- Portrait: provisional generic 412 × 800 CSS pixels, DPR 1, mobile/touch context
  with an explicitly recorded generic Android user agent. This is not a measured
  OnePlus/browser-chrome profile. Physical calibration remains pending.
- Both profiles: 100% text, reduced motion, screenshot scale `css`, viewport
  capture (not an arbitrary full-page image), font readiness and stable scene
  assertions before capture. Record browser/renderer observations and unknowns.
- PNG output is never cropped, recoloured, stretched or redrawn after capture.
  The viewer's layout owns presentation scaling; original images remain available.
- Each run gets a new directory and manifest: source/build/tool identities,
  viewport/context/DPR, fixtures/adapters, capture time, observed route/state,
  dimensions, hashes and network/error observations. Refuse overwrite.
- New images are labelled retrospective captures, with version and capture date;
  they do not establish original-period, human or physical-device acceptance.

The provisional portrait profile lets the workflow pilot proceed. It does not
settle the final Android reference size or replace the pending phone readout.

## Checkpoints

1. Inspect tag-specific build, route and fixture contracts; settle external adapters.
2. Implement the reusable capture tool with explicit inputs/outputs and failure
   cleanup; build exact old/new sources in owned worktrees.
3. Capture both profiles/scenes, inspect the actual images and verify manifests,
   unchanged source trees and absence of unintended network/private inputs.
4. Add a curated UI-evolution pilot entry, preserving current-reference priority
   and all existing review URLs. Reuse the current viewer and standalone exporter.
5. Run appropriate type/unit/build and browser/source-media checks, fix findings,
   commit locally and verify the integrated Carbon preview. Close owned resources.

## Execution

Investigation started. Luna owns the bounded tag/build/fixture investigation;
the primary owns capture contracts, orchestration, integration and acceptance.
