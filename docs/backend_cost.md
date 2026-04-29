# Backend Cost And Free-Tier Headroom

Scope: track the cost shape of backend pieces as they are added. The goal is
to keep Gomoku2D comfortably inside free tiers while the project is small, and
to notice early when a feature changes the cost profile.

Live setup state lives in `backend_infra.md`; backend architecture and trust
lanes live in `backend.md`.

This is an estimate document, not billing truth. Re-check official pricing
before each backend release and use the Firebase/GCP usage dashboards after
deploying. Pricing assumptions below were last re-checked against Firebase's
official pricing and Firestore billing docs on 2026-04-29.

Sources:

- [Cloud Firestore billing](https://firebase.google.com/docs/firestore/pricing)
- [Cloud Firestore quotas](https://docs.cloud.google.com/firestore/quotas)
- [Firebase pricing plans](https://firebase.google.com/docs/projects/billing/firebase-pricing-plans)
- [Firebase Authentication](https://firebase.google.com/docs/auth/)
- [Cloud Run pricing](https://cloud.google.com/run/pricing)
- [Google Cloud free tier](https://cloud.google.com/free/docs/gcp-free-tier)

## Current Backend Footprint

Project: `gomoku2d`

| Piece | Current state | Cost posture |
|---|---|---|
| Firebase project | Linked to GCP project `gomoku2d` | No direct cost |
| Firebase web app | `Gomoku2D Web` | No direct cost |
| Firebase Auth | Initialized as Identity Platform; Google provider enabled | Billing is enabled; stay inside no-cost MAU tier and avoid phone/SAML/OIDC |
| Firestore | Default Native-mode database in `us-central1`; `freeTier: true` | Uses the one free Firestore database for the project |
| Firestore rules | Repo rules deployed to `cloud.firestore` release | No runtime cost |
| Cloud Run | Not created in `v0.3` | Future online/trusted-match phase, or earlier only if lab-powered analysis needs server compute; billing is enabled, but no service exists yet |

## Firestore Free Tier

Firestore free quota resets daily around midnight Pacific time. A project gets
exactly one free database.

| Resource | Free quota |
|---|---:|
| Stored data | 1 GiB |
| Document reads | 50,000 / day |
| Document writes | 20,000 / day |
| Document deletes | 20,000 / day |
| Outbound data transfer | 10 GiB / month |

Features to avoid while we are intentionally staying free-tier-first:

- TTL deletes
- point-in-time recovery
- backups/restores/clones
- extra Firestore databases

## Expected `v0.3` Firestore Usage

The `v0.3` cloud path is private profile/history continuity, not live gameplay.
Local guest play has no backend cost.

Counting assumptions:

- Firestore bills document reads, writes, and deletes separately.
- A query has a one-read minimum even when it returns no documents.
- Private casual history is embedded in `profiles/{uid}.recent_matches`; there
  is no per-match document create in the current casual cloud path.
- Replay currently resolves from the active Profile/history cache; there is no
  dedicated cloud replay fetch route in `v0.3.3`.

Estimated operation budgets for the current implementation:

| Action | Expected reads | Expected writes | Expected deletes | Notes |
|---|---:|---:|---:|---|
| Guest local play / local match finish | 0 | 0 | 0 | LocalStorage only |
| No-config build / signed-out Profile | 0 | 0 | 0 | Firebase bootstrap stays disabled |
| First sign-in / cloud profile create | 1 | 1 | 0 | `getDoc(profiles/{uid})`, then create profile |
| Cloud profile/history load | 1 | 0-1 | 0 | `getDoc(profiles/{uid})`; existing profiles only write when provider metadata or schema needs refresh |
| Coalesced profile/history sync | 0 | 0-1 | 0 | One profile snapshot write at most every 15 minutes when local profile/history is dirty |
| Open Replay from loaded history | 0 | 0 | 0 | Uses active local/cloud history cache |
| Reset signed-in profile | 1-2 | 1 | 0 | Profile reset write clears `recent_matches`; optional refresh read returns server timestamps |

Initial implementation guardrails:

- Limit embedded recent history to the existing local cap of 24 records.
- Avoid always-on listeners for full match history unless the UI actually needs
  live updates.
- Prefer one coalesced profile snapshot write over per-match cloud documents for
  casual play.
- Keep routine signed-in profile refreshes read-only; write profile updates only
  when a user-visible setting or provider-owned field changed.
- Defer signed-in profile/settings sync to sign-in, retry, and match-finish
  checkpoints so rapid name typing or default-rule toggles stay local until a
  meaningful cloud sync point.
- Enforce a 15-minute server-side cooldown between normal profile snapshot
  updates in Firestore rules. Reset-barrier writes can bypass the normal edit
  cooldown, but are still scoped to real reset writes.
- Keep public replay publishing out of `v0.3`; private history and public
  shareables have different cost and trust profiles.
- Budget cloud history as profile writes, not match creates. The free-tier write
  knob is now the sync interval: `1440 / interval_minutes` writes per
  continuously dirty active user per day.

## `v0.3.3` Cost Formulas

These formulas are intentionally conservative enough for release planning.

Let:

- `P = profile opens per signed-in user per day`.
- `S = profile/history snapshot syncs per signed-in user per day`.
- `I = sync interval in minutes`; currently `15`.

Approximate daily operations per signed-in user:

| Flow | Reads | Writes | Deletes |
|---|---:|---:|---:|
| Signed-in Profile opens | `P` | `0` normally | `0` |
| Dirty profile/history syncs | `0` | `S`, capped by `1440 / I` | `0` |
| Reset Profile | `1..2` | `1` | `0` |

With `I = 15`, a continuously dirty signed-in user can write at most 96 profile
snapshots per day. Real users should be far lower because idle users do not
write, and multiple matches/profile edits inside a 15-minute window coalesce
into the next snapshot.

## Headroom Scenarios

These are deliberately rough. They are meant to catch order-of-magnitude
mistakes before implementation.

### Small Public Release

Assumption: 100 signed-in users/day, each opens Profile twice and has one dirty
cloud snapshot sync.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~200/day | ~250x below 50,000/day |
| Writes | ~100/day | ~200x below 20,000/day |
| Deletes | 0/day | No concern |

This is safely inside the free tier.

### Profile-Heavy Usage

Assumption: 500 signed-in users/day, each opens Profile 5 times.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~2,500/day | ~20x below 50,000/day |
| Writes | ~0/day before match saves | No concern |

Embedding capped history in the profile removes the old history-query read
amplification. If future history grows beyond the embedded cap, revisit
pagination before adding more history-heavy surfaces.

### Continuously Dirty Users

Worst case: each active signed-in user keeps changing local profile/history
state continuously all day.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| 5-minute interval | 288 writes/user/day | ~69 continuously dirty users |
| 10-minute interval | 144 writes/user/day | ~138 continuously dirty users |
| 15-minute interval | 96 writes/user/day | ~208 continuously dirty users |
| 30-minute interval | 48 writes/user/day | ~416 continuously dirty users |
| 60-minute interval | 24 writes/user/day | ~833 continuously dirty users |

The current 15-minute interval is a conservative alpha default. It keeps a
simple cost knob without introducing per-match sync UX.

### One-Time Promotion Spike

Assumption: 100 existing local players sign in for the first time with 24 local
matches each.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~100 one-time reads | ~500x below 50,000/day |
| Writes | ~200 one-time writes | ~100x below 20,000/day |
| Deletes | 0 | No concern |

This is fine for current scale because promotion writes one capped profile
snapshot rather than one document per match.

### Reset-Heavy Debug Day

Assumption: 20 signed-in test profiles each reset a full embedded private
history during QA.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~20..40/day | No concern |
| Writes | ~20/day | No concern |
| Deletes | 0/day | No concern |

Reset is not a cost risk at current scale. The bigger risk is correctness:
reset-barrier writes and stale-match rejection must stay covered by rules tests.

### Replay Sharing / Online Future

Public replay pages, online match subscriptions, and trusted server-written
match state can change the read/write profile quickly. Before the online
product-expansion phase, add a separate estimate for:

- active online match listeners
- per-move server writes
- public replay page reads
- profile/ranked leaderboard queries

## Auth Cost Notes

The project is currently initialized with an Identity Platform Auth config and
has Cloud Billing enabled. This is still safe for the expected scale, but cost
control now depends on staying inside the no-cost tier and monitoring usage,
not on Spark-plan shutoff behavior.

Current no-cost planning assumptions:

- Email, social, anonymous, and custom auth have a no-cost tier of 50,000 MAUs
  on Blaze / Identity Platform.
- SAML/OIDC has a much smaller no-cost tier and is out of scope.

Avoid:

- phone auth
- SAML/OIDC providers

Re-check Auth pricing before adding any provider beyond Google/GitHub social
sign-in.

## Cloud Run Future Budget

Cloud Run is intentionally out of `v0.3`. When it arrives, use
request-based billing in `us-central1` and keep minimum instances at zero.

Current free tier to plan against:

| Resource | Free quota |
|---|---:|
| Requests | 2,000,000 / month |
| CPU | 180,000 vCPU-seconds / month |
| Memory | 360,000 GiB-seconds / month |
| North America outbound transfer | 1 GiB / month |

Guardrails for future Cloud Run work:

- no always-on minimum instances unless we intentionally accept cost
- short request timeouts for bot/analysis endpoints
- no per-move Cloud Run calls for casual/local play
- use Cloud Run only for trust boundaries or work the browser cannot safely do

## Cost Review Checklist

Before each backend release:

- Check Firebase Console usage for Firestore reads/writes/storage.
- Check GCP billing dashboard if billing is enabled.
- Re-read official pricing pages if adding a new backend product.
- Add a row to this doc for the new backend feature.
- Decide whether the feature needs a read/write cap, pagination, cache, or
  manual rate limit.

`v0.3.0` note: Firebase/Auth/Firestore dashboards were reviewed after the
public sign-in smoke test and looked normal for the first cloud-profile slice.

`v0.3.1` note: the first local-build guest promotion smoke imported 24 private
`guest_import` match documents for one signed-in profile. This was well inside
the free tier, but it exposed the per-match-write cost model that `v0.3.3`
later replaced.

`v0.3.2` note: production and local-build smoke covered signed-in
`cloud_saved` writes, cloud-history reload, Reset Profile deletion/barrier
behavior, and post-reset saves. At that point, one finished signed-in match
wrote one private match document after a small number of dedupe/profile reads,
and Reset Profile paid one profile write plus one delete per private
`client_uploaded` match cleared. That per-match document model was replaced in
`v0.3.3`.

`v0.3.3` note: routine existing-profile loads are read-only when cloud profile
fields are already current. Casual private history moved from one match document
per save to an embedded, capped `recent_matches` profile snapshot. Profile,
settings, and history updates now share one coalesced profile write lane with a
15-minute rules-enforced cooldown; Reset-barrier writes can bypass the normal
cooldown so Reset Profile is not blocked by a recent profile sync.
