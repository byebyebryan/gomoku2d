# Data Model

Scope: canonical persisted data shapes for Gomoku2D local and cloud state.

This document is the schema contract for persisted app data. Backend design and
trust boundaries live in `backend.md`; live project setup and deployed rulesets
live in `backend_infra.md`.

## Schema Policy

- Every persisted document family with app-managed structure carries an explicit
  `schema_version`.
- Version numbers are monotonically increasing integers.
- A schema change must update writer code, reader/decoder code, Firestore rules,
  tests, and this document in the same slice.
- Stored fields should be either canonical source data or query/display metadata.
  Derived UI details should be reconstructed from canonical data where practical.
- Private client-uploaded records are not trusted for ranked/public claims. They
  are continuity records for the signed-in owner.

## Collections

| Path | Visibility | Writer | Purpose |
|---|---|---|---|
| `profiles/{uid}` | owner read | owner client | Cloud identity, user settings, and capped private recent history |
| `profiles/{uid}/matches/{matchId}` | disabled for casual history | server later | Reserved for future server-verified/shareable match records |
| `matches/{matchId}` | participants read | server | Future trusted live match state |
| `replays/{replayId}` | public read | owner/server on explicit publish | Future shareable replay projection |

Guest-local state is not mirrored one-to-one in Firestore. On sign-in, the app
merges the latest local matches into the signed-in profile's capped
`recent_matches` snapshot while leaving local copies on-device.

## `profiles/{uid}`

Current profile documents are owned by the signed-in user.

```ts
type CloudProfileDocument = {
  auth_providers: string[];
  avatar_url: string | null;
  created_at: Timestamp;
  display_name: string;
  email: string | null;
  history_reset_at?: Timestamp | null;
  last_login_at: Timestamp;
  preferred_variant: "freestyle" | "renju";
  recent_matches: {
    matches: SavedMatchV1[]; // newest first, capped at 24
    schema_version: 1;
    updated_at: Timestamp | null;
  };
  schema_version: 2;
  uid: string;
  updated_at: Timestamp;
  username: string | null;
};
```

Rules:

- `uid` must match the document ID.
- `created_at` and `username` are app-owned after creation.
- `history_reset_at` is a reset barrier. When present, local promotion, direct
  sync retry, cloud history load, and active-history resolution ignore matches
  with `saved_at <= history_reset_at`.
- `schema_version` is always `2`; increments require a writer + rules + reader
  update in the same slice.
- `recent_matches.matches` stores the latest private cloud history snapshot
  directly inside the profile document. The browser writes a merged snapshot,
  not one document per match. Firestore rules cap the array at 24 records and
  require `recent_matches.updated_at` to be the current request time whenever
  the snapshot changes.
- Writer behavior: during guest promotion, the browser only sends
  `display_name` if the local guest has a custom name and the loaded cloud name
  still matches the provider default. If the local display name is still the
  default `Guest`, the app adopts the provider/cloud display name locally
  instead of overwriting cloud state with `Guest`.
- Firestore rules enforce ownership, shape, write timestamps, and reset-barrier
  movement for profile writes. The browser can leave `history_reset_at`
  unchanged or set it to the current write's `request.time`; it cannot move the
  barrier backward, remove it, or set an arbitrary timestamp.
- Firestore rules also enforce a 15-minute cooldown between normal profile
  snapshot updates using `updated_at`. Profile changes and recent-history
  changes remain local-first and coalesce into the next eligible sync checkpoint.
  Reset-barrier writes can bypass the normal edit cooldown so Reset Profile is
  not blocked by a recent profile sync.
- Reset Profile while signed in writes `history_reset_at`, resets cloud profile
  display/default-rule fields to provider/default values, clears
  `recent_matches.matches`, and clears this device's local/cloud caches.

## Private Match History

Current casual private history is embedded in `profiles/{uid}.recent_matches`.
The old `profiles/{uid}/matches/{matchId}` casual subcollection path is closed
in rules for `v0.3.3`; keep that namespace reserved for future
server-verified/shareable records rather than private browser snapshots.

### Saved Match v1

Current `schema_version: 1` is shared by browser-local history and private
cloud history. Cloud records diverge only in source-specific fields; the core
replay payload and player shape stay aligned across all sources.

```ts
type SavedMatchV1 = {
  id: string;
  schema_version: 1;
  board_size: 15;

  source: "local_history" | "guest_import" | "cloud_saved";
  trust: "local_only" | "client_uploaded" | "server_verified";

  match_kind: "local_vs_bot" | "local_pvp" | "online_pvp" | "puzzle_challenge";
  variant: "freestyle" | "renju";
  status: "black_won" | "white_won" | "draw";

  move_count: number;
  move_cells: number[];
  undo_floor: number;

  player_black: SavedMatchPlayer;
  player_white: SavedMatchPlayer;

  saved_at: string;
};
```

