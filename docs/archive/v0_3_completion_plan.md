# v0.3 Completion Plan

Status: completed ad-hoc planning note. `docs/roadmap.md` owns canonical phase
intent; this file preserves the practical rest-of-`0.3` release plan after the
roadmap pivot toward lab-powered product identity in `v0.4`.

## Frame

`v0.3` should stay intentionally narrow: finish **private cloud continuity**
without turning the app into an online/social product yet.

The product reason is straightforward: `v0.4` needs durable saved games as
source material for analysis, puzzles, bot tuning, and "save this game"
challenges. Those saved games should not depend on one browser's local storage.

The production reason is equally important: `v0.3` establishes the cloud
persistence workflow that future AI-agent work can build on safely. If auth,
Firestore writes, docs, cost notes, and release gates are clear here, later
lab-powered and online work can focus on product features instead of guessing
at infrastructure.

## Release Shape

### `0.3.0` — Auth + Cloud Profile

Purpose: prove that optional cloud identity works in the released app while
guest/local play remains complete.

Already in place:

- Firebase config is optional and env-driven.
- CI and tag deploy builds include the public Firebase web config.
- Google provider is configured through Firebase Auth / Identity Toolkit.
- Profile can start auth state, sign in with Google, sign out, and load/create
  `profiles/{uid}`.
- Localhost popup sign-in has been manually confirmed.
- At least one live `profiles/{uid}` document has been observed in Firestore.
- Unit coverage exists for Firebase config gating, auth store behavior, cloud
  profile mapping/writes, cloud profile store behavior, and Profile cloud UI
  states.
- Firestore rules were deployed for owner-scoped `profiles/{uid}` reads/writes,
  with private match writes intentionally closed for the `0.3.0` slice.
- Public app host is `https://gomoku2d.byebyebryan.com/`, served from the
  GitHub Pages custom-domain root.
- Static `/privacy/` and `/terms/` policy pages are deployed and linked from
  Home for users and OAuth crawlers.
- Google Auth Platform is published to production, with the OAuth logo left
  blank intentionally to avoid brand verification for this slice.
- Production Google sign-in has been smoke-tested from the public domain.
- The no-Firebase-config production build path has been smoke-tested: Profile
  shows cloud sign-in as unavailable, makes no Auth/Firestore requests, and
  local Home/Match still work.
- Firebase/Auth/Firestore dashboards have been reviewed after the public smoke
  test and looked normal.

Release prep now in place:

- `gomoku-web/package.json` and lockfile are bumped to `0.3.0`.
- `CHANGELOG.md` has a dated `0.3.0` section and updated compare links.
- `scripts/release.sh --check 0.3.0` passes.
- The full release checklist in `docs/release.md` has been run for the
  prepared `0.3.0` diff.

Release status:

- `v0.3.0` has been cut and published.
- Follow-up workflow fixes split Firestore rules deployment into a tag/manual
  GitHub Actions flow backed by Workload Identity Federation.

Not required for `0.3.0`:

- guest-to-cloud promotion
- saving future signed-in matches to cloud
- loading cloud history or cloud replays
- public replay links
- usernames/public profiles
- GitHub sign-in
- Cloud Run

`0.3.0` is releasable if we are comfortable with the product message:
"sign in creates your private cloud profile; local history remains local until
the next `0.3.x` slice."

### `0.3.1` — Guest-To-Cloud Promotion

Purpose: make sign-in feel like continuity instead of a separate account mode.

Implementation status:

- deterministic private cloud match IDs are based on local match IDs
- local history now has an explicit `guest-profile.v2` saved-match schema, with
  v1 history migrated on read and preserved as a rollback source
- private cloud match v1 uses the same compact `move_cells` replay storage and
  records field-level schema details in `docs/data_model.md`
- player records now carry stable human profile IDs and versioned practice-bot
  identity/config snapshots
- Profile starts promotion in the background after cloud profile load
- default `Guest` local names adopt the cloud name on first sign-in; custom local
  names still override the cloud name during promotion
- Firestore rules allow owner-only `guest_import` and prepared `cloud_saved`
  match creates and keep updates/deletes closed
- local-build smoke verified one 24-match local history promoted into exactly
  24 private `guest_import` Firestore docs with matching `local_match_id`s

Release status:

