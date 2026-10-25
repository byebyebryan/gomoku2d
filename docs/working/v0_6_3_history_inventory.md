# v0.6.3 UI history inventory

Owner: Luna Max. This inventory records source evidence from the actual release
tags; catalog, lifecycle, and layout decisions remain with the primary. The
source audit itself was read-only and did not edit application code.

## Coverage and recommendation

There are 25 actual tags in scope. There is no `v0.6.0` tag. Every actual
release should appear in the release provenance list, even when it does not
need its own UI milestone. I recommend milestones for all tags below except
`v0.5.0`, whose changelog and route diff describe runtime/refactoring and
report refreshes without a distinct user-visible screen or meaningful new
normal-page state. Keep its report data as that source's report identity where
the Lab trail uses it, and record the caching/runtime work in provenance. This
adds `v0.4.3`, `v0.4.4`, `v0.5.3`, and `v0.5.4` to the proposed density set;
it does not add a `v0.5.0` milestone.

The source is the peeled commit behind each annotated tag, not the annotated
tag-object hash. Release dates below are from `CHANGELOG.md`.

| Release | Date | Source commit | UI/report evidence and milestone recommendation |
|---|---|---|---|
| `v0.2.1` | 2026-04-22 | `add1953c3228849a8d37c424d0deec971c1fc937` | First React baseline for Home, Local Match, Profile, and Replay; guest profile and local history. Include as the opening milestone. `CHANGELOG.md` 1203–1230; `gomoku-web/src/app/App.tsx`, `src/routes/*`, `src/profile/guest_profile_store.ts`. |
| `v0.2.2` | 2026-04-22 | `a0a87e2adaf31f50c5e45f245c9765b09eac6fbc` | Noticeable flatter shell, clearer action roles, Profile ledger, live undo and Replay → Play From Here. Include; establishes a useful changed state for Home/Match/Profile/Replay. `CHANGELOG.md` 1166–1202; Home/LocalMatch/Profile/Replay routes and styles. |
| `v0.2.3` | 2026-04-22 | `95b9707a6c01d77fa50d181bdd1ee4542047a2a2` | Authored portrait layouts for Match/Replay/Profile, touch-placement flow, icon language, pending-rule note. Include. `CHANGELOG.md` 1131–1165; LocalMatch/Replay/Profile CSS, `src/components/Board/Board.tsx`, board input/scene code. |
| `v0.2.4` | 2026-04-24 | `d9a3476d11dcb24e1c3fa67374a27b867a781da1` | Shell polish, Home version label, Practice Bot name, short-viewport scrolling, board overlay/sprite changes, asset preview publishing. Include; this is the existing pilot endpoint for the first group. `CHANGELOG.md` 1051–1130; Home/Match/Replay routes, `gomoku-web/assets/`, `scripts/publish_asset_previews.mjs`. |
| `v0.3.0` | 2026-04-27 | `5e44832d872cc1030be6707b97dcd9f9b067d83c` | Adds Google identity and local/cloud availability states on Profile; Privacy and Terms static pages and Home links. Include. `CHANGELOG.md` 999–1050; `src/cloud/firebase.ts`, Profile/Home routes, static `gomoku-web/public/privacy/` and `public/terms/`. |
| `v0.3.1` | 2026-04-28 | `b9fcc3df81cc2ccea24140a3556db40c1929c5f7` | Guest-to-cloud promotion/import progress and identity-bearing saved-match rows, while guest history remains local. Include a cloud/Profile transition state. `CHANGELOG.md` 941–998; cloud promotion/profile stores, Profile and Replay routes. |
| `v0.3.2` | 2026-04-28 | `3a8dbb05d94f8811e995ae7621329a390a313954` | Signed-in cloud history becomes Profile/Replay's active history; visible pending/sync recovery badges and inline reset confirmation. Include. `CHANGELOG.md` 888–940; Profile/Replay/LocalMatch routes and `cloud_history_store.ts`. |
| `v0.3.3` | 2026-04-29 | `73c1698667ddb85085feedf42c7f2cc2a9de7073` | Profile adds cloud delete, sync badges, Show more, reset-scope wording; saved replay URL changes from `/replays/local/:matchId` to `/replay/:matchId`. Include. The path rename is not a new screen identity. `CHANGELOG.md` 821–887; App route, Profile/Replay, `profile/local_profile_store.ts`. |
| `v0.4.0` | 2026-05-03 | `0fc4a45b79c5056cefa5e7485599c0eef5e6295a` | Adds the static Bot Lab Report with rankings/details, match trees and provenance. The tagged `.4.0` page has no separate interactive Search or Pairwise controls; those arrive in `.4.1`. Include as a new report screen. `CHANGELOG.md` 770–820; `gomoku-bot-lab/reports/index.html`, `latest.json`, `gomoku-web/scripts/publish_bot_reports.mjs`. |
| `v0.4.1` | 2026-05-04 | `aaa6ac548fc2f3fc8c1a3815abd262473823be6f` | Bot Report gets a larger ranking/search/pairwise ladder, expandable comparisons, mobile table and navigation from Game/Assets. Include as an update to Bot Report. `CHANGELOG.md` 731–769; report HTML/JSON and Home/asset-preview links. |
| `v0.4.2` | 2026-05-08 | `5e132248d9602ab0ce3abbdf86da8f92a3fa0e26` | Adds standalone Analysis Report with board proof, threat markers and per-turn explanations; Bot Report remains. Include Analysis Report as added and Bot Report as updated. `CHANGELOG.md` 667–730; `gomoku-bot-lab/analysis-reports/{index.html,latest.json}`, report publishers. |
| `v0.4.3` | 2026-05-12 | `dba0fb4213520d30f3640729cccaad0ceb5ccc58` | No React route change, but the published Bot Report visibly adds `Portal` and `Exit` table metrics and updated run/results; Analysis Report data/presentation is also regenerated. Include as a Bot Report/Analysis Report update. This is not only a Rust change: `git show v0.4.3:gomoku-bot-lab/reports/index.html` contains the added Portal/Exit columns. `CHANGELOG.md` 618–666. |
| `v0.4.4` | 2026-05-14 | `9744aa2576860fbc3af14148504cb2144ef69b75` | Bot Report changes from depth/breadth/avg nodes to pooled budget plus stage columns (Move gen, Ordering, Scoring, Threat detection, Proof, Other, TT, Pool); Analysis Report is refreshed. Include both report updates. `git show v0.4.4:gomoku-bot-lab/reports/index.html` directly shows the new labels and pooled reserve; `CHANGELOG.md` 570–617. |
| `v0.4.5` | 2026-05-16 | `b97dcff422350ebe78bb0d2cf5061ce52a408ed0` | Adds the Settings route, presets/Custom controls, hint controls, timers and touch options; removes configuration controls from Profile. Include Settings added and Profile updated. `CHANGELOG.md` 523–569; `src/app/App.tsx`, `SettingsRoute.tsx`, Profile/LocalMatch routes, `profile_settings.ts`. |
| `v0.4.6` | 2026-05-17 | `2ec50c02f29deb02c14295d07929e9f7e835fd2b` | Adds progressive browser Replay Analysis on the existing Replay screen: analysis status/timeline, last-escape and board evidence. Include as Replay's Analysis state added, not as another Replay screen. `CHANGELOG.md` 475–522; Replay route, `replay_analysis_*`, board overlay/sprite code. |
| `v0.4.7` | 2026-05-19 | `0b93fed092975c172959343a45decc9c8c89fe2e` | Replay timeline separates setup corridor, lethal tail, last escape and lethal onset; status copy changes with player/frame. Include as a Replay Analysis update. `CHANGELOG.md` 422–474; Replay route/styles and analysis timeline. |
| `v0.4.8` | 2026-05-21 | `ea188acaf3e98b0b0b0952c05993391bc8a8a1ef` | Adds mistake classification and configurable evidence overlays to Replay Analysis; Bot Report pairwise/columns also change. Include updates to Replay Analysis and Bot Report. `CHANGELOG.md` 373–421; Replay/LocalMatch/Settings route and board overlay code, report HTML/JSON. |
| `v0.5.0` | 2026-05-22 | `678a69ac51f2b1f4fa968bf9fe03b69831b847c9` | Refactors route/board/replay/cloud plumbing, adds replay-analysis caching and browser bot memory cap, and refreshes the report baseline. Changelog explicitly frames this as foundation/refinement rather than a new strategy or UI. Do not make a standalone milestone unless primary finds a distinct source-visible state worth showing. `CHANGELOG.md` 326–372; large route refactor, no new route. |
| `v0.5.1` | 2026-05-25 | `bf861f596e7a5bf03b588953f6552b9a3c4c7827` | Generated static Bot/Analysis Report HTML becomes React Lab backed by compact JSON; static asset previews consolidate into React Visuals within the same logical Visuals family. Include Bot Report and Analysis Report as retired/merged into Lab; record Visuals as updated, not removed and re-added. `CHANGELOG.md` 289–325; `src/routes/BotReportRoute.tsx`, `AssetPreviewRoute.tsx`, App redirects, `gomoku-bot-lab/{reports,analysis-reports}/report.json`. |
| `v0.5.2` | 2026-05-27 | `71793b0f21adf408b5b714514937d979ecffc944` | Adds Rules and Guide routes with live board-rendered teaching examples; `/lab/` and `/visuals/` become the canonical support URLs and replay entry labels become Analyze/Inspect/Replay Analysis. Include Rules and Guide as added, plus Lab/Visuals/Replay updates. URL renames do not create new screen identities. `CHANGELOG.md` 257–288; Rules/Guide routes, App, Home/Profile/Replay. |
| `v0.5.3` | 2026-07-22 | `8fa3035f2e2e7445907c1d74903d27b1fa072c1a` | Privacy and Terms move from static pages into `PolicyRoute` at the same public URLs; Home/product-first presentation, Visuals and direct Lab analysis links are refined. Include as a visible policy/navigation milestone, preserving Privacy/Terms screen identities. `CHANGELOG.md` 218–256; App, PolicyRoute, Home/Visuals/Lab routes, removed static pages. |
| `v0.5.4` | 2026-07-23 | `b09ac7ff56f1452bccdf63334ce38415b92fdbd8` | Small but real visible reconciliation: Profile empty/reset copy and “Play a game” action, policy text/date, Settings/Rules/Guide wording, Visual-system label, and Lab preset badges/spec readability. Include a concise updated-state milestone; don't present the report component split as a visual redesign. `CHANGELOG.md` 171–217; source diff in Profile/Policy/Settings/Rules/Guide/Lab/Visuals routes and Profile CSS. |
| `v0.5.5` | 2026-10-16 | `73658d0a546ba558c746854d38cdac7ec2465802` | Adds visible local-save-unavailable notice, failed-bot retry, keyboard board placement/accessibility, read-only semantics, reduced-motion cues, possible/confirmed markers and mobile Rules/Guide refinement. Include selected new/updated states rather than implying all are a new screen. `CHANGELOG.md` 76–170; `best_effort_storage.ts`, LocalMatch/Replay/Settings/Guide/Rules, and cloud emulator bootstrap. |
| `v0.6.1` | 2026-10-23 | `b1a67d32feee5b57275e3d04c684e27bcd1a571f` | Adds shared replay links, Profile My Games/Saved Replays, opt-in public profile/public catalog, public stats and the Public Player/Shared Replay routes. Include the added Public Player and Shared Replay screens plus Profile/Replay sharing states. `CHANGELOG.md` 40–75; `PublicPlayerRoute.tsx`, `SharedReplayRoute.tsx`, `replay_library_store.ts`, sharing/catalog modules, App. |
| `v0.6.2` | 2026-10-24 | `37a122d2bd8e899bcfc581055e685c0327f43cb4` | Updates the Public Player card/catalog, Profile controls, shared-replay return/focus restoration, and adds two account-free Guide examples. Include as updates to Public Player/Profile/Shared Replay/Guide. `CHANGELOG.md` 15–39; PublicPlayer/Profile/SharedReplay/Guide and public-player navigation. |

