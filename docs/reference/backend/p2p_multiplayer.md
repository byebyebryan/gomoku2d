# Deferred P2P Multiplayer Research

Owner: Bryan. Research and deferral recorded October 21, 2026. Multiplayer is
deferred from v0.6 because of implementation/UX effort, uncertain usage, and the
coordination needed to demonstrate it. No multiplayer work is implemented or
deployed. Preserve this preferred backend direction if multiplayer is revisited;
its UI/UX and detailed protocol remain open.

## Goal

Gomoku2D is primarily a demo and production experiment. Optimize for minimal
backend usage as gameplay volume grows, within the existing GCP free allowances.
Keeping deployment work small matters, but removing backend work from each move
is the stronger constraint. Low expected traffic is not a reason to design
per-move compute or database traffic into the game.

## Preferred Responsibilities If Revisited

| Part | Direction |
|---|---|
| App delivery | Existing static hosting |
| Finding/joining a peer | A small signaling mailbox; reuse Firestore as the initial candidate |
| Gameplay transport | Direct WebRTC data channel between the two browsers |
| Rules and results | Each browser validates moves with the shared Rust/Wasm core |
| Reconnection state | Peers reconcile an agreed move history over the data channel |
| Replay storage | Local by default; existing private cloud continuity remains optional |

The backend introduces the players and exchanges connection information. It
does not validate moves, store a live move stream, or adjudicate results.
Signaling can be needed again when a peer reconnects.

An established direct game should continue if signaling becomes unavailable,
provided its peer connection remains usable. Rematches should reuse that
connection where practical.

## Cost Boundary

- Backend usage should primarily follow session setup and reconnection, rather
  than move count, game duration, or an open server connection per match.
- Keep room records, connection attempts, signaling payloads, and retention
  bounded. Avoid periodic backend presence writes and broad lobby listeners.
- An ordinary move should produce no application-backend read, write, or
  compute request.
- Local replay persistence is separate from optional private cloud sync.
- The preferred PoC would not require a Cloud Run match service, authoritative
  game server, or Firestore move relay.

This is a scaling goal, not a measured capacity claim or a guarantee of zero
charges. Session setup, retries, cleanup, auth, and optional history sync still
have usage. Measure those operations before claiming support for a particular
number of players or games within the free allowances.

## Connectivity Boundary

The preferred approach is direct WebRTC with STUN-assisted discovery. An initial
PoC would accept that some NAT/firewall combinations cannot establish a direct
path, with a clear connection failure and a useful retry/leave path.

TURN and database move relaying are not automatic fallbacks. Either would be
a separate decision with its own infrastructure and cost assessment. The STUN
provider and the signaling protocol, access rules, expiry, and cleanup policy
still need implementation choices; no provider availability or connection
success rate has been established.

## Agreement And Trust

Both peers validate legal moves and check turn order, protocol/rules
compatibility, and agreed state. Reconnection must reconcile move history
before play resumes. These checks detect invalid input and divergence; they
do not create an independent referee or server-verified result.

P2P history remains casual client-validated history. Do not assign
`server_verified` trust to it. Disconnects, interrupted sessions, and disputed
states need their own product treatment rather than an invented win/loss.
The persisted representation is a later implementation contract; this decision
does not change current schemas or open any Firestore namespace.

## Design Questions When Resumed

Work through the player journey before choosing layouts: entry and invitation,
waiting and joining, rules/colors/readiness, active play and move acknowledgement,
connection failure and reconnection, leaving, completion, rematch, and replay.
Identity requirements and the treatment of local assists such as hints and undo
remain UI/product decisions. Preserve the existing visual language and
board-first screen roles while resolving them.

## References

- [Roadmap](../product/roadmap.md)
- [Current backend](backend.md)
- [Cost planning](../ops/backend_cost.md)
- [WebRTC peer connections and signaling](https://webrtc.org/getting-started/peer-connections)
- [Firebase signaling example](https://webrtc.org/getting-started/firebase-rtc-codelab)
- [WebRTC relay requirements](https://webrtc.org/getting-started/turn-server)