**`cloud_saved`** — local match included in the signed-in profile snapshot:

```ts
type CloudDirectSavedMatchV1 = SavedMatchV1 & {
  id: string; // same as the local match UUID
  source: "cloud_saved";
  trust: "client_uploaded";
};
```

All embedded cloud history records currently use `source: "cloud_saved"` and
`trust: "client_uploaded"`, whether they were first played before or after
sign-in. Human player's `local_profile_id` is always `null`; use `profile_uid`
for cross-device identity matching (see `matchUserSide` in `saved_match.ts`).

```ts
type SavedMatchPlayer = {
  kind: "human" | "bot";
  profile_uid: string | null;
  local_profile_id: string | null;
  bot: SavedMatchBotIdentityV1 | null;
  display_name: string;
};
```

Side is encoded by the field name (`player_black` or `player_white`), not
duplicated inside the player object. `display_name` is a replay snapshot, not
an identity key.

**Identity matching across devices:** Use `matchUserSide(match, { profileUid,
localProfileId })` from `saved_match.ts`. It tries `profile_uid` first (correct
for cloud matches on any device) then falls back to `local_profile_id`
(correct for local-only and guest-imported records). For `cloud_saved` records,
`local_profile_id` is always `null` on the human player — only `profile_uid`
identifies them.

### Bot Identity

Bot participants are not fake users. They carry a bot identity snapshot:

```ts
type SavedMatchBotIdentityV1 = {
  id: "practice_bot";
  version: 1;
  engine: "baseline_search";
  config_version: 1;
  config: {
    kind: "baseline";
    depth: 3;
  };
};
```

This is intentionally more structured than the current UI needs. Future bot
personalities, benchmark-backed presets, and stronger analysis bots can add new
bot IDs/config versions through an explicit writer, reader, rules, and docs
slice without pretending every bot is the same opponent.

### Move Encoding

`move_cells` is the canonical replay payload.

Each move is encoded as:

```ts
row * board_size + col
```

For the current 15x15 board, valid cell indexes are `0..224`. The array index
derives the rest:

- move number is `index + 1`
- black plays even indexes
- white plays odd indexes

This keeps the high-volume replay payload compact while preserving an ordered,
lossless move history. Human-readable notations such as `A1` can be generated at
the UI/export layer later, but are not the canonical private storage format.

### Derived Fields

Winning cells are not stored in v1. For a won match, reconstruct the board from
`move_cells`, then ask the rules/core layer for the winning line from the final
move. This avoids storing redundant derived state that can disagree with the move
history.

`move_count` is stored even though it is derivable from `move_cells.length`
because it is useful summary metadata. Embedded private-history records are
client-uploaded continuity records; Firestore rules validate the top-level
profile snapshot shape and cap rather than every nested replay field.

## Indexing Notes

`recent_matches` and `move_cells` are not queried directly and should not be
indexed. The repo-level `firestore.indexes.json` disables single-field indexes
for the embedded `profiles.recent_matches` field and keeps the old
`matches.move_cells` override for the future reserved match namespace.

## Local Guest History

Current local history lives in browser `localStorage` under
`gomoku2d.guest-profile.v2`. Finished local matches use `SavedMatchV1` with:

- `source: "local_history"`
- `trust: "local_only"`
- `id` equal to the browser-local match ID

```ts
type GuestSavedMatch = SavedMatchV1 & {
  id: string;
  source: "local_history";
  trust: "local_only";
};
```

Legacy local history from `gomoku2d.guest-profile.v1` is migrated into v2 on
store creation. The old v1 key is left untouched as a rollback/re-import safety
net. The migration:

- converts `mode: "bot"` to `match_kind: "local_vs_bot"`
- encodes verbose `moves` as `move_cells`
- converts local `players` into fixed `player_black` and `player_white`
- derives local side from the human player instead of storing `guestStone`
- drops `winningCells`; replay/result screens reconstruct them from moves
- drops malformed schema-version-1 records instead of loading them into the UI

Cloud profile sync turns canonical local records into embedded `cloud_saved`
records by changing `source`, `trust`, and the human player's `profile_uid`.
It does not reinterpret moves or player sides.

## Migration Notes

There is no meaningful production cloud match history before the profile-v2
embedded snapshot pivot, so this slice intentionally updates alpha data in
place instead of carrying a migration layer. Future schema versions should
either:

- keep readers backward-compatible for existing versions, or
- run a one-time migration and record it here.