- `v0.3.1` has been cut and published.
- CI, GitHub Release, GitHub Pages deploy, and Firestore rules deployment all
  passed from the `v0.3.1` tag.

Done:

- signing in with existing local history imports that history once — verified
- retrying the import does not duplicate matches — covered by deterministic IDs
  and unit tests; repeat live smoke still optional
- local guest play still works if import fails or the user signs out — covered
  by current fallback behavior; keep one guest-only smoke in future release
  checks

### `0.3.2` — Private Cloud History

Purpose: make match history feel like one continuous product surface while
making new signed-in matches durable across browsers/devices.

Release status:

- `v0.3.2` has been cut and published.
- CI, GitHub Release, GitHub Pages deploy, and Firestore rules deployment all
  passed from the `v0.3.2` tag.
- production and local-build smoke checks are green:
  signed-in save, refresh/sign-out/sign-in restore, Reset Profile, old-row
  non-reimport, post-reset save, and cross-build cloud sync
- Firestore rules and the matching web build were deployed together because
  private match creates now require `match_saved_at`

UX decision:

- do not make users manage "local history" versus "cloud history"
- signed out: Match History is device-local
- signed in: Match History becomes "my history", backed by private cloud and
  cached locally where useful
- cloud is durable storage; the local active-history cache remains the replay
  source for `0.3.2`
- show sync state only when it matters: pending, failed, offline, or retrying
- keep a single **Reset Profile** danger action, but make its confirmation copy
  state-specific: signed out resets the guest profile on this device; signed in
  resets the cloud profile/history and clears this device's cloud cache
- keep `source`, `trust`, local IDs, cloud IDs, and replay provenance internal
  for dedupe, rules, future sharing, and future trusted/ranked history

Sync model:

- save local history immediately at match end
- enqueue/attempt direct cloud save in the background when signed in
- retry pending saves at natural sync points: sign-in, Profile open, app start,
  auth recovery, and later online recovery if we add explicit network signals
- treat batching as a reliability tool, not a cost tool; Firestore charges by
  read/write/delete operations, so delaying ten match writes still costs ten
  writes
- use sidecar local sync metadata instead of adding sync-only fields to
  `SavedMatchV1`; the canonical replay record should stay valid for local,
  cloud-cache, and future publish flows
- cloud reset writes a reset barrier such as `history_reset_at` before deleting
  history; promotion, direct-sync retry, history load, and the active-history
  resolver must ignore records older than that barrier
- pause or re-check sync attempts while reset is in progress so an in-flight
  match-end save cannot recreate pre-reset history
- avoid write amplification in this slice: do not update profile counters after
  every match unless a later performance problem proves it is needed

Work:

- added a direct `cloud_saved` write path for finished signed-in casual matches
  at `profiles/{uid}/matches/{match.id}`
- prevented duplicate cloud records: guest promotion skips a local match if
  either the deterministic guest-import ID or the raw direct-save ID already
  exists
- added a small pending-sync model for local records that have not reached cloud
  yet
- loaded private cloud match history on Profile, with an initial cap/pagination
  boundary to avoid repeated long-history reads
- cached loaded cloud records locally per signed-in `uid` rather than mixing them
  into the guest/device history bucket
- presented one Match History list in Profile while surfacing pending/failed sync
  only when needed
- defined the active-history resolver: merge pending local rows and per-uid cloud
  cache, dedupe direct `cloud_saved` IDs against `guest_import` `local_match_id`
  records, prefer synced cloud records over local duplicates, sort by
  `saved_at`, and derive Profile stats from this resolved list
- kept replay resolution local-first by reading from the active visible history
  cache; defer a dedicated cloud replay route until public/shareable replay work
- replaced the old local-only reset wording with **Reset Profile** plus inline
  Confirm/Cancel controls
- implemented signed-in reset as reset-barrier write, bounded deletion of owned
  `profiles/{uid}/matches/*`, profile-field reset to defaults, active local
  cache/history clear on this device, per-uid cloud cache clear, and pending-sync
  queue clear
- updated Firestore profile schema/rules for `history_reset_at`, `match_saved_at`
  reset-barrier enforcement, owner-only private match creates/deletes, and the
  signed-in reset write path