## Screen identities and lifecycle

These are source-derived candidate identities and lifecycle edges for primary
catalog selection. Keep views/states on a screen separate from its route. The
existing `App.tsx` route shape shows the local routes stay stable through
`v0.4.4`; `/replays/local/:matchId` becoming `/replay/:matchId` in `v0.3.3` is
just a URL rename.

| Screen identity | Lifecycle and useful comparison states |
|---|---|
| Home | Added at `v0.2.1`; updated at `.2.2–.2.4` (shell, version/Practice Bot), `.3.0` (policy links), `.4.1` (Lab/Assets navigation), `.5.1–.5.4` (product-first Lab/Visuals/Rules/Guide/Policy navigation and copy), `.6.1–.6.2` (share/profile loop and Guide discoverability). Capture Home with the same local history record at each chosen milestone. |
| Local Match | Added at `v0.2.1`; updated `.2.2` (undo), `.2.3–.2.4` (touch and short-screen flow), `.4.5` (hints/timers/settings), `.4.6–.4.8` (matching overlay vocabulary), `.5.5` (keyboard, retry and storage-unavailable states), `.6.1` (share/publish actions). Use Replay → Play From Here at ply 8 from `.2.2` onward for a populated, idle Black-human turn; `.2.1` predates branching and needs its own clean-start Match state. |
| Profile | Added at `.2.1`; record/history at `.2.2`, portrait at `.2.3`, cloud availability/sign-in at `.3.0`, import at `.3.1`, active cloud history/sync/reset at `.3.2`, delete/Show more at `.3.3`; Settings is removed from Profile and moves to a new screen at `.4.5`; empty/reset/local-save states at `.5.4–.5.5`; My Games/Saved Replays and public-profile controls at `.6.1`; refined controls at `.6.2`. Distinguish default guest, no-config, signed-in/sync and reset confirmation states. |
| Replay | Added at `.2.1`; branch/undo-floor at `.2.2/.2.4`; portrait at `.2.3`; identity/cloud history `.3.1–.3.3`; Analysis state added `.4.6`, timeline update `.4.7`, failure/evidence update `.4.8`; caching `.5.0` is not a distinct visual milestone; `.5.2` updates user-facing Analyze/Inspect labels; `.5.5` preserves analysis attribution/escape confidence; `.6.1` adds shared/saved replay context; `.6.2` adds return-to-origin behavior. Prefer final Analysis-ready state and a separate Play From Here state where relevant. |
| Settings | Added at `.4.5` (moved from Profile); `.5.4` visible wording; `.5.5` semantic settings, read-only cues, and failed-bot recovery. Capture Basic/Normal settings and the Advanced/Custom or touch/hints state as named states only where it explains the change. |
| Bot Report | Added as a static ranking/details page at `.4.0`; interactive Search and Pairwise controls appear at `.4.1`. Updated `.4.3`, `.4.4`, `.4.8`, with `.5.0` refreshed data. At `.5.1`, Bot Report is removed as a route and its report view moves into Lab. Catalog it as a retired report screen merged into Lab, not as a new Lab screen for each legacy URL. |
| Analysis Report | Added as a static report at `.4.2`; report content/data updates `.4.3–.4.4` and `.4.8`. At `.5.1`, Analysis Report is removed as a route and its view moves into Lab. Catalog the former screen as merged into Lab, retaining proof/turn-detail as a Lab view. |
| Visuals / Asset Preview | Static per-asset preview pages begin at `.2.4`; `.5.1` consolidates them into the React Visuals view as an update within the same logical Visuals family, with Sprites/Icons/Fonts states. `.5.2` renames `/assets/` to `/visuals/`; treat that as a URL/screen-title update, not a second replacement. |
| Rules | Added at `.5.2`; mobile explanation/diagram layout refined at `.5.5`. Include an added capture and a later mobile-state update. |
| Guide | Added at `.5.2`; marker explanations and mobile layout refined `.5.5`; two account-free sharing/continuation example games at `.6.2`. Include Guide as one screen with `rules/examples/sharing`-type states if the primary catalog needs them. |
| Privacy and Terms | Added as static pages at `.3.0`, copy refined `.3.3`, moved into SPA `PolicyRoute` at `.5.3` at the same `/privacy/` and `/terms/` URLs, with later copy/date updates `.5.4`. Preserve two stable identities and show `.5.3` as an implementation replacement/update, not loss of either page. |
| Public Player | Added at `.6.1` as a public profile/catalog page; updated `.6.2` to two-column card/catalog, compact rows, optional dated stats and recovery states. The route is `/player` with a UID/query-based profile identity, not one path per user. Include `profile`, `catalog`, and optional `statistics` states as needed. |
| Shared Replay | Added at `.6.1` at `/replay/shared` using a portable replay fragment; `.6.2` preserves return location/page/scroll/focus. It's a separate public-view route from private `Replay`, not private history import. Include the return-to-public-page state. |

