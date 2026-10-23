# v0.6.1 release plan

Owner: Bryan. `v0.6.1` was published on October 23, 2026; see the
[release record](../archive/v0_6_1_release.md) for execution receipts and
remaining acceptance. Exact-source CI, backend rollout, tag workflows,
served-artifact parity and the production-safe browser smoke are complete. The
real signed-in sharing round trip remains pending; this plan records the
executed sequence and its remaining gate.
This plan covers a direct `v0.5.5` to `v0.6.1` publication. It does not require an
intermediate `v0.6.0` release. The maintained
[release workflow](../reference/ops/release.md) owns the general process;
this document fixes the scope, ordering and acceptance for this release.

## Scope and current evidence

Release theme: **share games and public player profiles**.

- Include self-contained replay links, explicit saved replay copies, ordinary
  continuation games, opt-in public profiles, dated opt-in statistics and paged
  publications, accepted sharing/profile controls and icons, and the
  Canvas/WebGL colour fix.
- Multiplayer, avatars/bios, richer statistics, game thumbnails and further
  presentation work remain deferred. Gameplay and analysis stay in the browser.
- Do not add infrastructure, billing-plan changes, automatic statistics writes,
  polling or per-move cloud traffic to ship this release.

The [final integrated review](v0_6_1_evidence/wrap_review.json) found no
remaining functional/privacy/data-integrity defect: 493 web units, 49 rules/SDK
checks, 219 ordinary browser passes with six intentional WebGL skips, 42 cloud
journeys, 45 three-worker race repetitions, 18 clean accessibility audits,
build/types, 12 direct routes and the production npm audit passed. Exact-source
CI also passed with 348 Rust tests and those integrated web, rules, browser,
cloud and race results. After deployment, all 89 selected served files matched
the Pages artifact and the production smoke passed 96 checks with six
intentional WebGL skips and no failures. The
[release record](../archive/v0_6_1_release.md) links exact workflow identities.

Read-only baseline immediately before version/documentation preparation,
October 23, 2026:

| Item | Observed state |
|---|---|
| Local `main` | `95eb5ab`, clean before preparation, 40 commits ahead of the verified remote head |
| Remote `main` | `0995af835a5e81377f1270b19d0cc780dce44f36` |
| Package/changelog | `0.5.5`, new work under Unreleased before this preparation |
| Latest published release | `v0.5.5`; no remote `v0.6*` tags |
| Successful Pages deployment | `7142877612`, source `73658d0a546ba558c746854d38cdac7ec2465802` / `v0.5.5` |
| Live Firestore ruleset | `projects/gomoku2d/rulesets/bb6ecb9e-0c7d-4e31-b601-82f95799b3b6` |
| Live rules source | Exactly matches `v0.5.5:firestore.rules`, differs from the candidate |
| Existing private-history index configurations | Four reviewed fields have no active indexes |
| Five new sharing field exemptions | Not applied; fields still inherit the standard indexes |
| OAuth | Live Firebase recorded Google sign-ins at 11:44:36 and 11:47:19 Pacific; Bryan reports renaming the client removed its inactivity warning |

Pre-release public HTML SHA-256:
`f5c4a0b2d8edb932e61b45a08ecc6408f616cd6b151268a2a3b2cb52e99f8e5e`.
Pre-release entry script `/assets/index-DVVTG5oM.js` SHA-256:
`e9c8c0e5a559e8e0328516395ea8aa6d5bbf248bff751e6be80f29d1b8ae39dd`.
This table is a pre-preparation baseline; it is not current deployed state.
The frozen source `b1a67d32feee5b57275e3d04c684e27bcd1a571f` is tagged and
published. CI, GitHub Release, Firestore rules, and Pages workflows succeeded.
The five planned field exemptions were applied and read back, with the four
private-history settings unchanged. Served artifact parity and automated
public-origin checks passed; the real signed-in sharing round trip remains
pending. See the linked release record for exact workflow IDs, ruleset evidence,
and retained manual deferrals.

## 1. Prepare one release commit — complete

1. Recheck clean branch, remote head and absent `v0.6.1` tag. Preserve any new
   concurrent work; use an isolated checkout if it prevents a clean release.
2. Run `scripts/set-web-version.sh 0.6.1` for the web package and lockfile.
3. Move all current Unreleased notes into a dated `0.6.1` section with the theme
   above. Leave Unreleased empty. Compare from `v0.5.5` to `v0.6.1`, then from
   `v0.6.1` to HEAD; include both the original sharing work and the 0.6.1 pass.
