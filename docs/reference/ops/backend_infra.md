# Backend Infra

Purpose: live Firebase/GCP setup and operational checks.

Backend product contract lives in [`backend.md`](../backend/backend.md). Future
Cloud Run design lives in [`future_cloud_run.md`](../backend/future_cloud_run.md).
Cost posture lives in [`backend_cost.md`](backend_cost.md).

## Current Project

| Item | Value |
|---|---|
| GCP/Firebase project | `gomoku2d` |
| Project number | `892554744656` |
| Firebase web app | `Gomoku2D Web` |
| Firebase web app ID | `1:892554744656:web:17524b73c8afb856841255` |
| Auth provider | Google |
| OAuth status | In production |
| Firestore database | `(default)` |
| Firestore mode/location | Native, `us-central1` |
| Firestore edition/free tier | Standard, free tier |
| Rules deployer | `github-firestore-deploy@gomoku2d.iam.gserviceaccount.com` via GitHub OIDC |
| Public policy pages | `/privacy/`, `/terms/` |
| Contact/deletion email | `gomoku2d@byebyebryan.com` |

Do not delete/recreate the default Firestore database casually; database
location is a foundational project decision.

## Repo Infra Files

| File | Purpose |
|---|---|
| `.firebaserc` | Firebase project mapping |
| `firebase.json` | Firebase rules/index config |
| `firestore.rules` | Owner-scoped profile/history plus public-sharing and private-library rules |
| `firestore.indexes.json` | Index config; exempts profile and bounded sharing payload fields |
| `gomoku-web/.env.example` | Public Vite Firebase config template |
| `gomoku-web/src/cloud/firebase.ts` | Optional browser Firebase bootstrap |

The browser bootstrap stays inert until every required `VITE_FIREBASE_*` value
is present.

## Verify Project State

Enabled APIs:

```sh
gcloud services list \
  --project=gomoku2d \
  --enabled \
  --filter='config.name:(firebase.googleapis.com OR firestore.googleapis.com OR firebaserules.googleapis.com OR iam.googleapis.com OR iamcredentials.googleapis.com OR identitytoolkit.googleapis.com OR sts.googleapis.com)' \
  --format='value(config.name)' \
  | sort
```

Firestore:

```sh
gcloud firestore databases describe \
  --project=gomoku2d \
  --database='(default)' \
  --format='yaml(name,locationId,type,databaseEdition,freeTier,deleteProtectionState)'
```

Auth config:

```sh
TOKEN=$(gcloud auth print-access-token)
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/config" \
  | jq '{name, subtype, authorizedDomains}'
```

Google provider:

```sh
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs/google.com" \
  | jq '{name, enabled, clientId}'
```

Expected essentials:

- Firestore `locationId: us-central1`, `type: FIRESTORE_NATIVE`, `freeTier: true`.
- Auth subtype `IDENTITY_PLATFORM`.
- Authorized domains include localhost/dev/public domains.
- Google provider enabled with an `.apps.googleusercontent.com` client ID.

## Firestore Rules Deploy

Normal rules deployment uses GitHub Actions OIDC and the dedicated deploy
service account. The `deploy-firestore.yml` workflow runs on every `v*` tag,
alongside `release.yml` (GitHub Release) and `deploy.yml` (Pages). These tag
workflows run independently. The Firestore job deploys rules from the tag's
source. Tag pushes establish no ordering between rules deployment and the Pages
build, so use the planned guarded dispatch below when rules must go live first.

### Planned Rules-First Deployment

For a compatibility release that needs rules live before the web app, use the
guarded `workflow_dispatch` path on `main` before creating the tag.

1. Freeze the reviewed `main` commit and require exact-SHA CI success.
2. Run this command on `main` with that full commit SHA.

   ```sh
   SOURCE_SHA="REPLACE_WITH_FULL_REVIEWED_COMMIT_SHA"
   gh workflow run deploy-firestore.yml --ref main -f expected_sha="$SOURCE_SHA"
   ```

   The workflow requires `expected_sha` and checks it against both the
   dispatched SHA and checked-out commit before Google Cloud authentication.
   It reads back the created ruleset and compares its source bytes with the
   reviewed `firestore.rules`, records the previous ruleset, then verifies the
   live release pointer. Wait for success and retain the previous/current
   ruleset IDs and source/SHA evidence.
3. If the release includes reviewed changes to `firestore.indexes.json`, apply
   those changes separately through a scoped operation. The Firestore workflow
   deploys rules only. Follow an operation for the specific reviewed fields and
   read each change back, verifying unrelated settings remain unchanged. The
   [v0.6.1 release plan](../../working/v0_6_1_release_plan.md) records the
   single-field exemption operation used there.
4. After the guarded dispatch and any reviewed index operation succeed, create
   and push the release tag. The tag starts the Release, Pages and Firestore
   workflows again. Require all three to succeed and verify the deployed source
   and served artifact under [`release.md`](release.md#cut-release).

Direct Firebase/GCP admin changes to Firestore rules or the active ruleset
pointer outside the guarded workflow are reserved for emergency break-glass
use. Keep the deployed rules source and active release pointer explicit, read
both back after the operation, and retain the previous and current ruleset IDs
for recovery. Coordinate the rules/web compatibility decision with the
[release recovery procedure](release.md#recovery).

Local checks:

```sh
cd gomoku-web
npm run test:rules
```

## Remote Dev Note

For SSH development, prefer port forwarding so the browser origin remains
`localhost`:

```sh
ssh -L 8001:127.0.0.1:8001 bryan@starship.lan
```

## Change Policy

- Update [`data_model.md`](../backend/data_model.md) and rules tests with any
  profile schema change.
- Update [`backend_cost.md`](backend_cost.md) when a backend feature changes
  read/write/storage shape.
- Update [`future_cloud_run.md`](../backend/future_cloud_run.md) rather than
  this runbook for undeployed server-authority designs.

## v0.6 sharing deployment

The v0.6 sharing families are deployed: `public_profiles/{uid}` and its four
catalog pages, plus the owner-scoped `replay_libraries/{uid}`. The v0.6.1
release applied and read back five single-field exemptions for
`public_profiles.display_name`, `public_profiles.shared_stats`,
`public_profiles.catalog_page_counts`, `pages.entries`, and
`replay_libraries.entries_json`. Its release record confirms the four existing
`profiles.match_history` exemptions were preserved. The v0.6.4 release deployed
schema-2-compatible rules before the web tag and made no index changes. The
[v0.6.1 release record](../../archive/v0_6_1_release.md) and
[v0.6.4 release record](../../archive/v0_6_4_release.md) contain release-time
deployment/readback evidence.

A later Chrome check on the published 0.6.5 build verified Google return to an
existing account, publishing and editing, statistics, signed-out readback and
cleanup. First-time Google consent and enabling a new public profile remain
open. See the [data contract](../backend/sharing_data.md), the
[v0.6.1 rules-first procedure](release.md#v061-sharing-release-and-compatibility),
and [schema-2 rollout guidance](release.md#rules-first-rollout-for-replay-library-schema-2).

To disable public access operationally, change `publicSharingEnabled()` in
`firestore.rules` from `true` to `false`, validate rules, deploy the reviewed
ruleset and read back its release pointer. Owner cleanup stays available;
portable links and local analysis need no public Firestore access.
