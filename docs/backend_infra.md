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
| Authorized Auth domains | `gomoku2d.firebaseapp.com`, `gomoku2d.web.app`, `localhost`, `dev.byebyebryan.com` |
| Google OAuth client | `projects/gomoku2d/locations/global/oauthClients/gomoku2d-web-auth` |
| Google OAuth client ID | `afb571e3f-1dd4-44d0-902b-f5664aa8f5aa` |
| Firestore database | `(default)` |
| Firestore mode | Native |
| Firestore location | `us-central1` |
| Firestore edition | Standard |
| Firestore free tier | `true` |
| Firestore rules release | `projects/gomoku2d/releases/cloud.firestore` |
| Current ruleset | `projects/gomoku2d/rulesets/4ff3b3d1-4315-49ed-8d81-115ca6ba30dd` |

Important irreversible choice: the default Firestore database is in
`us-central1`. Do not delete/recreate it casually; the database location is a
foundational project decision.

## Enabled APIs

Required for the current `v0.3` backend foundation:

- `firebase.googleapis.com`
- `firestore.googleapis.com`
- `firebaserules.googleapis.com`
- `identitytoolkit.googleapis.com`

Verify:

```sh
gcloud services list \
  --project=gomoku2d \
  --enabled \
  --filter='config.name:(firebase.googleapis.com OR firestore.googleapis.com OR firebaserules.googleapis.com OR identitytoolkit.googleapis.com)' \
  --format='value(config.name)' \
  | sort
```

## Repo Infra Files

| File | Purpose |
|---|---|
| `.firebaserc` | Maps default Firebase project to `gomoku2d` |
| `firebase.json` | Points Firebase tooling at Firestore rules/index files |
| `firestore.rules` | Owner-scoped profile/history security rules |
| `firestore.indexes.json` | Firestore index config, currently empty |
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
- authorized domains include `localhost` and `dev.byebyebryan.com`

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
- `clientId: "afb571e3f-1dd4-44d0-902b-f5664aa8f5aa"`

Verify the backing OAuth client:

```sh
gcloud iam oauth-clients describe gomoku2d-web-auth \
  --project=gomoku2d \
  --location=global \
  --format='yaml(name,clientId,state,allowedRedirectUris,allowedScopes,clientType)'
```

Expected essentials:

- `state: ACTIVE`
- `clientType: CONFIDENTIAL_CLIENT`
- `allowedScopes` includes `openid` and `email`
- `allowedRedirectUris` includes:
  - `https://gomoku2d.firebaseapp.com/__/auth/handler`
  - `https://gomoku2d.web.app/__/auth/handler`
  - `https://dev.byebyebryan.com/gomoku2d/__/auth/handler`
  - `http://localhost:5173/__/auth/handler`

The OAuth client credential secret was shown once at creation time and is not
stored in the repo. Identity Toolkit stores the provider copy. If the secret
needs rotation, create a new credential and patch the provider config with the
new value.

Create the Google provider without using the Firebase console:

```sh
gcloud iam oauth-clients create gomoku2d-web-auth \
  --project=gomoku2d \
  --location=global \
  --client-type=confidential-client \
  --display-name='Gomoku2D Web Auth' \
  --description='Firebase Auth Google provider for Gomoku2D web app' \
  --allowed-grant-types=authorization-code-grant,refresh-token-grant \
  --allowed-scopes=openid,email \
  --allowed-redirect-uris=https://gomoku2d.firebaseapp.com/__/auth/handler,https://gomoku2d.web.app/__/auth/handler,https://dev.byebyebryan.com/gomoku2d/__/auth/handler,http://localhost:5173/__/auth/handler

gcloud iam oauth-clients credentials create gomoku2d-web-auth-secret \
  --project=gomoku2d \
  --location=global \
  --oauth-client=gomoku2d-web-auth \
  --display-name='Gomoku2D Web Auth Secret'
```

Capture the `clientSecret` from the credential creation output, then configure
Identity Toolkit:

```sh
TOKEN=$(gcloud auth print-access-token)
CLIENT_ID='afb571e3f-1dd4-44d0-902b-f5664aa8f5aa'
CLIENT_SECRET='<clientSecret from credential creation output>'

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

If the provider already exists, use the same body with:

```sh
curl -sS -X PATCH \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  -H "Content-Type: application/json" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs/google.com?updateMask=enabled,clientId,clientSecret" \
  -d @-
```

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

## Deploy Firestore Rules

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
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/releases/cloud.firestore?updateMask=rulesetName" \
  -d "{\"rulesetName\":\"${RULESET_NAME}\"}"
```

Verify the live release:

```sh
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://firebaserules.googleapis.com/v1/projects/gomoku2d/releases/cloud.firestore"
```

## Pending Infra Checklist

Before cloud UI ships publicly:

- Test real browser Google sign-in on localhost and the deployed GitHub Pages
  URL.
- Verify that first sign-in creates the expected owner-scoped Firestore
  profile document.
- Confirm the production build initializes Firebase only when config is present.
- Review Firebase/Firestore usage dashboards after the first signed-in test.

Deferred until later phases:

- GitHub sign-in provider, unless Google-only feels too narrow.
- Cloud Run service and runtime service account.
- Firestore indexes beyond the empty starter file.
- Public replay storage and publish/share infra.