4. Update README Project Status, changelog/evidence links, web feature inventory
   and roadmap release wording. Default to retaining the accepted hero/OG/media,
   including their honest 0.5.5 capture caption. Do not regenerate unchanged bot
   reports; verify committed report provenance and copied build artifacts.
5. Refresh required production npm and Rust advisory audits. Retain the known
   development-only Firebase CLI findings separately; investigate any new
   release-relevant finding before proceeding. Avoid unrelated dependency work.
6. Build with the production Firebase configuration and `GOMOKU_BASE_PATH=/`.
   Check the visible 0.6.1 version, static/public route entries, reports, sprites
   and a focused replay/profile/Canvas smoke. Use an owned free test port such
   as 8014; preserve the manual preview/emulators and unrelated port 8001.
7. Review and commit only preparation files. Record the resulting source SHA.

**Checkpoint passed:** release commit `b1a67d32feee5b57275e3d04c684e27bcd1a571f`
has the `0.6.1` version and dated changelog. The production-configured build,
advisory checks and focused smoke passed; existing integrated test evidence
remains attached. Any future runtime/dependency fix reopens its affected gates.

## 2. Push main and qualify the exact commit — complete

Push the reviewed preparation commit directly to `main`, following the
repository's existing workflow. This push starts CI; the deployment workflows
are tag/manual triggered.

Require CI success with `headSha` exactly equal to the preparation SHA:

- Rust formatting, Clippy and workspace tests;
- fresh Wasm generation and locked npm install;
- web types, units and rules/SDK scenarios;
- production-configured build and Chromium/Firefox/WebKit journeys;
- full emulator cloud lane and configured conflict/account/reset repetitions.

The final local three-worker 45-case race result supplements CI's configured
lane. Preserve the six intentional Firefox/WebKit WebGL skips explicitly.
Any fix requires a new preparation commit and CI success on that new SHA.
Freeze the selected `main` SHA through backend preparation and tagging.

