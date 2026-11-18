# Gomoku2D 0.6.4 replay collections

Proposed contract for review, November 18, 2026. Source baseline `f94dde6`.
The [design plan](v0_6_4_design_contract_plan.md) records the checkpoint scope.

## Player flows

- **Saved replays.** Keep eight rows initially. Add a static board, a title or
  player-name fallback, game information, opening move and Replay action.
  Remove clears this copy and its details.
- **Edit details.** Open a saved copy in the existing Replay Analysis view.
  Add **Edit details** beside the replay's Share action. It swaps in a title,
  one note and opening controls using the Share/Publish sidebar pattern.
  Use the existing timeline and step controls in the editor to choose a
  position, select **Use move N**, then save. **Use default** stages a return
  to the finished position. Saving keeps the on-screen move and changes where
  the replay starts next time. Prompt before leaving a changed draft. Account
  changes close the old owner's editor and discard its draft.
- **Edit publication.** Enter from Public settings → Published games. Open
  the same Replay Analysis view in its public edit sidebar. Save returns to
  the originating list row. A conflict preserves the draft while asking the
  owner to review the current publication. A removed or disabled publication
  cannot be recreated by an edit.
- **Review changes.** Compare both versions of each differing replay's private
  details. Choose a complete set per replay and confirm the resulting merge.
  Keep the explicit option to adopt the cloud library, with confirmation
  about pending local changes.

## Data and bounds

The private v2 entry retains exactly `fingerprint`, `payload`, `kind`,
`saved_at` and adds the required `metadata` object. The object has exactly
`title`, `note`, and `opening_ply`. A v1 entry maps to
`{ "title": "", "note": "", "opening_ply": null }`. Preserve `saved_at`,
participant snapshots, rules, moves and fingerprint on a details edit. New
copies start with default details. Saving a duplicate retains its details.
Keep legacy `practice` entries readable with default details through My
games. Edit details applies to `saved` entries.

| Field | Logical value |
|---|---|
| `title` | Trimmed, single-line plain text. Empty displays player names. Maximum 80 Unicode scalar values and 320 UTF-8 bytes. |
| `note` | Trimmed, multiline plain text. Normalize CRLF and CR to LF before trim and validation. Maximum 280 Unicode scalar values and 1,120 UTF-8 bytes. |
| `opening_ply` | `null` for the ordinary default (finished board), or integer 0 through the validated move count. Move 0 shows the empty board. |

Trim using JavaScript string trim. Reject lone surrogates, C0/C1 controls and
U+2028/U+2029 in stored text, allowing LF in the note. Keep internal whitespace
and original Unicode normalization. Render as text and retain note line breaks.
Reject malformed or over-limit values before durable writes. Editor counters
use normalized code points.

Private storage leaves `viewPly` out of the portable payload. On the private
route use `opening_ply ?? moveCount` when opening. Share can include the current
move through its existing explicit control. It carries game data and the
selected viewing ply. Private title, note and stored opening are never
implicitly copied into a shared link, publication or played-game data.

The Firestore document stays at `replay_libraries/{uid}` with exactly
`schema_version`, `uid`, `revision`, `updated_at`, `entries_json`.
For v2, the compact JSON array encodes the two strings as canonical unpadded
base64url of their UTF-8 bytes. Decode strictly, validate text limits on the
decoded fields and reject unknown fields. Empty strings encode as empty
strings. For example, “Fin ✨”, “Reply here.\nThen review.” and move 0 become
`{"title":"RmluIOKcqA","note":"UmVwbHkgaGVyZS4KVGhlbiByZXZpZXcu","opening_ply":0}`.
Empty snapshots remain exactly `[]`. Firestore rules bound the opaque ASCII
snapshot to 600,000 characters. Client codecs validate the shape and contents.

The local v2 envelope stores raw Unicode. Its 600,000-byte UTF-8 budget covers
guest and all cached accounts. Each owner can have up to 64 entries. Enforce
both size budgets before a store mutation, merge or import. Reject oversized
operations with a visible reason and preserve the prior snapshot and draft.