## Bounded capture adapter groups

The existing CLI in `gomoku-web/scripts/capture_retrospective_ui.mjs` only
supports `v0.2.4` (`v024`) and `v0.6.2` (`v062`). Keep its manifest's
`sources[]`/`captures[]` provenance shape, add explicit screen/state IDs for
history captures (default state ID `default`), and keep each snapshot tied to
the exact tag's built source and receipt. New adapters should follow this
source grouping rather than branching on dates alone.

1. **Local guest baseline: `v0.2.1–v0.2.4`.** The app routes are common, but
   `.2.1` has no Play From Here action, so its Match image must use the actual
   clean-start screen. From `.2.2` onward a shared completed replay can branch
   to the populated 8-ply Match. Candidate shared content is the existing
   `missed_defense_closed_four` analysis corpus game documented below: nine
   plies, Freestyle, Black win, and Black to move after branching at ply 8.
   Keep the game's player/label differences tag-accurate. The persisted envelope is
   `gomoku2d.guest-profile.v1` through `.2.1`, `.2.2` and `.2.3`; `.2.4` keeps
   the same key. `.3.1` changes that guest schema to `guest-profile.v2`, `.3.2`
   retains it, and `.3.3` moves to `local-profile.v3`. The `.2.1–.2.3` tags lack a tracked
   Cargo lock and `rust-toolchain.toml`, so record generated Cargo resolution
   separately and do not claim the build is lock-reproducible. Use the tag's
   real default opponent label/config: `.2.1–.2.3` say `Classic Bot`, `.2.4–.4.4`
   say `Practice Bot`, and the `.4.5` preset bridge changes the default to
   `Normal Bot`. Preserve that source-visible evolution in screenshots.