- added emulator-backed Firestore rules tests for owner scoping, reset-barrier
  writes, stale match rejection, private match creates, and private match
  deletes
- kept deletes limited to private `client_uploaded` records in this phase; do
  not create rules that could delete future `server_verified` records

Done when:

- a signed-in player can finish a match on one browser and review it on another
- guest-only play still saves and replays locally with no Firebase config
- sync retries do not create duplicate `guest_import` and `cloud_saved` records
- cloud-loaded matches can replay after being cached locally for the signed-in
  user
- reset profile cannot silently delete the wrong scope and cannot re-import
  pre-reset local history afterward
- account switching does not mix per-user cloud caches or pending sync queues
- Firestore rules tests or deployed-rule smoke cover cloud saves, cloud loads,
  reset-barrier writes, and private match deletes
- the UI reads as one history surface, while docs/tests still keep local guest
  history, private cloud history, and future public replay sharing distinct

Smoke checklist:

- signed-out/no-Firebase build still records and replays local history
- signed-in match saves locally first, then reaches Firestore as `cloud_saved`
- a second browser/device loads cloud history, caches it locally, and replays it
- Reset Profile while signed in clears cloud history and this device's active
  cache without re-importing older local records
- failed/offline sync remains visible but does not block play

### `0.3.3` — Backend Continuity Wrap-Up

Purpose: stop expanding `v0.3` and harden the cloud-continuity layer enough
that `v0.4` can build on it.

This slice replaced several intermediate `0.3.1`/`0.3.2` shapes rather than
preserving them as long-lived migrations. The alpha data set was still test
data, so the project accepted clean breaks where they made the backend model
simpler.

Work completed after `v0.3.2`:

- moved casual private history out of `profiles/{uid}/matches/*` and into the
  owner profile's embedded `match_history` snapshot
- introduced profile schema v3 with `auth.providers`, `settings.default_rules`,
  `reset_at`, `replay_matches`, `summary_matches`, and `archived_stats`
- aligned local history to the clean-break `gomoku2d.local-profile.v3` key and
  intentionally ignored older `guest-profile.*` keys
- capped full replay retention at 128 records, kept the next 1024 summaries,
  and rolled older records into archive counters
- coalesced signed-in profile/settings/history writes into a 5-minute profile
  snapshot lane
- hardened Firestore rules for schema v3, history caps, profile update
  cooldowns, reset-barrier writes, reset cooldown bypasses, and closed casual
  match subcollection writes
- added a signed-in Delete Cloud path behind Reset Profile for owner-only cloud
  profile deletion and sign-out without clearing local browser history
- reconciled queued cloud-history sync after live/local races so stale local
  errors clear when Firestore already contains the match
- added Firebase Auth redirect fallback for mobile, embedded, blocked-popup,
  and unsupported-popup environments
- added direct GitHub Pages route entries for `/profile` and `/match/local`
- polished Profile and policy copy around local profile, cloud profile,
  experimental online features, sync state, and reset scope
- updated release, infra, data-model, and cost docs to match the final `v0.3`
  backend posture

Release status:

- `v0.3.3` is the intended wrap-up release for the backend-continuity line.
- Final pre-tag work is limited to release mechanics: bump package version,
  run release checks, deploy the release candidate if needed, smoke production,
  then tag.

### Later `0.3.x` — Narrow Hardening Only

Purpose: only handle backend-foundation hardening if real usage exposes it.
This is not an active product scope.

Work:

- watch Firebase/Auth/Firestore usage after a little more real traffic and
  refresh cost notes if the dashboard shows anything surprising
- carry any remaining auth/offline polish into later `0.3.x` only if real usage
  exposes it; do not invent more backend scope before `v0.4`

Done when:

- `v0.3` has private cloud continuity, not just cloud setup
- the default local-first product remains intact
- `v0.4` can start from durable saved-game data instead of browser-only state

## Boundary Rules

- Do not add a sign-in wall.
- Do not create public artifacts implicitly.
- Do not blur internal provenance for local guest history, private cloud
  history, or future public replay sharing; the UX can abstract storage details
  when sync is healthy.
- Do not introduce Cloud Run unless a chosen `0.3.x` task cannot be solved with
  Firebase Auth + Firestore rules.
- Do not start replay analysis, puzzles, skins, PvP, or sharing inside `0.3`.
