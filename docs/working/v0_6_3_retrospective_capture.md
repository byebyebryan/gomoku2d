# Retrospective UI capture pilot

Owner: Bryan. Authorized October 25, 2026 with the luna-goal-loop skill.
This is the bounded v0.2.4 / v0.6.2 pilot, not a bulk history backfill or release.
Implementation and local validation are complete. Human review and phone
calibration remain separate acceptance steps.

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

Luna completed the tag/build/fixture investigation without application edits.
Both versions support the same deterministic freestyle win. Their storage keys,
schemas and Replay routes differ; external adapters will express those differences.
Replay at ply 8 can start a populated Match with Black/human to move, avoiding
live bot timing. Luna implemented the capture and build-receipt CLIs, guarded
preflight and focused tests. The primary owned tagged builds, media orchestration,
developer-site integration and overall review.

The primary built both peeled tag commits in separate owned worktrees using
wasm-pack 0.14.0, Rust 1.99.1, locked Cargo dependencies, Node 26.12.1 and npm
12.2.1. Each used its own `npm ci` and tag-specific production build/postbuild.
No application or lockfile edits were required; tracked worktrees remained clean.
The declared Node versions were 20 and 24 respectively; the actual modern host
runtime is a recorded deviation, not a claim of reproducing original binaries.
The v0.2.4 and v0.6.2 outputs contained 90 and 147 files. Local receipts retain
all file hashes and tool/configuration details.

Both builds deliberately omit Firebase configuration: the scene is an offline
local guest, and v0.6.2's cloud sign-in is disabled. This is an explicit capture
condition, not a reproduction of the cloud-configured production account state.
No Auth/Firestore emulator or production backend is needed for these four scenes.

An initial old-version preview inherited its historical auto-open behavior and
tried to start unavailable Firefox; no user tab opened. It was restarted with
`BROWSER=none`. Owned preview ports are 46924 and 46962; the integrated Carbon
preview and existing manual services remain separate.

## Qualified pilot

[The frozen pilot](v0_6_3_retrospective_capture/pilot-2026-10-25/README.md)
contains 16 PNGs and the complete capture manifest. Its source commits are
`d9a3476d11dcb24e1c3fa67374a27b867a781da1` (v0.2.4) and
`37a122d2bd8e899bcfc581055e685c0327f43cb4` (v0.6.2). The manifest identifies
the exact harness, browser, build, fixture, viewport and each image hash.

The single curated entry, **From local play to replay analysis**, lists eight
scene/profile comparisons. It appears under UI evolution, after Current screens
and before Design & layout reviews. Existing references and review URLs are
unchanged. Open on Carbon:
`http://localhost:46862/dev/reviews/ui-evolution-024-062/`.

The first capture attempt stopped when v0.2.4 opened Replay at move 4 rather than
the expected final move. The adapter now seeks to move 9 with the supported
timeline End key, waits for newer analysis to settle, then steps back once to
branch Match at move 8. Compact account controls are observed without opening
their disclosure. Failure output was retained locally; it was not curated.
The successful run recorded no blocked/failed requests, console or page errors
in any of its four isolated contexts.

All 16 images were visually inspected and independently checked against manifest
hashes and PNG dimensions. Both sources stayed clean. Visible constraints are
recorded honestly: the rightmost desktop Profile record action clips at 1280px
in both tags, and the long synthetic name truncates the later Match status.
Those are follow-ups for the product, not reasons to modify historical sources.
The later UI adds replay analysis, sharing and distinct Profile collections
while retaining the title screen's visual identity.

## Reproduce a new capture

The scripts are intentionally limited to these two adapters. Add a new adapter
only after inspecting that tag's actual routes, persistence and readiness.
Never reuse an existing capture directory or edit a frozen manifest. New runs
can differ in canvas animation phase even with identical inputs.

1. Create two **new detached worktrees** at the exact commits above. Give each
   its own dependency install and Cargo target directory. Verify tag resolution
   and clean tracked/untracked source before building. Leave other worktrees and
   the original manual preview/emulators alone.