2. **Cloud account/history: `v0.3.0–v0.3.3` and `v0.4.0–v0.4.4`.** Guest
   records remain locally seedable but schema changes: `.3.0` uses guest
   profile v1, `.3.1/.3.2` use guest profile v2, and `.3.3` changes to
   `gomoku2d.local-profile.v3` with schema v3. Use sub-adapters for these
   payloads. Add separate Profile states only for evidenced cloud behavior:
   unavailable/signed-out, import progress, synced/retry, Reset confirmation,
   delete-cloud and history expansion. All tags in this group initialize
   Firebase from `VITE_FIREBASE_*` when configured, but none has an
   `FIREBASE_*_EMULATOR` bootstrap hook. The primary-owned loopback transport
   now forwards tagged SDK requests to synthetic demo Auth/Firestore services;
   its `.3.0` signed-in profile flow has been exercised against those services.
   The legacy proxy supports signed-in and sync UI captures for `.3.x–.5.4`
   using synthetic accounts and data; `.5.5+` uses the tagged native-emulator
   hooks. It does not replace application SDK responses. The tag source still has a
   `getFirebaseClients()` bootstrap seam in `src/cloud/firebase.ts` and no call
   to `connectAuthEmulator`/`connectFirestoreEmulator` in these releases.
3. **Static report renderer: `v0.4.0–v0.5.0`.** Keep the tagged standalone
   Bot Report HTML and its data identity; add the Analysis Report beginning
   `.4.2`. The `.4.3` and `.4.4` changes are part of this group and deserve
   capture milestones because their actual tagged HTML changes the visible
   metric columns. Use existing tagged `index.html` and `latest.json` as report
   inputs and the tag's publisher to place them in the preview build. Do not
   run tournaments or regeneration just to make captures. `.5.0` refreshes
   data but has no distinct UI milestone.
