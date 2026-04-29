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
| `profiles/{uid}` | owner read | owner client | Cloud identity, user settings, and capped private match history |
| `profiles/{uid}/matches/{matchId}` | disabled for casual history | server later | Reserved for future server-verified/shareable match records |
| `matches/{matchId}` | participants read | server | Future trusted live match state |
| `replays/{replayId}` | public read | owner/server on explicit publish | Future shareable replay projection |

Local profile state is not mirrored one-to-one in Firestore. On sign-in, the app
merges the latest local matches into the signed-in profile's capped
`match_history` snapshot while leaving local copies on-device.

## `profiles/{uid}`

Current profile documents are owned by the signed-in user.

```ts
type CloudProfileDocument = {
  auth: {
    providers: Array<{
      provider: "google.com" | "github.com";
      display_name: string | null;
      avatar_url: string | null;
    }>;
  };
  created_at: Timestamp;
  display_name: string;
  match_history: {
    replay_matches: SavedMatchV1[]; // newest first, capped at 128
    summary_matches: CloudMatchSummaryV1[]; // next tier, capped at 1024
    archived_stats: CloudArchivedMatchStatsV1;
  };
  reset_at: Timestamp | null;
  schema_version: 3;
  settings: {
    default_rules: {
      ruleset: "freestyle" | "renju";
      opening: "standard"; // future-safe slot for openings such as swap2
    };
  };
  uid: string;
  updated_at: Timestamp;
  username: string | null;
};
```

Rules:

- `uid` must match the document ID.
- `created_at` and `username` are app-owned after creation.
- `auth.providers` stores provider-sourced display/avatar metadata without
  copying email into the app profile document.
- `settings.default_rules.ruleset` is the user's default casual rule set.
  `settings.default_rules.opening` is currently always `standard` and exists so
  later rule/opening combinations can be added without another top-level field.
- `reset_at` is a reset barrier. When present, local promotion, direct
  sync retry, cloud history load, and active-history resolution ignore matches
  with `saved_at <= reset_at`.
- `schema_version` is always `3`; increments require a writer + rules + reader
  update in the same slice.
- `match_history` is one embedded private-history container, not a set of
  independent record stores. The browser writes a merged snapshot, not one
  document per match.
- `match_history.replay_matches` stores the newest full replay payloads and is
  capped at 128 records.
- `match_history.summary_matches` stores the next private-history tier after a
  record ages out of the full replay window. These records keep summary metadata
  for stats/filtering without retaining every move list, newest first, capped at
  1024 records.
- `match_history.archived_stats` stores aggregate stats for records that age out
  of the summary tier. These three tiers are mutually exclusive retention
  stages for the same match history.
- Writer behavior: during local profile promotion, the browser only sends
  `display_name` if the local profile has a custom name and the loaded cloud name
  still matches the provider default. If the local display name is still the
  default `Guest`, the app adopts the provider/cloud display name locally
  instead of overwriting cloud state with `Guest`.
- Firestore rules enforce ownership, shape, write timestamps, and reset-barrier
  movement for profile writes. The browser can leave `reset_at`
  unchanged or set it to the current write's `request.time`; it cannot move the
  barrier backward, remove it, or set an arbitrary timestamp.
- Firestore rules also enforce a 5-minute cooldown between normal profile
  snapshot updates using `updated_at`. Profile changes and match-history
  changes remain local-first and coalesce into the next eligible sync checkpoint.
  Reset-barrier writes can bypass the normal edit cooldown only when they clear
  history and advance `reset_at` inside the constrained reset shape.
- Reset Profile while signed in writes `reset_at`, resets cloud profile
  display/default-rule fields to provider/default values, clears every
  `match_history` tier, and clears this device's local/cloud caches.

### Match History Tiers

`match_history.replay_matches` contains `SavedMatchV1` records. It is the only
cloud tier that can power replay playback directly.

`match_history.summary_matches` contains private, compact summary records
derived from saved matches after they age out of the replay tier. It exists so
stats and future filtering can survive longer than the full replay window
without keeping every move list forever.

```ts
type CloudMatchSummaryV1 = {
  id: string;
  schema_version: 1;
  match_kind: "local_vs_bot" | "local_pvp" | "online_pvp" | "puzzle_challenge";
  ruleset: "freestyle" | "renju";
  opening: "standard";
  side: "black" | "white";
  outcome: "win" | "loss" | "draw";
  opponent: {
    kind: "bot" | "human";
    display_name: string;
    profile_uid: string | null;
    bot_key: string | null;
  };
  move_count: number;
  saved_at: string;
  trust: "local_only" | "client_uploaded" | "server_verified";
};
```

