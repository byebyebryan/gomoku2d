# Portable replay v1

Portable replay v1 is the allowlisted record used by shared replay links and
the separate saved-replay library. It stores a complete game from an empty
15x15 board under the current free-opening rules. Black moves first. V1 accepts
only freestyle or Renju games that finish with a win or a full-board draw.
Tournament openings, arbitrary starting positions, resignation, clocks, and
other terminal causes need a later format version.

The implementation lives in `gomoku-web/src/replay/portable_replay.ts`,
`portable_replay_codec.ts`, and `portable_replay_adapters.ts`.

## Wire format

The static shared route is `/replay/shared/`. The URL fragment has the form
`#g2r1=<payload>`, where `<payload>` is unpadded base64url of compact UTF-8
JSON. Fragments keep the game data out of HTTP requests and server logs. The
encoded payload is limited to 8,192 characters and the decoded JSON to 6,144
bytes. The largest current fixture (225-move draw, 80 four-byte characters
for each label, two 128-character UIDs, practice origin, and viewing ply) is
1,353 decoded bytes and 1,804 encoded characters.

The compact JSON object uses these keys in this exact order:

| Key | Meaning |
|---|---|
| `v` | Format version, `1` |
| `r` | Ruleset: `freestyle` or `renju` |
| `m` | Unpadded base64url of one byte per move; each byte is a row-major cell from 0 through 224 |
| `b`, `w` | Black and White participant snapshots |
| `o` | Optional practice origin, `{ "f": fingerprint, "p": inheritedPly }` |
| `at` | Optional viewing ply, from 0 through the move count |

The `o` field, when present, follows `w`; `at`, when present, follows `o` (or
`w` if there is no origin). Participant keys are compact too. A human is
`{ "k": "h", "n": name }` with optional `"u": profileUid`. A bot is
`{ "k": "b", "n": name, "e": engine, "v": version, "cv": configVersion, "c": config }`.
The config is the current strict `BotConfig` shape: presets have `mode`,
`preset`, `version`; custom configs have `mode`, `depth`, `extraPass`,
`scoring`, `version`, `width`. Unsupported config keys or combinations are
rejected. The v1 bot identity is `search_bot` version 1 with config version 1.

All objects are serialized in the order above, with optional fields omitted
when absent. Decoding rejects padding, invalid base64url, non-canonical pad
bits, unknown fields, wrong field order, alternate JSON encodings, malformed
UTF-8, and non-canonical move-byte encodings. A parser must not silently strip
unrecognized data.

## Validation and limits

Validate metadata and bounds before creating a Wasm board. Each label is 1–80
Unicode code points, at most 320 UTF-8 bytes, and has no control characters.
An optional human profile UID must match `^[A-Za-z0-9_-]{1,128}$`; it is only a
public-profile reference, not a provider or local account identifier. A
practice source fingerprint is the canonical unpadded base64url form of a
32-byte SHA-256 digest (43 characters). Its source is not fetched or verified.

Move arrays contain 9–225 distinct integer cells in `[0, 224]`. The validator
replays every move in order through `createWasmBoard` / `applyWasmMove` from the
empty board. It rejects illegal Renju moves, duplicates, moves after a win,
and games that do not finish. The final status is derived from Wasm; a stored
`SavedMatchV2` adapter input is rejected if its saved status disagrees.

When `o` is present, `p` must be an integer from 4 through one less than the
final move count. Its prefix is part of the complete standalone move sequence
and is checked as a legal, still-ongoing Wasm position. The portable record
contains no local source-record ID or recursively embedded ancestry.

The public TypeScript APIs are:

- `validatePortableReplay(value)` validates metadata and full move legality,
  returning `{ replay, status }` with a derived status.
- `portableReplayFromSavedMatch(value, viewPly?)` allowlists player snapshots
  and moves from `SavedMatchV2`, then checks the saved status against Wasm.
- `encodePortableReplay(value)` and `decodePortableReplay(payload)` encode or
  decode the strict wire format. The fragment helpers add or read `#g2r1=`.
- `portableReplayFingerprint(validated)` hashes the canonical replay identity.
- `portableReplayToSavedMatch(value, viewedAt?)` makes an ephemeral
  `SavedMatchV2` projection for the existing viewer and analyzer, returning it
  with the portable replay and fingerprint.

The projection is marked `source: "cloud_saved"` and
`trust: "client_uploaded"`, uses null local profile IDs, and derives result
from Wasm. It is a viewer adapter, not a write into ordinary played history.
The existing portable `practice` origin field describes a continuation, without
creating a separate UI category. Its `inheritedPly` maps to the viewer's undo floor.
Normal played continuations store the same origin as optional SavedMatchV2
`replay_origin: {source_fingerprint, inherited_ply}`. Promotion preserves it and
export maps it back to the portable origin; the complete game remains shareable.

## Identity and privacy

The replay fingerprint is SHA-256 over the UTF-8 bytes of
`gomoku2d-portable-replay-v1\0` followed by canonical compact portable JSON.
It includes the ruleset, complete moves, both participant snapshots, bot
configuration, and optional practice origin. It excludes `viewPly`, derived
status, trust, and adapter-only storage metadata. Missing optional fields have
one canonical representation. Changing attribution or practice origin creates
a distinct identity; changing only the initial viewing ply does not.

The allowlist excludes saved-match IDs, local profile IDs, authentication
provider data, private settings, trust claims, analysis frames, and stored
results. A profile UID and practice fingerprint remain untrusted client claims.
Recipients rebuild the board and recompute bounded analysis in their own
browser; the link carries no trusted analysis or server-verification flag.

The Vitest suite exercises the installed Rust/Wasm bridge through the same
validator for both rulesets. Production-browser direct-route, static hosting,
and guest/signed-in network-isolation checks remain part of M2 acceptance.