4. **Settings and in-app Replay Analysis: `v0.4.5–v0.4.8`.** Profile schema
   jumps to `gomoku2d.local-profile.v5`/version 5 at `.4.5`; Settings and board
   hint defaults need a v5 fixture. Use one valid finished match from the
   existing analysis fixture corpus, wait for the tag's actual Replay
   Analysis to settle, and capture the final status/evidence plus a distinct
   mid-game branch if Match is compared. A good existing candidate is
   `missed_defense_closed_four` in
   `v0.4.8:gomoku-bot-lab/gomoku-eval/src/analysis_fixture.rs`: Freestyle,
   moves `H8 G8 I8 A1 J8 A2 K8 B1 L8`, black win, missed-defense classification,
   critical loser ply 8. Verify that the same input is accepted/renderable by
   each older analyzer before reusing it across `.4.6–.4.8`.
5. **React Lab, Visuals, Rules and Guide: `v0.5.1–v0.5.5`.** Profile remains
   schema v5. React pages replace old generated HTML; static reports become
   `gomoku-bot-lab/{reports,analysis-reports}/report.json` and are consumed by
   tagged React code. `.5.1` still exposes legacy report URLs as redirects to
   Lab tabs; `.5.2` establishes `/lab/` and `/visuals/`; `.5.3` adds SPA policy
   paths. `.5.5` is the first tagged source with explicit
   `VITE_FIREBASE_EMULATORS`, `VITE_FIREBASE_AUTH_EMULATOR_PORT`, and
   `VITE_FIREBASE_FIRESTORE_EMULATOR_PORT` support and a cloud-acceptance
   script. That makes an owned `demo-*` Auth/Firestore emulator the supported
   path for account/sync visuals from `.5.5` forward. Additional states are
   Profile empty/reset, local-persistence notice, Match retry/read-only,
   keyboard focus, Replay possible/confirmed escapes, Lab loading/error, and
   narrow Rules/Guide.
