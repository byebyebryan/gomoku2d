# Backend Infra

Scope: live setup and operational runbook for Gomoku2D's Firebase/GCP backend.
This file records what exists now and how to reproduce or verify it. The target
backend design lives in `backend.md`; free-tier estimates live in
`backend_cost.md`.

## Current Project

| Item | Value |
|---|---|
| GCP/Firebase project | `gomoku2d` |
| Project number | `892554744656` |
| Cloud billing | Enabled |
| Firebase web app | `Gomoku2D Web` |
| Firebase web app ID | `1:892554744656:web:17524b73c8afb856841255` |
| Auth config | Initialized; subtype `IDENTITY_PLATFORM` |
| Auth providers | Google enabled |
| Authorized Auth domains | `gomoku2d.firebaseapp.com`, `gomoku2d.web.app`, `localhost`, `dev.byebyebryan.com`, `gomoku2d.byebyebryan.com` |
| OAuth publishing status | `In production` |
| OAuth scopes observed | `openid`, `userinfo.email`, `profile` |
| OAuth logo | Not set; intentionally blank to avoid brand verification for `v0.3.0` |
| Google OAuth client ID | `892554744656-hksl91isq2pb4pp4dga2h3mi2d02ris2.apps.googleusercontent.com` |
| Public policy pages | `https://gomoku2d.byebyebryan.com/privacy/`, `https://gomoku2d.byebyebryan.com/terms/` |
| Contact/deletion email | `gomoku2d@byebyebryan.com` |
| Firestore database | `(default)` |
| Firestore mode | Native |
| Firestore location | `us-central1` |
| Firestore edition | Standard |
| Firestore free tier | `true` |
| Firestore rules release | `projects/gomoku2d/releases/cloud.firestore` |
| Firestore rules deployer | GitHub Actions OIDC as `github-firestore-deploy@gomoku2d.iam.gserviceaccount.com` |
| Last manual ruleset before CD | `projects/gomoku2d/rulesets/4c4fcf1f-6e05-466c-aeb6-2588241a779f` |
| GitHub Actions default token | Read-only |

Important irreversible choice: the default Firestore database is in
`us-central1`. Do not delete/recreate it casually; the database location is a
foundational project decision.

## Enabled APIs

Required for the current `v0.3` backend foundation:

- `firebase.googleapis.com`
- `firestore.googleapis.com`
- `firebaserules.googleapis.com`
- `identitytoolkit.googleapis.com`
- `iam.googleapis.com`
- `iamcredentials.googleapis.com`
- `sts.googleapis.com`

Verify:

```sh
gcloud services list \
  --project=gomoku2d \
  --enabled \
  --filter='config.name:(firebase.googleapis.com OR firestore.googleapis.com OR firebaserules.googleapis.com OR iam.googleapis.com OR iamcredentials.googleapis.com OR identitytoolkit.googleapis.com OR sts.googleapis.com)' \
  --format='value(config.name)' \
  | sort
```

## Repo Infra Files

| File | Purpose |
|---|---|
| `.firebaserc` | Maps default Firebase project to `gomoku2d` |
| `firebase.json` | Points Firebase tooling at Firestore rules/index files |
| `firestore.rules` | Hardened owner-scoped profile rules and private guest-import match creates |
| `firestore.indexes.json` | Firestore index config; disables indexing for bulky match replay payload fields |
| `gomoku-web/.env.example` | Public Vite Firebase config template |
| `gomoku-web/src/cloud/firebase.ts` | Optional Firebase browser bootstrap |

The browser bootstrap stays inert until every required `VITE_FIREBASE_*` value
is present.

## Verify Firestore

```sh
gcloud firestore databases describe \
  --project=gomoku2d \
  --database='(default)' \
  --format='yaml(name,locationId,type,databaseEdition,freeTier,realtimeUpdatesMode,deleteProtectionState,pointInTimeRecoveryEnablement)'
```

Expected essentials:

- `locationId: us-central1`
- `type: FIRESTORE_NATIVE`
- `databaseEdition: STANDARD`
- `freeTier: true`

## Verify Auth

```sh
TOKEN=$(gcloud auth print-access-token)

curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/config" \
  | jq '{name, subtype, authorizedDomains, client}'
```

Expected essentials:

- `subtype: IDENTITY_PLATFORM`
- authorized domains include `localhost`, `dev.byebyebryan.com`, and
  `gomoku2d.byebyebryan.com`

Verify the Google provider:

```sh
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs/google.com" \
  | jq '{name, enabled, clientId}'
```