| 64-entry JSON array, bytes | At the 8,192-character token ceiling | At a conservative bound for valid v1 payloads (1,808 characters) |
|---|---:|---:|
| v1 entries | 532,673 | 124,097 |
| v2 with raw UTF-8 | 628,225 | 219,649 |
| v2 with JSON Unicode escapes | 812,545 | 403,969 |
| v2 with base64url UTF-8 fields | 659,009 | 250,433 |

The first column uses synthetic tokens. Canonical v1 restricts objects, names,
UIDs, bot configuration and move bytes. The second column bounds those fields
with room for the longest current participant configuration and strips viewing
ply as the library does. Both columns use 80 and 280 four-byte characters and
the longer `practice` kind as an upper bound. One complete local owner uses
219,912 bytes with the proposed envelope. A full guest and account use
439,643 bytes. A third full bucket would reach 659,502 bytes, so combined-owner
budget failures need their own scenario. Retain the portable 8,192-character
parse limit and preflight aggregate sizes based on validated records.

## Local and cloud compatibility

Use `gomoku2d.replay-library.v2` as the authoritative local cache after the
first durable mutation. The old key is `gomoku2d.replay-library.v1`. A separate
key isolates v2 data from a stale v1 writer that passes the old raw-value
check immediately before cutover. The v2 cache has exactly
`schema_version: 2`, `revision` (positive safe integer), and `owners`.
Each namespaced owner has exactly `entries`, `dirty`, `base_revision`, and
`restore_review`. The last two record the revision behind pending work
and whether an explicit restore is required after cloud deletion.
`base_revision` is a positive cloud revision, `null` for an observed missing
document, or `"unknown"` for a legacy/offline bucket with no verified base.
Guest buckets use `"unknown"` and `restore_review: false`.

When v2 is absent, use the v1 cache as the import source. At the first durable
mutation validate and copy every v1 owner, including dirty state. Persist the
complete v2 envelope before cleaning up the observed v1 bytes. Keep a valid
empty v2 envelope after local reset. A later v1 write can only change retired
bytes. New tabs prefer the v2 envelope even if it is empty and preserve
unreadable/future data verbatim. Treat a failed cutover as a failed durable
save. Old tabs with pre-cutover local changes should refresh before continuing.
Recovery of edits made using the retired cache after cutover is outside this
migration.

Serialize writes among v2 tabs with an exclusive origin-scoped Web Lock.
Re-read the envelope inside the lock and check the expected revision before
committing. The lock covers the brief local read and write. In an environment
without a working lock or durable storage, offer clearly labelled session-only
changes. An oversized envelope or revision conflict is a rejected edit with
the draft preserved. A storage fallback cannot become an unreviewed cloud
replacement. Report local persistence and cloud operations independently.

| Combination | Proposed behavior |
|---|---|
| New client + local v1 only | Read v1 with default details. First durable mutation commits v2 atomically and retires the observed legacy bytes. |
| Old client + local v2 | Uses its v1 cache/key if present. Writes cannot touch the authoritative v2 key. It may show old or empty local data until refreshed. |
| New client + cloud v1 or missing | Read without migrating. First explicit sync or clear writes v2 at the next revision, or revision 1 for a missing marker. |
| Old client + cloud v1 | Existing reads and revision-checked writes keep working until upgraded. |
| Old client + cloud v2 | Existing parser fails. Rules also reject v1 writes, including clear at the correct next revision and writes against an empty v2 marker. |
| New client + cloud v2 | Read and write at the same path. Owner, size and revision checks still apply. |
| New client + unsupported version or unreadable snapshot | Preserve bytes and show update-required or recovery state. Block mutation and sync from an unverified base. |

Deploy additive rules support before enabling v2 writers. Allow creation of v1
or v2 at revision 1 and only v1→v1, v1→v2 and v2→v2 with revision +1.
Reuse the owner checks, server timestamp, read restrictions, size constraints
and canonical empty marker. Use a private-library schema version constant.
Keep these rules through UI rollback. Once v2 data exists, a rollback must
retain v2 reading and cleanup or temporarily hold private cloud mutations
until a compatible build is restored. Down-converting existing data is lossy.

## Sync, merge and cleanup

