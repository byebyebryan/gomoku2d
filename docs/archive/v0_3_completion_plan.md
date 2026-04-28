# v0.3 Completion Plan

Status: ad-hoc planning note. `docs/roadmap.md` owns canonical phase intent;
this file tracks the practical rest-of-`0.3` release plan after the roadmap
pivot toward lab-powered product identity in `v0.4`.

## Frame

`v0.3` should stay intentionally narrow: finish **private cloud continuity**
without turning the app into an online/social product yet.

The product reason is straightforward: `v0.4` needs durable saved games as
source material for analysis, puzzles, bot tuning, and "save this game"
challenges. Those saved games should not depend on one browser's local storage.

The process reason is equally important: `v0.3` establishes the cloud
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
- Firestore rules are deployed for owner-scoped `profiles/{uid}` reads/writes,
  with private match writes intentionally closed until cloud history ships.
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

What is still missing before cutting `0.3.0`:

- Review and commit the prepared release diff.
- Finalize/tag `v0.3.0`, push `main` and the tag, then verify the release and
  deploy workflows.

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

Implementation is now started:

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
- Firestore rules allow owner-only `guest_import` match creates and keep
  updates/deletes closed

Work:

- import local guest profile/settings into `profiles/{uid}`
- import finished local matches into `profiles/{uid}/matches/{id}`
- assign stable local-origin IDs so retries are idempotent
- avoid deleting local history until cloud import is confirmed
- make duplicate prevention testable
- update Profile copy so users understand what was promoted and what remains
  local

Done when:

- signing in with existing local history imports that history once
- retrying the import does not duplicate matches
- local guest play still works if import fails or the user signs out

### `0.3.2` — Private Cloud History

Purpose: make new signed-in matches durable across browsers/devices.

Work:

- define the private cloud match record shape for
  `profiles/{uid}/matches/{id}`
- save future signed-in casual matches at match end
- load cloud match history on Profile
- open private cloud-saved replays without introducing public replay URLs
- make local guest history and cloud private history visibly distinct

Done when:

- a signed-in player can finish a match on one browser and review it on another
- local guest history, signed-in private cloud history, and future public
  replay sharing remain separate concepts

### `0.3.x` — Hardening

Purpose: close the backend-foundation line without pulling `v0.4` forward.

Work:

- offline/error-state review for auth, profile loading, promotion, and match
  saves
- Firestore rules validation beyond live smoke testing
- bundle-size check after Firebase imports
- smoke tests for guest-only play and signed-in Profile
- cost/headroom doc refresh after real usage
- final docs sync and release notes

Done when:

- `v0.3` has private cloud continuity, not just cloud setup
- the default local-first product remains intact
- `v0.4` can start from durable saved-game data instead of browser-only state

## Boundary Rules

- Do not add a sign-in wall.
- Do not create public artifacts implicitly.
- Do not blur local guest history with private cloud history.
- Do not introduce Cloud Run unless a chosen `0.3.x` task cannot be solved with
  Firebase Auth + Firestore rules.
- Do not start replay analysis, puzzles, skins, PvP, or sharing inside `0.3`.