Expected essentials:

- `enabled: true`
- `clientId` ends with `.apps.googleusercontent.com`

Important: `gcloud iam oauth-clients create` is not a valid substitute for a
Google Auth Platform Web client here. It creates a Cloud IAM OAuth client with a
UUID-style client ID, which Google Sign-In rejects with `invalid_client`.

If the provider must be recreated, create the correct OAuth client in the
Google Cloud console:

1. Open
   `https://console.cloud.google.com/auth/clients?project=gomoku2d`.
2. If prompted to configure Google Auth Platform branding, use:
   - App name: `Gomoku2D`
   - User support email: `byebyebryan@gmail.com`
   - Audience: external/public
   - Developer contact: `byebyebryan@gmail.com`
3. Create an OAuth client:
   - Application type: `Web application`
   - Name: `Gomoku2D Firebase Web`
4. Add Authorized JavaScript origins:
   - `http://localhost:8001`
   - `http://localhost:3001`
   - `https://gomoku2d.byebyebryan.com`
   - `https://dev.byebyebryan.com`
   - `https://gomoku2d.firebaseapp.com`
   - `https://gomoku2d.web.app`
5. Add Authorized redirect URIs:
   - `https://gomoku2d.firebaseapp.com/__/auth/handler`
   - `https://gomoku2d.web.app/__/auth/handler`
   - `http://localhost:8001/__/auth/handler`
   - `http://localhost:3001/__/auth/handler`
6. Save the generated client ID and client secret outside the repo. The client
   ID should look like `...apps.googleusercontent.com`.

For remote development over SSH, prefer port forwarding so the browser origin
stays `localhost`, for example:

```sh
ssh -L 8001:127.0.0.1:8001 bryan@starship.lan
```

After creating the Web client, configure Identity Toolkit:

```sh
TOKEN=$(gcloud auth print-access-token)
CLIENT_ID='<Google Auth Platform Web client ID>'
CLIENT_SECRET='<Google Auth Platform Web client secret>'

jq -n \
  --arg clientId "${CLIENT_ID}" \
  --arg clientSecret "${CLIENT_SECRET}" \
  '{
    name: "projects/892554744656/defaultSupportedIdpConfigs/google.com",
    enabled: true,
    clientId: $clientId,
    clientSecret: $clientSecret
  }' \
  | curl -sS -X POST \
      -H "Authorization: Bearer ${TOKEN}" \
      -H "X-Goog-User-Project: gomoku2d" \
      -H "Content-Type: application/json" \
      "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs?idpId=google.com" \
      -d @-
```

If the provider already exists, use the same JSON body with:

```sh
curl -sS -X PATCH \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  -H "Content-Type: application/json" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs/google.com?updateMask=enabled,clientId,clientSecret" \
  -d @-
```

### OAuth Audience And Public Access

Google Auth Platform has two separate gates:

- **User type**: `External` means the app can target any Google Account.
- **Publishing status**: `Testing` still limits authorization to explicitly
  listed test users. To let arbitrary Google users sign in, publish the app to
  production.

Before a public Gomoku2D release with Google sign-in:

1. Open `https://console.cloud.google.com/auth/audience?project=gomoku2d`.
2. Confirm user type is `External`.
3. Confirm publishing status. If it is `Testing`, only configured test users
   can authorize, up to Google's test-user limit.
4. Use **Publish app** to move the app to `In production` when public sign-in is
   intended.
5. Review `https://console.cloud.google.com/auth/scopes?project=gomoku2d`.
   Gomoku2D should only request the basic Google Sign-In scopes. Do not add
   Gmail, Drive, Calendar, or other sensitive/restricted scopes without a
   separate verification plan.
6. Review `https://console.cloud.google.com/auth/branding?project=gomoku2d` if
   Google asks for brand verification or if the consent screen needs public
   app identity polish. Verification can require a public homepage, privacy
   policy, and Search Console ownership for authorized domains.

Current `v0.3` state: the OAuth app is `In production`, and runtime sign-in
has been observed requesting only basic identity scopes:

```text
openid https://www.googleapis.com/auth/userinfo.email profile
```

Sensitive-scope verification should not be required as long as the app only
uses Google Sign-In identity scopes. The OAuth logo is intentionally blank for
the `v0.3` backend-continuity line; adding a logo or changing visible branding
details can trigger brand verification.

## Popup Auth Headers