6. **Sharing/public identity: `v0.6.1–v0.6.2`.** Use the `.6.1+` emulator
   hooks plus an owned `demo-*` project with a deterministic synthetic profile,
   public catalog, optional stats snapshot and portable shared replay. Local
   Profile/replay library also needs its own v5/v1 fixtures: `local-profile.v5`
   and `gomoku2d.replay-library.v1` (schema 1). Capture separate Profile
   My Games/Saved Replays/Public Profile states, Public Player default and
   optional stats/catalog states, Shared Replay, and `.6.2` return-to-origin
   plus Guide examples. No real account or production writes.

## Build and evidence limits

- Every tag tracks `gomoku-web/package-lock.json`. `.2.4` declares Node `>=20`;
  `.3.0` onward declares Node `>=24`; `.2.1–.2.3` do not declare a Node engine.
  Capture receipts must record actual runtime versions, not infer them from the
  lockfile.
- `gomoku-web/package.json` depends on `file:../gomoku-bot-lab/gomoku-wasm/pkg`
  throughout. That generated `pkg/` package and Wasm output are not tracked, so
  exact tag source alone is not a buildable web dependency. A tag adapter will
  need tag-local `wasm-pack build --target bundler` before its lockfile-based
  web install/build. Do not reuse the main checkout's `node_modules` across
  old lockfiles.
- Tag build hooks are material: `.2.1–.2.3` are web `tsc && vite build` plus
  404 copy (and still identify as package version `0.1.0`); `.2.4` adds asset
  preview publication; `.3.3` adds explicit SPA route entry generation; `.4.0`
  adds Bot Report publication and `.4.2` adds Analysis Report publication.
  `.5.1` moves those presentation/data contracts into React and JSON; `.5.4+`
  consolidates report copying under `publish_reports.mjs` and uses the Visual
  Design asset publisher. Run each tag's own `npm run build`/postbuild rather
  than substituting current scripts or manually pointing old pages at current
  report data.
