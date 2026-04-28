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
| `profiles/{uid}` | owner read | owner client | Cloud identity and user settings |
| `profiles/{uid}/matches/{matchId}` | owner read | owner client for casual imports; server later for trusted records | Private cloud match history |
| `matches/{matchId}` | participants read | server | Future trusted live match state |
| `replays/{replayId}` | public read | owner/server on explicit publish | Future shareable replay projection |

Guest-local state is not mirrored one-to-one in Firestore. On sign-in, the app
imports finished local matches into private cloud history while leaving local
copies on-device.

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
  schema_version: 1;
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
  with `match_saved_at <= history_reset_at`.
- `schema_version` is always `1`; increments require a writer + rules + reader
  update in the same slice.
- Writer behavior: during guest promotion, the browser only sends
  `display_name` if the local guest has a custom name and the loaded cloud name
  still matches the provider default. If the local display name is still the
  default `Guest`, the app adopts the provider/cloud display name locally
  instead of overwriting cloud state with `Guest`.
- Firestore rules enforce ownership, shape, write timestamps, and reset-barrier
  movement for profile writes. The browser can leave `history_reset_at`
  unchanged or set it to the current write's `request.time`; it cannot move the
  barrier backward, remove it, or set an arbitrary timestamp.
- Reset Profile while signed in writes `history_reset_at`, resets cloud profile
  display/default-rule fields to provider/default values, deletes private
  `client_uploaded` match documents where rules allow it, and clears this
  device's local/cloud caches.

## Private Match History

Path: `profiles/{uid}/matches/{matchId}`

### Document IDs

Imported guest matches use deterministic IDs:

```ts
`local-${encodeURIComponent(local_match_id)}`
```

The deterministic ID makes promotion idempotent. Retrying the same local import
checks and skips the already-created cloud document.

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

**`guest_import`** — local match promoted to cloud after sign-in:

```ts
type CloudImportedMatchV1 = SavedMatchV1 & {
  id: string; // deterministic: "local-${encodeURIComponent(local_match_id)}"
  source: "guest_import";
  trust: "client_uploaded";

  local_match_id: string;
  local_origin_id: string; // "guest:{guestProfileId}:{localMatchId}"

  imported_at: Timestamp; // server timestamp of the import write
  match_saved_at: Timestamp; // Firestore-comparable copy of saved_at
};
```

**`cloud_saved`** — match played while signed in, saved directly to cloud:

```ts
type CloudDirectSavedMatchV1 = SavedMatchV1 & {
  id: string; // same as the local match UUID (no encoding needed)
  source: "cloud_saved";
  trust: "client_uploaded";

  created_at: Timestamp; // server timestamp of the cloud write
  match_saved_at: Timestamp; // Firestore-comparable copy of saved_at
};
```

Key differences from `guest_import`:
- No `imported_at`, `local_match_id`, `local_origin_id` fields.
- `id` is the raw local UUID rather than a prefixed encoding.
- Human player's `local_profile_id` is always `null`; use `profile_uid` for
  cross-device identity matching (see `matchUserSide` in `saved_match.ts`).
- `match_saved_at` is duplicated from the ISO `saved_at` field as a Firestore
  timestamp so security rules can reject stale writes after Reset Profile.
- Reset Profile deletes only private `client_uploaded` records in this
  subcollection; future `server_verified` history must not be deleted by the
  owner-client reset path.

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
because it is useful summary metadata and lets rules assert
`move_cells.size() == move_count`. Current Firestore rules also restrict
`move_cells` values to valid 15x15 board indexes.

Cloud match documents also store `match_saved_at`, the timestamp form of
`saved_at`. The ISO string remains the app-facing replay/display field; the
timestamp exists so Firestore rules can compare match age against profile reset
barriers without parsing strings.

## Indexing Notes

`move_cells` is not queried directly and should not be indexed. The repo-level
`firestore.indexes.json` disables single-field indexes for the `matches`
collection group field.

Expected query fields for private history are metadata fields such as
`saved_at`, `variant`, `status`, and future trust/source fields.

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

Cloud guest promotion then turns the canonical local record into a
`CloudImportedMatchV1` by changing `id`, `source`, `trust`, human `profile_uid`,
and adding import metadata. It does not reinterpret moves or player sides.

## Migration Notes

There is no production cloud match history before `CloudImportedMatchV1`, so the
first promotion release can tighten the shape without a cloud data migration.
Future schema versions should either:

- keep readers backward-compatible for existing versions, or
- run a one-time migration and record it here.