The web app currently uses Firebase Auth's popup flow. In Chrome, the Firebase
SDK may log `Cross-Origin-Opener-Policy policy would block the window.closed
call` while it polls the Google popup. If sign-in completes and the profile
loads, this is popup-flow console noise rather than an Auth failure.

For local development, Vite dev/preview responses set:

- `Cross-Origin-Opener-Policy: same-origin-allow-popups`
- `Referrer-Policy: no-referrer-when-downgrade`

Google Identity Services recommends `same-origin-allow-popups` for popup flows
when FedCM is disabled. GitHub Pages cannot set custom response headers, so the
deployed GitHub Pages build may still show the warning even when sign-in works.
If this becomes unacceptable, the cleaner production fix is to move hosting
behind a platform that can set headers, or switch the app to a redirect-based
flow and follow Firebase's redirect best-practice setup.

## Firebase Web Config

Firebase web config is public configuration, not a secret, but keep local env
files out of git anyway so deploy environments can differ cleanly.

Required Vite env vars:

- `VITE_FIREBASE_API_KEY`
- `VITE_FIREBASE_AUTH_DOMAIN`
- `VITE_FIREBASE_PROJECT_ID`
- `VITE_FIREBASE_STORAGE_BUCKET`
- `VITE_FIREBASE_MESSAGING_SENDER_ID`
- `VITE_FIREBASE_APP_ID`

Fetch registered web apps and config:

```sh
TOKEN=$(gcloud auth print-access-token)

curl -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebase.googleapis.com/v1beta1/projects/gomoku2d/webApps"

APP_ID="1:892554744656:web:17524b73c8afb856841255"
curl -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebase.googleapis.com/v1beta1/projects/gomoku2d/webApps/${APP_ID}/config"
```

The `X-Goog-User-Project` header matters when using local user credentials with
Firebase Management APIs.

## Data Deployment

Normal Firestore rules deployment is CD from GitHub Actions, but it is not tied
to every `main` commit:

- Workflow/job: `.github/workflows/deploy-firestore.yml` / `deploy`
- Automatic trigger: `push` tags matching `v*`
- Manual trigger: `workflow_dispatch` from `main`
- Auth: GitHub OIDC through Workload Identity Federation, no JSON service-account
  key and no long-lived GitHub secret
- OIDC condition: only `byebyebryan/gomoku2d` runs from
  `.github/workflows/deploy-firestore.yml`, and only `v*` tag pushes or manual
  dispatches from `main`, can impersonate the deployer
- GCP service account:
  `github-firestore-deploy@gomoku2d.iam.gserviceaccount.com`
- GCP role: `roles/firebaserules.admin`
- WIF provider:
  `projects/892554744656/locations/global/workloadIdentityPools/github-actions/providers/gomoku2d`
- GitHub repo variables:
  `GCP_WORKLOAD_IDENTITY_PROVIDER`,
  `GCP_FIRESTORE_DEPLOY_SERVICE_ACCOUNT`

The workflow creates a new ruleset from the checked-out `firestore.rules`,
records the previous ruleset in the job summary, patches
`projects/gomoku2d/releases/cloud.firestore` to the new ruleset, and verifies
the live release points at the new ruleset. The live ruleset ID is an
operational artifact, not source-of-truth code; query it when needed instead of
committing a new docs-only ruleset update after every deploy.

Release model:

- `main` remains an integration branch; ordinary pushes run CI but do not publish
  Firestore rules.
- Version tags publish web assets and Firestore rules for the tagged source.
- Manual dispatch from `main` is the controlled escape hatch for transitional
  rules, rollback, or pre-release data migrations.

Live-data rollout rule: treat Firestore rules like an API/schema migration.
When existing clients or data may still use the old shape, use expand/contract:

1. Deploy transitional rules that accept both old and new valid shapes.
2. Deploy the web app that writes the new shape.
3. Wait for old cached/open clients to age out and smoke-test production writes.
4. Tighten rules in a follow-up commit/deploy.

Rollback path: the Firestore release can be patched back to the previous ruleset
ID if permission errors spike. Keep the previous ruleset from the GitHub Actions
summary or from the release history before tightening rules.

## Manual Firestore Rules Deploy

Manual deploys from a local machine are a break-glass path for local
verification or rollback. Prefer the GitHub Actions workflow for normal changes
so production rules are tied to a recorded source ref/SHA.

`firebase-tools` expects its own interactive `firebase login`, even when
`gcloud` is already authenticated. To keep this runbook usable with the current
machine setup, deploy rules through the Firebase Rules REST API.