- `gomoku-bot-lab/Cargo.lock` and `rust-toolchain.toml` begin at `.2.4`; the
  toolchain file requests the moving `stable` channel with the wasm target, not
  an exact compiler. Record resolved `rustc`, Cargo, `wasm-pack`, Wasm target,
  generated package hashes and `npm` version in each receipt. `.2.1–.2.3`
  need an explicit locally generated Cargo resolution record.
- `gomoku-web/public/assets/gomoku_wasm_bg.wasm` is tracked from `.2.1` through
  `.4.8` and removed in `.5.0`; do not assume this binary replaces building the
  tag-local `gomoku-wasm/pkg` dependency. From `.5.1`, the public report HTML
  and `latest.json` artifacts are replaced by report JSON consumed by React.
- The existing capture contract's desktop 1280×900 DPR1 and provisional generic
  touch 412×800 DPR1 remain the only proposed layouts. Neither is a measured
  OnePlus viewport. Historical source captures are recreated application
  layouts, not proof of the old browser/device runtime.
- No builds, browser captures, emulators, preview servers, or source worktrees
  were created during the source audit. The primary later prepared exact-tag
  builds and the synthetic transport; separate external capture runs produce
  candidate recaptures for media review and history integration. Those images
  do not establish the old browser or device runtime.

## Evidence references

- Release text: `CHANGELOG.md` sections cited in the release table.
- React route identities: per-tag `gomoku-web/src/app/App.tsx` route diffs.
- Static report appearance: per-tag `gomoku-bot-lab/reports/index.html` and
  `analysis-reports/index.html`; metric-label examples were read directly from
  `.4.2`, `.4.3`, and `.4.4` HTML.
- Persistence shapes: per-tag `src/profile/guest_profile_store.ts`,
  `src/profile/local_profile_store.ts`, `src/profile/profile_settings.ts`, and
  `.6.1+` `src/replay/replay_library_store.ts`.
- Emulator support: `src/cloud/firebase.ts` and
  `scripts/run_cloud_acceptance.mjs`; the tagged environment bootstrap appears
  at `.5.5`, not `.3.x`–`.5.4`.
- UI refinement cross-checks: source diffs at `v0.5.3..v0.5.4` and
  `v0.4.8..v0.5.0` plus the documented release text, to separate visible
  states from implementation-only refactors.

## History capture CLI

The external runner is `gomoku-web/scripts/capture_history_ui.mjs`. It accepts
one exact tag and its parsed build receipt per invocation. It does not build,
install, start previews/emulators, or modify application source. Use the
matching `builds/<tag>/receipt.json` and the worktree/base URL from
`previews.json`; embed the parsed receipt at `source.buildReceipt`.
Source and application tree fields are 40-character Git tree OIDs; artifact
fields remain 64-character SHA-256 digests.

From `gomoku-web`, this creates a runnable local-only `.2.4` config from the
primary's build outputs (set `HISTORY_CAPTURE_ROOT` to that owned root):

```sh
rtk node --input-type=module - "$HISTORY_CAPTURE_ROOT" "$HISTORY_CAPTURE_ROOT/v024-capture.json" <<'NODE'
import { readFile, writeFile } from "node:fs/promises";

const [root, configPath] = process.argv.slice(2);
const tag = "v0.2.4";
const preview = JSON.parse(await readFile(`${root}/previews.json`, "utf8"))[tag];
const buildReceipt = JSON.parse(await readFile(`${root}/builds/${tag}/receipt.json`, "utf8"));
const config = {
  schemaVersion: 1,
  source: {
    tag,
    expectedCommit: buildReceipt.sourceCommit,
    worktree: preview.worktree,
    baseUrl: preview.baseUrl,
    adapter: "guest-v1",
    buildReceipt,
  },
  profiles: ["desktop", "portrait"],
  fixture: "missed_defense_closed_four",
  scenes: [
    { screen: "home" },
    { screen: "match" },
    { screen: "profile" },
    { screen: "replay" },
  ],
};
await writeFile(configPath, JSON.stringify(config, null, 2) + "\n", { flag: "wx" });
NODE

rtk npm run capture:history -- --config "$HISTORY_CAPTURE_ROOT/v024-capture.json" --output "$HISTORY_CAPTURE_ROOT/v024-run"
```

