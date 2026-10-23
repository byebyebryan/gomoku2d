# Backend Cost And Free-Tier Headroom

Scope: track the cost shape of backend pieces as they are added. The goal is
to keep Gomoku2D comfortably inside free tiers while the project is small, and
to notice early when a feature changes the cost profile.

Live setup state lives in [`backend_infra.md`](backend_infra.md); backend
architecture and trust lanes live in [`backend.md`](../backend/backend.md).

This is an estimate document, not billing truth. Re-check official pricing
before each backend release and use the Firebase/GCP usage dashboards after
deploying. Pricing assumptions below were last re-checked against Firebase's
official pricing and Firestore billing docs on 2026-10-15.

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
| Cloud Run | Not created | Future online/trusted-match phase, or earlier only if lab-powered analysis needs server compute; billing is enabled, but no service exists yet |

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

## Expected Firestore Usage

The current cloud path is private profile/history continuity, not live gameplay.
Local play has no backend cost.

Counting assumptions:

- Firestore bills document reads, writes, and deletes separately.
- A query has a one-read minimum even when it returns no documents.
- Private casual history is embedded in `profiles/{uid}.match_history`, with
  full replay, summary, and archived-stats tiers in the same profile document;
  there is no per-match document create in the current casual cloud path.
- Replay currently resolves from the active Profile/history cache; there is no
  dedicated cloud replay fetch route.

Estimated operation budgets for the current implementation:

| Action | Expected reads | Expected writes | Expected deletes | Notes |
|---|---:|---:|---:|---|
| Guest play / local finish | 0 | 0 | 0 | LocalStorage only |
| No-config build / signed-out Profile | 0 | 0 | 0 | Firebase bootstrap stays disabled |
| First sign-in / cloud profile create | 1 | 1 | 0 | `getDoc(profiles/{uid})`, then create profile |
| Cloud profile/history load | 1 | 0-1 | 0 | `getDoc(profiles/{uid})`; existing profiles only write when provider metadata or schema needs refresh |
| Coalesced profile/history sync | 1+ when writing | 0-1 | 0 | Transaction reads the current snapshot before replacement; history retry may add a preflight read and contention may retry |
| Open Replay from loaded history | 0 | 0 | 0 | Uses active local/cloud history cache |
| Reset signed-in profile | 1-2 | 1 | 0 | Profile reset write clears `match_history`; optional refresh read returns server timestamps |

Initial implementation guardrails:

- Limit embedded full replay history to 128 records and the lightweight summary
  tier to 1024 records. Current sizing estimates keep 128 full-board 225-move
  games around 330 KiB before future growth; older stats roll into
  `match_history.archived_stats` instead of retaining every move list.
- Avoid always-on listeners for full match history unless the UI actually needs
  live updates.
- Prefer one coalesced profile snapshot write over per-match cloud documents for
  casual play.
- Keep routine signed-in profile refreshes read-only; write profile updates only
  when a user-visible setting or provider-owned field changed.
- Defer signed-in profile/settings sync to sign-in, retry, and match-finish
  checkpoints so rapid name typing or default-rule toggles stay local until a
  meaningful cloud sync point.
- Enforce a 5-minute server-side cooldown between normal profile snapshot
  updates in Firestore rules. Reset-barrier writes can bypass the normal edit
  cooldown, but are still scoped to real reset writes.
- Keep public replay publishing out of `v0.3`; private history and public
  shareables have different cost and trust profiles.
- Budget cloud history as profile writes, not match creates. The free-tier write
  knob is now the sync interval: `1440 / interval_minutes` writes per
  continuously dirty active user per day.

Snapshot writes use optimistic transactions: a stale content/reset barrier is
rejected instead of replacing another device's data. Offline transactions fail
and leave work pending; the app does not promise offline cloud commit. The
five-minute gate remains a write coalescing policy, not concurrency control.

## Cost Formulas

These formulas are intentionally conservative enough for release planning.

Let:

- `P = explicit profile loads per signed-in user per day` (sign-in, reconnect, or pending-sync refresh; route opens normally reuse memory).
- `S = profile/history snapshot syncs per signed-in user per day`.
- `I = sync interval in minutes`; currently `5`.

Approximate daily operations per signed-in user:

| Flow | Reads | Writes | Deletes |
|---|---:|---:|---:|
| Explicit signed-in profile loads | `P` | `0` normally | `0` |
| Dirty profile/history syncs | `S` minimum; retries/preflight add reads | `S`, capped by `1440 / I` | `0` |
| Reset Profile | `1..2` | `1` | `0` |

With `I = 5`, a continuously dirty signed-in user can write at most 288 profile
snapshots per day. Real users should be far lower because idle users do not
write, and multiple matches/profile edits inside a 5-minute window coalesce
into the next snapshot.

## Headroom Scenarios

These are deliberately rough. They are meant to catch order-of-magnitude
mistakes before implementation.

### Small Public Release

Assumption: 100 signed-in users/day, each loads the account profile twice and has one dirty
cloud snapshot sync.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~300/day before retries/preflight | ~166x below 50,000/day |
| Writes | ~100/day | ~200x below 20,000/day |
| Deletes | 0/day | No concern |

This estimate is inside the listed free quotas; actual usage and retries need monitoring.

### Profile-Heavy Usage

Assumption: 500 signed-in users/day, each explicitly reloads the account profile 5 times.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~2,500/day | ~20x below 50,000/day |
| Writes | ~0/day before match saves | No concern |

Opening Profile with an already loaded account normally costs no network read.
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

The current 5-minute interval is the alpha default. It gives user-visible sync a
shorter feedback loop while keeping writes bounded by one profile snapshot per
dirty interval.

### One-Time Promotion Spike

Assumption: 100 existing local players sign in for the first time with 128 local
matches each.

| Metric | Estimate | Free-tier headroom |
|---|---:|---:|
| Reads | ~200 one-time reads before retries | ~250x below 50,000/day |
| Writes | ~200 one-time writes | ~100x below 20,000/day |
| Deletes | 0 | No concern |

This estimate remains below the listed quotas because promotion writes one capped profile
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

### Deferred P2P Multiplayer Planning

The [October 21 research](../backend/p2p_multiplayer.md) favors direct WebRTC
gameplay with backend signaling; multiplayer is deferred from v0.6. Moves would
not use Firestore or Cloud Run. Budget backend usage by session setup and
reconnection, rather than moves or game duration. The signaling mailbox would
initially target existing Firestore; its schema, rules, expiry, and cleanup
remain to be designed.

Measure room creation/joining, offers/answers, connection candidates, retries,
reconnections, and cleanup. Account separately for existing auth and optional
private-history sync. Bound payloads, attempts, and retained rooms, and detach
signaling listeners when no longer needed. Direct-only connectivity is an
accepted limit of the preferred PoC; neither TURN nor database move relaying is
a selected fallback. There is no measured free-tier player/game capacity yet.

### Selected v0.6 Sharing Design

The local v0.6 candidate implements [bounded sharing data](../backend/sharing_data.md).
It is not deployed. Portable copy/view/scrub/analysis and guest-library operations
perform zero Firestore operations. Ordinary signed-in played-game history retains its existing coalesced profile
sync, including completed Play From Here continuations. New
account-library operations are deliberate,
with no background polling or persistent application listener. SDK server reads
may use a temporary Listen transport internally.

Logical document operations for a successful, uncontended attempt:

| Action | Client document reads | Writes | Notes |
|---|---:|---:|---|
| Public empty modern profile | 1 | 0 | Root manifest has no entries, so no page fetch |
| Modern public game page | Root once + needed physical pages | 0 | Eight logical rows across storage holes; cached pages reused |
| Legacy public profile without counts | 5 | 0 | Root + four bounded pages once; no migration write |
| Open owner management | 5 | 0 | Identity and four pages, only on explicit open/reload |
| Enable / disable | 5 | 5 | Atomic identity and four empty markers |
| Rename enabled profile | 1 | 1 | Root only; preserves existing stats and exact timestamp |
| Share / refresh / remove public stats | 1 | 1 | Explicit root transaction, independent of catalog pages |
| Open publication preview | 1 | 0 | Deliberate owner root load |
| Commit publication from preview | 7 | 2 | Fresh root + four pages + transaction root/page; first legacy manifest adds three reads |
| Unpublish from loaded management | 2 | 2 | Transaction root/page plus atomic manifest update, preserving private copies |
| Load account library | 1 | 0 | Missing library is not created by read |
| Save / remove after load | 1 | 1 | Transaction compares observed revision |
| First account save | 2 | 1 | Baseline load plus transaction |
| Reset account library | 2 | 1 | Additional to existing profile/history reset |
| Clear sharing on Delete Cloud | 6 | 6 | Atomic public root, four pages and private empty marker; additional to old profile deletion |

These are application document operations, not transport-request or billing
measurements. Rules can charge dependent `get`/`exists`/`getAfter` reads, including
on denied requests; retries cost more. Visibility batches check four pages and
the root. A changed/initialised catalog manifest validates before/after states
of all four pages, so budget for rule-dependent reads as well; unchanged
manifests avoid that fan-out. The instrumented counts exclude these rule
lookups. Firebase's
[billing contract](https://firebase.google.com/docs/firestore/pricing#cloud_firestore_security_rules)
and [free quota](https://firebase.google.com/docs/firestore/quotas#free_quota)
were checked October 21, 2026; emulator results cannot prove a billing allowance.

The public catalog is 32 entries (four pages of eight); private library is 64
entries. Payloads are at most 8,192 ASCII characters. The measured largest legal
portable fixture uses 1,804 characters, including 225 moves and maximum Unicode
names/UIDs. Current operation instrumentation and bounded document sizes live in
the [0.6.1 sharing receipt](../../working/v0_6_1_evidence/sharing_shape.json);
the [original v0.6 evidence](../../working/v0_6_evidence/README.md) retains portable
fixture and earlier candidate measurements. JSON fixture sizes exclude
Firestore document-name/field/index overhead and serialized network framing.
Payload fields are exempted from automatic indexes. Six empty markers remain
after cloud deletion, without names, statistics or replay payloads. The legacy profile-delete
guard can add two dependent sharing-document reads to the prior deletion flow.

Caps bound each account’s retained data. Public visits are still billable work,
and repeated traffic is not globally rate-limited by this design. A shared URL
is the zero-database demo path. The operational `publicSharingEnabled()` switch
blocks public reads/new publications while retaining owner cleanup. This adds
no Cloud Run service, move relay, scheduled cleanup, storage bucket or upload
pipeline. No free-tier player/visit capacity is claimed.

Future server-authoritative matches would require a separate estimate.

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

## Parked Cloud Run Budget

Cloud Run is outside the preferred P2P design, and multiplayer itself is
deferred from v0.6. If a later phase selects Cloud Run, use request-based billing
in `us-central1` and keep minimum instances at zero.

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

`v0.3.1` note: the first local-build local profile promotion smoke imported 24
private match documents for one signed-in profile under the old pre-v3
per-match model. This was well inside the free tier, but it exposed the
per-match-write cost model that `v0.3.3` later replaced.

`v0.3.2` note: production and local-build smoke covered signed-in
`cloud_saved` writes, cloud-history reload, Reset Profile deletion/barrier
behavior, and post-reset saves. At that point, one finished signed-in match
wrote one private match document after a small number of dedupe/profile reads,
and Reset Profile paid one profile write plus one delete per private
`client_uploaded` match cleared. That per-match document model was replaced in
`v0.3.3`.

`v0.3.3` note: routine existing-profile loads are read-only when cloud profile
fields are already current. Casual private history moved from one match document
per save to an embedded, capped `match_history` profile snapshot. Profile,
settings, and history updates now share one coalesced profile write lane with a
5-minute rules-enforced cooldown; Reset-barrier writes can bypass the normal
cooldown so Reset Profile is not blocked by a recent profile sync.