`match_history.archived_stats` carries lifetime aggregates for records evicted
from the summary tier:

```ts
type CloudArchivedMatchStatsV1 = {
  schema_version: 1;
  archived_before: string | null;
  archived_count: number;
  totals: CloudMatchStatsCounter;
  by_ruleset: Record<"freestyle" | "renju", CloudMatchStatsCounter>;
  by_side: Record<"black" | "white", CloudMatchStatsCounter>;
  by_opponent_type: Record<"bot" | "human", CloudMatchStatsCounter>;
};

type CloudMatchStatsCounter = {
  matches: number;
  wins: number;
  losses: number;
  draws: number;
  moves: number;
};
```

## Private Match History

Current casual private history is embedded in
`profiles/{uid}.match_history.replay_matches`.
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

  source: "local_history" | "cloud_saved";
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
(correct for local-only records). For `cloud_saved` records,
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

The 128 full-replay cap plus 1024 summary cap leaves practical room under
Firestore's 1 MiB document limit. Current `SavedMatchV1` sizing estimates put
128 typical 60-move matches around 160 KiB and 128 full-board 225-move matches
around 330 KiB before future schema growth; the summary tier is intentionally
metadata-only so stats can retain a longer window without storing moves.

## Indexing Notes

`match_history`, `match_history.replay_matches`,
`match_history.summary_matches`, and `match_history.archived_stats` are not
queried directly and should not be indexed. The repo-level
`firestore.indexes.json` disables single-field indexes for those embedded
profile history fields.

## Local Profile History

Current local history lives in browser `localStorage` under
`gomoku2d.local-profile.v3`. The local profile uses the same replay, summary,
and archived-stats retention tiers as the cloud profile:

```ts
type LocalProfileV3 = {
  matchHistory: {
    replayMatches: LocalProfileSavedMatch[]; // newest first, capped at 128
    summaryMatches: CloudMatchSummaryV1[]; // next tier, capped at 1024
    archivedStats: CloudArchivedMatchStatsV1;
  };
  profile: LocalProfileIdentity | null;
  settings: LocalProfileSettings;
};
```

Finished local matches use `SavedMatchV1` with:

- `source: "local_history"`
- `trust: "local_only"`
- `id` equal to the browser-local match ID

```ts
type LocalProfileSavedMatch = SavedMatchV1 & {
  id: string;
  source: "local_history";
  trust: "local_only";
};
```

`gomoku2d.local-profile.v3` is a clean-break key. Older
`gomoku2d.guest-profile.*` keys are intentionally ignored instead of migrated;
this is acceptable during the alpha period because no real user data needs to be
preserved yet. If the new key is missing or malformed, the browser starts a new
local profile and empty local history.

The signed-in cloud-history cache lives under `gomoku2d.cloud-history.v3`.
It is a disposable per-user cache/pending-sync queue, not a canonical data
source. The v3 key intentionally drops older cached cloud-history state so old
pre-profile-snapshot records cannot reappear after the schema pivot.

Cloud profile sync turns canonical local replay records into embedded
`cloud_saved` records by changing `source`, `trust`, and the human player's
`profile_uid`. Local summary records are imported as `client_uploaded` summary
records when possible. Local archived stats are imported only when the target
cloud history is empty, because archived counters no longer carry per-match IDs
and cannot be safely deduped against an existing cloud archive.

## Migration Notes

The profile-v3 pivot stores casual cloud history directly in
`profiles/{uid}.match_history`, moves rule preferences into `settings`, moves
provider metadata into `auth.providers`, splits private history into
`replay_matches`, `summary_matches`, and `archived_stats` tiers, removes copied
email from app-owned profile data, and standardizes the profile reset barrier as
`reset_at`. This is a clean alpha break: the app no longer imports old v2
profile fields such as `recent_matches`, `preferred_variant`, or
`history_reset_at`. Existing alpha Firestore v2 profile documents were test
data and were cleared before deploying the matching v3 rules, rather than
carrying a long-lived client-side migration layer. Future schema versions should
either:

- keep readers backward-compatible for existing versions, or
- run a one-time migration and record it here.