`--validate-only` performs source, receipt, preview, and served-byte checks
without creating the output directory. Every capture run needs a new output
directory disjoint from the checkout and selected source worktree. Failed runs
retain `capture.json` and completed PNGs for diagnosis. The manifest records
separate `screen` and `state` IDs.

Supported states are source-gated:

| Screen family | States | Availability |
|---|---|---|
| Home, Match, Profile, Replay | `default` | all 25 tags; Match is a fresh idle board only in `.2.1`, then branches from Replay ply 8 |
| Profile | `empty` | `.5.3` and `.5.4`; uses a fresh v5 local profile with no match rows. `.5.3` leaves the history area blank; `.5.4` shows “No matches yet” and its Play a game action. |
| Profile | `account`, `sync`, `reset` | account `.3.0+`, sync `.3.1+`, reset confirmation `.3.2+` |
| Replay | `share`, `publish` | `.6.1+`; opens the real panel and never submits publication |
| Profile | `saved-replays`, `public-settings` | `.6.1+`; selects the actual Profile tab |
| Shared Replay | `default` | `.6.1+`; validates the seeded portable replay |
| Shared Replay | `from-profile` | `.6.2+`; preserves the public return location |
| Public Player | `default` | `.6.1+`; seeded catalog and shared statistics appear together |

| Settings | `default`, `custom` | `.4.5+` |
| Bot Report | `default` | `.4.0–.5.0`; `.4.0` is static rankings/details only |
| Bot Report | `search`, `pairwise` | `.4.1–.5.0`; `.4.3/.4.4` Search checks the tagged metric headers |
| Analysis Report | `default`, `proof-open` | `.4.2–.5.0` |
| Visuals | `default`, `sprites`, `icons`, `fonts` | `.2.4+`; static pages before `.5.1`, React Visuals tabs from `.5.1` |
| Lab | `default`, `search`, `analysis` | `.5.1+` |
| Rules, Guide | `default`; Guide also `examples` | `.5.2+` |
| Privacy, Terms | `default` | `.3.0+` |

Framed named states retain source layout and record the selected content's
actual viewport rectangle and scroll region. `analysis-report/proof-open`
frames the first proof frame; `settings/custom` keeps the selected Custom card
and first Advanced Controls visible; `guide/examples` opens the tagged Examples
anchor and records both example rows; `.6.2 shared-replay/from-profile` records
the real Return to public profile button in its tagged header. These captures
do not alter source CSS or viewport geometry.

For a Firebase-enabled build receipt, include a matching explicit transport
object: `legacy-proxy` for `.3.x–.5.4` or `native-emulators` for `.5.5+`, with
`ownerRoot` set to the primary-owned demo root. Use a separate
`session: "account"` config for account transitions; public routes use
`session: "guest"`. The runner permits only service URLs approved by the
primary-owned `history_capture_transport.mjs`; it blocks other off-origin,
identity/data, mutation, and WebSocket traffic. The synthetic demo project is
the only allowed Auth/Firestore target. The `.6.2` public fixture must seed all
four public catalog page documents, including valid empty pages, because the
tagged pagination/reconciliation code reads those documents as a four-page
snapshot.

For native-emulator transport only, the harness separately records a Firestore
WebChannel cancellation when an exact owned `GET`/`fetch` to
`/google.firestore.v1.Firestore/Write/channel` or
`/google.firestore.v1.Firestore/Listen/channel` first receives HTTP 200 and
later fails with `net::ERR_ABORTED`. It retains the redacted URL, method,
response status, and timing as expected-cancellation evidence. All other failed
requests remain fatal, including cancellations before a 200 response and any
other origin, path, method, resource type, or failure reason. The optional
`https://apis.google.com/js/api.js` popup bootstrap is also blocked and recorded
for synthetic transport; this does not qualify OAuth popup behavior.