- Save locally first. For accounts with a reviewed base, one explicit save
  can use bounded sync. Offline edits retain their base revision. A pending
  bucket whose base is unknown or differs from the loaded cloud revision
  requires review. Viewing and scrubbing produce zero writes. A failed save
  preserves the draft or saved local changes and reports the actual state.
- Verify incoming payloads, kind and fingerprint through the existing portable
  validator before reviewing or writing a merge. A matching identity, kind
  and metadata coalesces to one copy. A differing title, note or opening
  requires choosing one whole metadata object, even if devices edited
  different fields. On duplicate identity choose the earlier save time.
  A payload or kind mismatch blocks merge/import with an explanation and
  leaves both inputs recoverable. Legacy practice stays outside the editor.
- Prepare merges and guest imports in memory and preflight count, both size
  limits and the local revision. Confirm any
  local-only or cloud-only additions, including copies removed on one side.
  Guest import preserves its source. A review binds the selected owner, local
  revision, cloud revision and source snapshots. Wait for the revision-checked
  cloud transaction before replacing the device copy in one local mutation.
  Repeat review if the cloud rejects the revision. If local work changed
  while the cloud write was in flight, preserve it and review against the
  confirmed cloud snapshot before syncing again. Ordinary saves still keep
  local edits available offline.
- Remove clears a private entry and its metadata. Guest reset clears the guest
  bucket. Signed-in reset clears the account's local and cloud library and
  existing private profile data, leaving public data alone. Delete Cloud
  clears account sharing and private entries, retains empty revision markers,
  deletes the cloud profile and signs out. Preserve this device's account
  copies under the existing policy and durably mark them for review before
  cloud deletion can replace their base. Stale pending copies require an
  explicit restore before re-upload. Revision checks guard in-flight work
  on other devices. Treat a newer empty cloud snapshot against a nonempty
  local account cache as a cleanup boundary. Show the user any local copies
  still available before discarding or explicitly restoring them. This also
  protects copies when the local restore flag could not be committed. Report
  device storage errors separately so they do not prevent cloud deletion.

## Public edits and board previews

Keep the public catalog schema, physical page and row position. Existing public
title bounds remain 80 code points and 320 bytes. The owner editor requires a
nonempty title to save. The current public route uses the payload's
viewing ply, and the catalog separately carries `view_ply`. For legacy rows
use the verified payload's `viewPly ?? moveCount` so thumbnails and public
Replay agree. A changed opening writes a matching `view_ply` and canonical
portable `at`. That is the only payload change. Game identity, attribution and
fingerprint stay stable, and previously copied URLs still open their original
position. A title-only edit preserves payload bytes and current opening. Keep
root revision, page revision and generation checks so editing cannot restore a
removed game. Preserve unrelated rows, counts, visibility and statistics. A
typical catalog update uses two transaction reads and two writes before retries.

Project the exact validated move prefix on a static 15×15 Canvas 2D board.
Use the existing palette and stone frame 0 with the app's Black/White RGB
multiplication. Soften the grid at thumbnail sizes so dark stones stand out.
Use integer cell steps and cap backing density at DPR 3. The prototype uses
90 CSS px on desktop and 75 on narrow screens. Keep the board decorative
beside text and Replay. Use a neutral fallback for invalid or unavailable
data or assets. Share validated projections and draw from them without loading
Phaser or the analyzer into list routes for thumbnails. Key drawing work by
validated content, effective move, asset version and size. Normal rendering
uses zero additional cloud requests and uploads no images.

## Review and implementation order

Bryan approved the revised UI direction, including reuse of the existing
Replay Analysis view. The text limits, local cutover and public opening mirror
are proposed contract decisions. The implementation sequence is:

1. Private v2 codecs, local cutover and bounds, plus rules and cleanup tests.
2. Revision-bound sync, merge, guest import and account/reset/delete behavior.
3. Saved/public rows, editors and public revision-aware edit transaction.
4. Regression and visual review of real routes with emulated accounts and
   three browser engines. Include account switching, two tabs, offline
   recovery, denied storage, 64 saved entries and 32 publications.

The [design review receipt](v0_6_4_replay_collection_review.md) records the
prototype fixtures and validation.
Physical phone, real-account and human accessibility checks remain open.