**Checkpoint passed:** exact-source CI run
[40027552184](https://github.com/byebyebryan/gomoku2d/actions/runs/40027552184)
and `scripts/release.sh --check 0.6.1` succeeded before tagging. The original
run `40026924846` caught a timezone-specific unit expectation; its fixture date
assertion was corrected and passed in both UTC and America/Los_Angeles before
the final CI run.

## 3. Prepare the backend before publishing the web app — complete

1. Capture the current Pages deployment/tag/SHA and served hashes, Firestore
   release pointer and source, and the nine reviewed field configurations.
2. Dispatch `.github/workflows/deploy-firestore.yml` on the frozen reviewed
   `main`. Wait for success, verify the workflow SHA, read the live release
   pointer and compare the fetched rules source byte-for-byte with the selected
   commit. Keep both the original v0.5.5 ruleset and this new ruleset ID.
3. Apply only these five new exemptions in `gomoku2d`, database `(default)`:

   | Collection group | Field |
   |---|---|
   | `public_profiles` | `display_name` |
   | `public_profiles` | `shared_stats` |
   | `public_profiles` | `catalog_page_counts` |
   | `pages` | `entries` |
   | `replay_libraries` | `entries_json` |

   Use the documented `gcloud firestore indexes fields update FIELD
   --project=gomoku2d --database='(default)' --collection-group=GROUP
   --disable-indexes` operation, one field at a time. Wait for operations and
   read each setting back. Verify the four existing history exemptions remain.
   Preserve unrelated indexes and field settings; the rules workflow does not
   deploy indexes. These exemptions reduce unneeded payload indexing and are
   not composite indexes required by a query.
4. Check that the existing v0.5.5 site still boots and the live rules source is
   the reviewed additive version. A source/pointer comparison does not replace
   the end-user sharing acceptance in step 5.

**Checkpoint passed:** pre-tag rules workflow
[40029725502](https://github.com/byebyebryan/gomoku2d/actions/runs/40029725502)
succeeded. Ruleset `dccb94b6-7dc7-41a5-bdf2-d9c807fb07f9` matched the reviewed
source hash; all five exemptions were read back and the four private-history
settings stayed unchanged. No database migration, user-data reset or new
service was needed.

## 4. Publish v0.6.1 — complete

1. Reconfirm the chosen `main` SHA, exact-commit CI success, clean tree and live
   backend checkpoint. Run `scripts/release.sh --check 0.6.1`.
2. Run `scripts/release.sh 0.6.1`; inspect the annotated tag and its peeled SHA.
3. Push only `v0.6.1`. This starts GitHub Release, Pages and Firestore rules
   workflows independently. They are not ordered dependencies; step 3 above
   establishes rules-first deployment before this fan-out.
4. Require all three workflows to succeed at the intended tag/source SHA.
   The tag-triggered rules workflow creates another ruleset with the same
   source. Verify its final live pointer/content and retain both pre-tag and
   post-tag ruleset IDs rather than expecting their IDs to match.
5. Verify release title/body/version, deployed Pages source identity and served
   artifact bytes, including entry JS/CSS/Wasm and the direct public routes.

**Checkpoint passed:** tag `v0.6.1` peels to the frozen source SHA. The GitHub
Release [40031565001](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565001),
Firestore rules [40031565082](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565082),
and Pages [40031565071](https://github.com/byebyebryan/gomoku2d/actions/runs/40031565071)
workflows succeeded. Pages deployment `7289818516` serves the selected release
artifact; the post-tag ruleset is recorded in the release receipt.

## 5. Accept the actual public release and close the receipt

The production-safe smoke passed 96 checks with six intentional WebGL skips
and no failures in fresh headless contexts across Chromium, Firefox and WebKit.
It covered visible version, public routes, reports/assets, local bot play,
finished-game replay, portable links, continuations, Canvas colours and
qualified WebGL. All 89 selected served files, including 12 direct/static
routes, matched the Pages deployment artifact. The destructive emulator cloud
suite was not pointed at production.

One real signed-in round trip remains pending, using a designated disposable
Google test account and synthetic game. It has not been counted as passed or
explicitly deferred. When a test account and signed-in browser are available,
Bryan performs these UI actions while the agent handles guest checks and
receipts:

1. Sign in on the deployed site, complete a short local game on the test
   account, and open the public-profile tab.
2. Enable a test public name, explicitly share a complete stats snapshot, and
   publish one synthetic finished game.
3. Open the public profile and replay as a signed-out guest; confirm stats and
   replay attribution. Explicit save and continuation affect only that fresh
   guest context until the user chooses otherwise.
4. Unpublish the test game, turn stats sharing off and disable the test public
   profile. Verify the guest view is unavailable while a previously copied
   portable replay still opens. Leave the required revision/generation markers.

Do not use Bryan's real account/history for Reset or Delete Cloud acceptance.
Their destructive lifecycle and account races retain the existing emulator
evidence. Today's live Google login and cleared warning establish a different
scope from this new publication/view/cleanup round trip. If the real round trip
is not performed, record publication as deployed with that acceptance pending;
do not silently call it a passed check.

The [release receipt](../archive/v0_6_1_release.md) records preparation/tag
SHAs, exact CI and deployment run IDs, Pages deployment/artifact and served
hashes, previous/current ruleset IDs/source hashes, before/after index
settings, browser smoke and the pending signed-in sharing gate. Raw material is
under ignored `gomoku-bot-lab/outputs/release-prep/0.6.1/`. Any receipt-only
follow-up commit leaves the release tag fixed.

Carry forward physical-device/touch performance, human screen-reader/zoom,
two-device private account/reset and novice-feedback deferrals unless new
evidence actually closes them. Clean up only owned temporary tests/tabs and
services; preserve the manual preview until Bryan is finished with it.

## Hold and recovery rules

- Hold before the next stage on a failed audit/check, unexpected source or
  remote change, missing index read-back, workflow failure or production
  correctness/privacy issue. Do not fix an unrelated issue by broadening the
  deployment or resetting production data.
- Before any new sharing data is written, the captured prior ruleset pointer
  remains available for a scoped backend rollback if needed. Verify that
  precondition; do not infer it from the older web app still being visible.
- After sharing data exists, prefer a compatible forward fix or disable public
  sharing with the reviewed `publicSharingEnabled()` switch while retaining
  owner cleanup and private-library access. Verify no pending rules deployment
  can overwrite the chosen recovery state, then deploy and read it back.
- For a web regression, redeploy the captured previously accepted Pages ref
  through the workflow and verify its source/served bytes. A v0.5.5 web rollback
  retains the additive sharing rules: older tabs cannot manage new sharing
  families and their Delete Cloud can be denied until the current client is
  available. Do not restore v0.5 rules after sharing writes or bypass the guard.
- Do not move release tags, delete lifecycle markers, clear user documents or
  undo the bounded index exemptions as an automatic rollback action.

References: [single-field index update](https://docs.cloud.google.com/sdk/gcloud/reference/firestore/indexes/fields/update),
[rules release pointer](https://firebase.google.com/docs/reference/rules/rest/v1/projects.releases/patch),
[existing acceptance deferrals](../archive/reconciliation_2026_10/README.md#deferred-acceptance).
