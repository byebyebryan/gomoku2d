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

Normal deploy goes through GitHub Actions OIDC and the dedicated deploy service
account. Manual deploy should be break-glass only.

Local checks:

```sh
cd gomoku-web
npm run test:rules
```

Deploy workflow is tag/manual-gated; do not couple Firestore rules deployment
to every web deploy unless the trust model changes.

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

The real signed-in profile, statistics and publication round trip remains
pending. See the [data contract](../backend/sharing_data.md), the
[v0.6.1 rules-first procedure](release.md#v061-sharing-release-and-compatibility),
and [schema-2 rollout guidance](release.md#rules-first-rollout-for-replay-library-schema-2).

To disable public access operationally, change `publicSharingEnabled()` in
`firestore.rules` from `true` to `false`, validate rules, deploy the reviewed
ruleset and read back its release pointer. Owner cleanup stays available;
portable links and local analysis need no public Firestore access.
