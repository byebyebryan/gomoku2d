# Future Cloud Run Backend

Purpose: park target backend ideas that are not deployed today.

The current backend contract lives in [`backend.md`](backend.md). This file is
not current scope unless a later phase explicitly selects server authority or
other trusted server-side work.

The October 21 [P2P multiplayer research](p2p_multiplayer.md) favors backend
signaling only, but multiplayer is deferred from v0.6. This Cloud Run sketch
remains parked and is not a dependency of that preferred PoC. Its trusted-match
assumptions apply only if a future phase explicitly selects server authority.

## Why Cloud Run

Cloud Run is the path when browser authority is not enough:

- server-validated online moves;
- atomic username reservation;
- replay verification and publishing;
- stronger bot moves outside browser compute limits;
- puzzle generation from trusted history.

The Rust service should reuse `gomoku-core` as the rules authority, matching the
browser wasm engine without duplicating game logic in TypeScript.

## Candidate Service Shape

Start as one Rust binary in the existing workspace, likely
`gomoku-bot-lab/gomoku-api/`, and split to a top-level backend only if deploy
cadence or ownership clearly diverges.

Likely endpoints:

- `POST /reserve_username`
- `POST /match`
- `POST /match/{id}/move`
- `POST /verify`
- `POST /bot/move`

Firestore remains the durability/fanout layer. Cloud Run is the trusted writer
for authoritative match state.

## Trusted Match Sketch

```text
client A            Cloud Run             Firestore             client B
   | POST /match       |                      |                    |
   |------------------>| create match doc     |                    |
   |<------------------| match id             |                    |
   | subscribe         |                      |<-----subscribe-----|
   | POST /move        | validate gomoku-core |                    |
   |------------------>| write move/result    |----snapshot------->|
   |<------------------| accepted             |----snapshot------->|
```

Clients do not directly write authoritative moves.

## Trust Rules

- `client_uploaded` casual matches can receive private analysis.
- `server_verified` matches can feed public/ranked surfaces.
- Published replay state should be explicit, not automatic for every saved game.

## Deployment Model

Planned split:

- GitHub Actions deploy credentials use Workload Identity Federation.
- Runtime credentials use an attached service account and ADC.
- Third-party secrets go in Secret Manager with pinned versions.

No long-lived JSON keys should be required.
