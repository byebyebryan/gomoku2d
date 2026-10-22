# Backend

Purpose: document the current backend contract behind the web app.

The current deployed backend is Firebase only. [P2P multiplayer research](p2p_multiplayer.md)
records a preferred signaling-only backend and browser-owned gameplay, deferred
from v0.6. It is not implemented or deployed. Longer-term authoritative service
ideas remain parked in [`future_cloud_run.md`](future_cloud_run.md).

## Current Deployed Backend

- Firebase Auth for optional Google sign-in.
- Firestore `(default)` in `us-central1`.
- `profiles/{uid}` stores owner-scoped cloud profile, settings, and capped
  private match history.
- Firestore rules in this repo enforce ownership, schema shape, cooldowns,
  reset barriers, and closed future namespaces.
- No Cloud Run service is deployed.
- Local/casual gameplay, bot moves, tactical hints, and replay analysis run in
  the browser through Rust/WebAssembly.

Operational setup lives in [`backend_infra.md`](../ops/backend_infra.md).
Cost posture lives in [`backend_cost.md`](../ops/backend_cost.md). Persisted
schema details live in [`data_model.md`](data_model.md).

## Trust Lanes

Keep these lanes separate:

| Lane | Authority | Storage | Public/trusted? |
|---|---|---|---|
| Local guest play | Browser | Local storage | No |
| Signed-in casual history | Browser upload, owner-scoped rules | `profiles/{uid}.match_history` | Private only |
| Casual P2P match (deferred research) | Both browsers validate through Rust/Wasm | Local history; optional private continuity | No server-verified result |
| Published replay | Explicit future publish step | Public replay doc | Public, but trust depends on source |
| Trusted online/ranked match | Future server authority | Server-written match record | Yes |

Local play should remain complete without Firebase config. Cloud sign-in extends
continuity; it should not become the default entry gate.

## Identity Model

Three layers:

1. Local profile: browser-managed identity before sign-in.
2. Firebase user: cloud auth identity after sign-in.
3. Gomoku cloud profile: game-owned profile document at `profiles/{uid}`.

Google is the current provider. GitHub can be added later if a product surface
needs it.

## Cloud Profile Shape

The exact schema is versioned in [`data_model.md`](data_model.md). The important
contract:

- profile identity and display name;
- saved bot/rule/hint/touch settings;
- reset barrier;
- capped private replay matches;
- lightweight summaries and archived stats for older history.

Private match history is embedded in the profile document for the current
casual path. Future trusted/public match records should use server-owned docs.

## Promotion Flow

When a local player signs in:

1. Firebase returns a stable `uid`.
2. The app creates or loads `profiles/{uid}`.
3. Existing cloud settings restore once for that sign-in; a new account starts
   from this device's settings. Later device edits upload at the next sync.
   Eligible finished local history is merged into the account snapshot.
4. Imported records keep stable local-origin IDs so retries are idempotent.
5. Local-only history remains local unless explicitly promoted/synced.

## Firestore Rules Contract

Current rules allow owners to read/write their own profile only when the write
matches the expected schema and owner constraints.

Rules also enforce:

- immutable app-owned fields where needed;
- update cooldowns for normal profile snapshots;
- reset-barrier writes for destructive profile reset;
- private match-history caps;
- closed future namespaces until their corresponding designs and rules are
  implemented; the P2P decision alone opens no namespace.

Any change to cloud profile schema should update `data_model.md`, Firestore
rules, and rules tests together.

## Future Backend Boundary

If casual multiplayer is revisited, the preferred backend role is peer
discovery/joining and signaling; moves would travel directly between browsers.
The [deferred P2P research](p2p_multiplayer.md) records this architecture and its
direct-connectivity limitation.

Use Cloud Run only when the browser cannot safely own the action:

- username reservation;
- trusted online match authority;
- public replay verification/publishing;
- server-side puzzle generation;
- strong bot endpoint if browser wasm is insufficient.

Those designs are tracked in [`future_cloud_run.md`](future_cloud_run.md), not
this current backend contract.