2. Build that tag's Wasm with `wasm-pack build gomoku-bot-lab/gomoku-wasm
   --target bundler -- --locked`, using an isolated `CARGO_TARGET_DIR`. Then run
   `npm ci --no-audit --no-fund` and `npm run build` in its `gomoku-web` directory.
   Prefix shell commands with `rtk` according to the workspace instructions.
   Clear inherited `VITE_*` and `GOMOKU_BASE_PATH` variables for both builds and
   previews, and record actual tools and install-script policy. Both builds must
   use unconfigured Firebase/local guest mode for this adapter.
3. Immediately record each build with the maintained read-only receipt CLI.
   It does not install, compile or start previews. The environment values are
   operator attestations of the build just performed, not autodetected evidence
   of an older build. For example, the v0.2.4 build config is:

   ```json
   {
     "schemaVersion": 1,
     "tag": "v0.2.4",
     "expectedCommit": "d9a3476d11dcb24e1c3fa67374a27b867a781da1",
     "worktree": "/absolute/owned/worktrees/v024",
     "environment": {
       "node": "ACTUAL_NODE_VERSION",
       "npm": "ACTUAL_NPM_VERSION",
       "rustc": "ACTUAL_RUSTC_VERSION",
       "wasmPack": "ACTUAL_WASM_PACK_VERSION",
       "firebaseMode": "unconfigured-local-guest",
       "declaredNode": "20",
       "wasmBuild": "release --target bundler --locked",
       "npmInstall": "ACTUAL_INSTALL_COMMAND_AND_POLICY"
     }
   }
   ```

   From the current checkout's `gomoku-web` directory:

   ```sh
   rtk proxy npm run capture:retrospective:receipt -- --config /absolute/build-v024.json --output /absolute/new-receipt-v024.json
   ```

   Repeat with v0.6.2, its exact commit and declared Node 24. Receipt outputs
   must be new JSON files outside their source worktrees. They include sorted
   dist file hashes, Wasm/lockfile identities and source trees; the aggregate
   dist digest hashes `JSON.stringify(sortedFiles) + "\n"`.
4. Start each tag's built preview on a separate, available loopback port using
   `BROWSER=none` and `--strictPort`. The pilot used 46924 and 46962. For example,
   from the old worktree's `gomoku-web` directory, after clearing build variables:

   ```sh
   rtk proxy env BROWSER=none npm run preview -- --host 127.0.0.1 --port 46924 --strictPort
   ```

5. Create an external capture config with `schemaVersion: 1` and two `sources`.
   Each source has `tag`, full `expectedCommit`, absolute `worktree`, root
   loopback `baseUrl`, `adapter` (`v024` or `v062`) and `buildReceipt` containing
   the **parsed receipt object**, not its file path. All expected source/build
   identities and served index bytes must match before capture starts.
6. From the current checkout's `gomoku-web`, validate without writing, then
   capture into the same still-nonexistent output path:

   ```sh
   rtk proxy npm run capture:retrospective -- --config /absolute/sources.json --output /absolute/new-run --validate-only
   rtk proxy npm run capture:retrospective -- --config /absolute/sources.json --output /absolute/new-run
   ```

   The output parent must already exist. Output must be disjoint from the
   current checkout, both source worktrees and config file; symlinks are
   canonicalized. If the host lacks browser libraries, run in the project's
   Playwright container with host networking and mount the current checkout,
   owned worktree root and output parent at the **same absolute paths**. Detached
   worktree Git pointers otherwise cannot resolve. Use the installed Playwright
   package and its matching browser cache, not arbitrary image-bundled browsers.
   The manifest records the actual capture runtime/browser separately from the
   build host. Close only owned browsers/previews after the run.
7. Inspect `status: complete`, exactly 16 captures, all expected fixture states,
   source cleanliness and error arrays. Independently verify image hashes and
   dimensions, then inspect every image. Copy qualified files byte-for-byte to a
   new dated source directory and add an allowlisted review record. Preserve
   failed runs separately; a failed run is not a reference dataset.

Focused tool verification:

```sh
rtk proxy node --test scripts/capture_retrospective_ui.test.mjs
```

The scripts reject source/adapter/port mismatches, source changes, unexpected
served builds, unsafe output paths and overwrite attempts. Capture blocks
off-origin/data-identity traffic, mutation requests and WebSockets. Browser
contexts close on failure; an actionable failure manifest remains available.
These checks protect this local workflow, rather than establish a sandbox for
arbitrary untrusted historical programs.

## Validation and acceptance

The [pilot validation](v0_6_3_retrospective_capture/pilot-2026-10-25/validation.json)
records source/artifact hashes, result scopes and retained limitations.

- Type checks, 527 unit tests and production build/postbuild pass.
- Six focused Node checks pass; live negative checks reject a different tag's
  preview before writing and reject an existing run without changing its 17 files.
  The maintained receipt generator agrees with both independently recorded builds.
- All 30 developer journeys pass across Chromium, Firefox and WebKit, including
  version pairing, image scale, deep-link focus, reload and 320px/200% text.
- The same pilot's standalone export passes all three engines, including history,
  reload, native/full inspection and narrow text. This is a local export check;
  no new public report was published.
- An 18-view responsive/accessibility audit has no overflow or automated
  violations. Six incomplete colour-contrast results still require manual
  judgment. Automation does not establish screen-reader or physical-device
  acceptance.
- Source/media delivery covers 126 routes/files and confirms existing recorded
  Lab reports remain unchanged. Carbon delivery is separately hash-checked against
  every file in the integrated build.

Bryan's review of the history presentation is the next gate before expanding
the milestone list. Phone viewport/DPR/browser-chrome calibration remains open.
No bulk backfill, application repair, release or production publication is part
of this pilot.
