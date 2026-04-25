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
| Auth providers | Google pending OAuth client/provider config |
| Authorized Auth domains | `gomoku2d.firebaseapp.com`, `gomoku2d.web.app`, `localhost`, `dev.byebyebryan.com` |
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

Google sign-in provider is not fully enabled yet. Creating the provider through
the Identity Toolkit API requires an OAuth client ID/secret; the Firebase
console normally provisions that credential when enabling the Google provider.

Current API check:

```sh
curl -sS \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "X-Goog-User-Project: gomoku2d" \
  "https://identitytoolkit.googleapis.com/admin/v2/projects/892554744656/defaultSupportedIdpConfigs/google.com"
```

Expected until provider setup is complete:

- `CONFIGURATION_NOT_FOUND`

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

- Enable Google sign-in in Firebase Auth by provisioning/configuring the Google
  OAuth client.
- Confirm the production build initializes Firebase only when config is present.
- Review Firebase/Firestore usage dashboards after the first signed-in test.

Deferred until later phases:

- GitHub sign-in provider, unless Google-only feels too narrow.
- Cloud Run service and runtime service account.
- Firestore indexes beyond the empty starter file.
- Public replay storage and publish/share infra.
