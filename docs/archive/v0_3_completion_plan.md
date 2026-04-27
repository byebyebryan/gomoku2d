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
- Firestore owner-scoped rules are deployed for `profiles/{uid}` and
  `profiles/{uid}/matches/{matchId}`.

What is still missing before cutting `0.3.0`:

- Test Google sign-in on `https://gomoku2d.byebyebryan.com/` after a tagged
  deploy or release-candidate deploy.
- Confirm the Google Auth Platform Web client allows
  `https://gomoku2d.byebyebryan.com` as a JavaScript origin.
- Decide the OAuth audience gate:
  - keep `Testing` if `0.3.0` is a limited smoke release for configured test
    users
  - publish to `In production` if arbitrary public Google users should be able
    to sign in
- Confirm the released production build behaves correctly with Firebase config
  present.
- Do one no-Firebase-config production build/Profile smoke test so the local
  guest path still shows the unconfigured cloud state cleanly.
- Review Firebase/Auth/Firestore usage dashboards after the first deployed
  cloud-profile smoke test.
- Prepare release mechanics:
  - bump `gomoku-web/package.json` and lockfile to `0.3.0`
  - add a dated `CHANGELOG.md` entry for `0.3.0`
  - run the release checklist in `docs/release.md`

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