Create a ruleset from the repo file:

```sh
TOKEN=$(gcloud auth print-access-token)

RULESET_BODY=$(jq -n --rawfile rules firestore.rules \
  '{source:{files:[{name:"firestore.rules", content:$rules}]}}')

RULESET_RESPONSE=$(curl -sS -X POST \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  -H "Content-Type: application/json" \
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/rulesets" \
  -d "${RULESET_BODY}")

RULESET_NAME=$(printf '%s\n' "${RULESET_RESPONSE}" | jq -r '.name')
printf '%s\n' "${RULESET_NAME}"
```

If the `cloud.firestore` release does not exist yet, create it:

```sh
curl -sS -X POST \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  -H "Content-Type: application/json" \
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/releases" \
  -d "{\"name\":\"projects/gomoku2d/releases/cloud.firestore\",\"rulesetName\":\"${RULESET_NAME}\"}"
```

For normal redeploys after the release exists, patch it:

```sh
curl -sS -X PATCH \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  -H "Content-Type: application/json" \
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/releases/cloud.firestore" \
  -d "{\"release\":{\"name\":\"projects/gomoku2d/releases/cloud.firestore\",\"rulesetName\":\"${RULESET_NAME}\"},\"updateMask\":\"rulesetName\"}"
```

Verify the live release:

```sh
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/releases/cloud.firestore"
```

## Deploy Firestore Index Settings

`firestore.indexes.json` is the repo source of truth. For the current local CLI
setup, field exemptions can also be applied directly with `gcloud`.

The `move_cells` replay payload is not queried directly, so indexing is disabled
for that field across the `matches` collection group:

```sh
gcloud firestore indexes fields update move_cells \
  --project=gomoku2d \
  --database='(default)' \
  --collection-group=matches \
  --disable-indexes
```

Verify:

```sh
gcloud firestore indexes fields list \
  --project=gomoku2d \
  --database='(default)' \
  --collection-group=matches \
  --format=json \
  | jq '.[] | select(.name | endswith("/fields/move_cells"))'
```

## Verify Cloud Profile Documents

`gcloud firestore` does not currently expose a `documents list` command in this
local SDK install, so use the Firestore REST API when checking smoke-test data:

```sh
TOKEN=$(gcloud auth print-access-token)

curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firestore.googleapis.com/v1/projects/gomoku2d/databases/(default)/documents/profiles?pageSize=5" \
  | jq '{documents: (.documents // []) | map(.name), error}'
```

Expected after a successful Profile sign-in smoke test: at least one
`profiles/{uid}` document.

## Smoke-Test State

Current cloud UI / data smoke state:

- Localhost Google sign-in has been manually confirmed.
- Production sign-in from `https://gomoku2d.byebyebryan.com/profile` has been
  manually confirmed after publishing the OAuth app to production.
- Runtime OAuth requests from production use only basic identity scopes:
  `openid`, `userinfo.email`, and `profile`.
- The first live `profiles/{uid}` document has been observed in Firestore.
- The no-Firebase-config production build path has been smoke-tested: Profile
  reports cloud sign-in unavailable, the sign-in button is disabled, no
  Auth/Firestore requests are made, and Home/Local Match still load.
- Firebase/Auth/Firestore dashboards have been reviewed after the public smoke
  test and looked normal.
- Local-build guest promotion has been manually smoke-tested for
  `profiles/DbsocAJ0vHVd9LYjk2oaeQx2qec2`: Chrome `localStorage`
  `gomoku2d.guest-profile.v2` had 24 local matches, Firestore had 24 matching
  `guest_import` docs under `profiles/{uid}/matches`, and no local/cloud
  `local_match_id` mismatches were found.
- Local Phase 1 cloud-history sync has been manually smoke-tested: after
  signing out and resetting local profile state, signing back in restored the
  cloud-backed profile/history; a newly finished signed-in match also restored
  after refresh/sign-out/sign-in and could be opened from Replay.

Remaining before the next `v0.3.x` release:

- Deploy the updated Firestore rules for `history_reset_at` and owner-scoped
  private `client_uploaded` match deletes.
- Smoke-test Reset Profile while signed in: cloud history clears, this device's
  local/cache state clears, and older local rows do not re-import afterward.
- Refresh cost/headroom notes after a few more real cloud writes.

Deferred until later phases:

- GitHub sign-in provider, unless Google-only feels too narrow.
- Cloud Run service and runtime service account.
- Firestore indexes beyond the empty starter file.
- Public replay storage and publish/share infra.
