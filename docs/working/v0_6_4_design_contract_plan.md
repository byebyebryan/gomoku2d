# 0.6.4 design and contract pass

Owner: Bryan. Prepared November 17, 2026 against `f94dde6` on local `main`.
Status: Bryan approved the revised UI direction on November 18. The
[proposed replay collection contract](v0_6_4_replay_collection_contract.md)
records the design after feasibility checks and takes precedence over the
initial recommendations below.

The [roadmap](../reference/product/roadmap.md#064-replay-collection-and-study-tools)
owns release scope. The [follow-up research](v0_6_followup_plan.md) records the
earlier investigation. This pass turns that scope into a reviewable interaction
prototype and a compatible persistence contract before product implementation.

## Outcome and scope

A player can recognize a saved replay, give it a private title and short note,
save a moment to open next time, and return to it after reload. An owner can
change a published game's title/opening without removing the publication.

Keep the existing 64 private entries, 32 publications and four public storage
pages. Keep portable v1, fingerprints, participant attribution, ordinary My
games/statistics and public-profile consent independent of collection metadata.
Folders, per-move annotation lists, lessons, public notes/comments, ordering
tools and multiplayer remain outside this release.

## Observed starting point

These are source observations, not new contracts:

- Saved replays are text rows with Inspect/Remove and eight initially visible
  entries. Saving strips portable viewing ply and preserves game identity.
- The local schema-1 envelope accepts exactly four entry fields: `fingerprint`,
  `payload`, `kind`, `saved_at`. Its 600,000-byte budget covers **all cached
  owners together**, not 600,000 bytes per account.
- The cloud private library replaces a revisioned snapshot of at most 64 entries.
  Its JSON is restricted to 600,000 ASCII characters. Existing duplicate merge
  prefers one entry without a metadata review; that is insufficient for notes.
- The private cloud document is schema 1. Rules currently require the schema to
  stay equal on update, so an upgrade requires a deliberate rules change.
- Public entries already carry `title` and `view_ply`. The existing catalog
  transaction can update a page while preserving profile identity/statistics,
  page counts and generation. An ordinary mutation uses two transaction reads
  and two writes; SDK retries/rules-dependent reads are separate.
- Replay already has full-sidebar Share/Publish modes. Stored replays use the
  finished board by default. Public/shared replay move 0 is a real empty-board
  opening, not a signal to use the final board.

Relevant implementation: [library store](../../gomoku-web/src/replay/replay_library_store.ts),
[sync](../../gomoku-web/src/replay/replay_library_sync.ts),
[cloud serializer](../../gomoku-web/src/cloud/sharing_common.ts),
[catalog transaction](../../gomoku-web/src/cloud/sharing_public.ts),
[lifecycle cleanup](../../gomoku-web/src/cloud/sharing_lifecycle.ts),
[rules](../../firestore.rules), [library UI](../../gomoku-web/src/routes/ReplayLibraryPanel.tsx)
and [replay UI](../../gomoku-web/src/routes/ReplayRoute.tsx).

## Pass 1: interaction and visual design

Prepare one recommended composition, with responsive variants where needed.
Compare alternatives only for a concrete unresolved question.

| Surface | Recommended direction | Review question |
|---|---|---|
| Saved replays | Compact thumbnail row; private title or existing player-label fallback, rules/move count and saved date; clear Replay/Remove actions | Can eight rows remain easy to scan on desktop and phone without becoming a card gallery? |
| Private replay details | A full-sidebar mode using existing typography, fields, icons and button treatments; private title, note and opening choice; explicit Save/Cancel | Is the editor reachable without crowding the normal replay controls? |
| Public game list | Reuse the static thumbnail treatment beside the existing public title/game information | Does the two-column profile remain balanced with stats on/off and sparse/full lists? |
| Existing publication | Edit action in the owner's publication list opens the replay in public-edit context; full-sidebar title/opening editor | Are public edits clearly distinct from private details and first-time Publish? |
| Conflict review | Show local/cloud private details for each differing replay, with an explicit version choice | Can someone understand what will be kept before committing a merge? |

Keep the board visible while editing. Stage the opening with a "Use current
move" action; it changes the draft only. Show the staged move clearly and
provide "Use default" for private entries. Save commits the staged fields;
Cancel leaves stored data unchanged. Pausing, scrubbing and inspecting never
save. A new saved copy starts with blank private fields and no saved opening;
opening a public link does not import its title or opening into private metadata.

Prototype unsaved-change handling for closing the editor, route navigation and
Play From Here. Account changes cancel ownership-bound drafts and prevent
in-flight results from entering the new account. Returning from an editor
restores the trigger's focus and sidebar/list position where applicable.

Use concise field labels and state feedback. Avoid long explanatory blocks,
checkboxes, a second replay shell or new icon variants unless existing assets
cannot express the action. Keep Previous/Next positions and originating public
profile return behavior intact. Opening details must not load private account
data merely because a visitor views someone else's profile.

Required prototype states:

- Empty, one-entry and full eight-row lists; a 64-entry library for density.
- No title/note/opening, edited details, move 0, a middle move and final move.
- Long Unicode text, multiline note and over-limit input feedback.
- Guest, account synced, pending/offline, storage unavailable and conflict.
- Existing public edit, stale publication, disabled profile and removed entry.
- Desktop/portrait, 320px reflow, enlarged text and keyboard focus. Generic
  browser viewports remain distinct from actual phone/touch acceptance.

## Pass 2: metadata and identity contract

Recommended private schema-2 entry: retain the four existing fields and add one
required `metadata` object with exactly `title`, `note`, `opening_ply`. Legacy
entries map to empty strings and `null` in the new reader. Do not add a metadata
timestamp as a conflict winner or change `saved_at` when editing.

| Proposed field | Candidate contract to validate |
|---|---|
| `title` | Plain text; empty means player-label fallback. Start with 80 Unicode code points / 320 UTF-8 bytes, matching public-title bounds. |
| `note` | One private plain-text note, with line breaks; start with 280 code points / 1,120 UTF-8 bytes. No markup rendering. |
| `opening_ply` | `null` means ordinary viewer default, currently the finished position. Otherwise an integer from 0 through the validated game's move count. |

The text limits are candidates until serialization/budget measurements pass.
Specify trimming, newline normalization, allowed control characters and exact
code-point/byte counting once; an input's UTF-16 `maxLength` is not that contract.
Clearing individual fields is a draft edit committed by Save; Remove deletes
the whole private entry. Re-saving the same fingerprint preserves its metadata.

Private metadata stays outside portable payloads and identity. Saving it changes
neither attribution nor statistics and never changes an existing publication.
Share retains its explicit selected-move behavior without adding private text.
Publication edits use only public title/opening; no private-field auto-fill.

The public schema can remain unchanged: the existing `view_ply` always stores
an explicit integer, including 0. "Finished position" resolves to move count.
Changing an existing publication preserves its fingerprint, payload, physical
page, row order/count, generation and unrelated entries/statistics. Retain
current title bounds and decide whether empty public titles remain accepted
data but disallowed by the editor, as in first-time Publish today.

Define opening precedence by source: a private saved route uses its private
opening; a public catalog link uses that entry's public opening; an ordinary
shared link uses its encoded viewing ply or existing default. Opening a shared
link must not apply a matching private entry's opening behind the user's back.

## Pass 3: compatibility, sync and lifecycle contract

Prefer a one-way private document upgrade at the existing path, without a second
metadata collection. Give private-library schema constants their own identity;
do not bump the shared constant used by public documents. The contract must
define all of the following before adopting this approach:

1. New readers accept v1 and v2. Opening or inspecting does not migrate cloud
   data. The first explicit save/clear can create or upgrade the private marker
   to v2 in a revision-checked transaction.
2. Rules permit v1 to v2 and v2 to v2, preserve compatible v1 writes before
   migration, and reject v2 to v1. A retained empty v2 marker also stays v2.
   Old code must not overwrite metadata even through clear/reset/delete.
3. Older deployed clients may show an existing unsupported-schema/rules error;
   they cannot be retrofitted with a new message. The new client gives a useful
   update-required state for a future unsupported schema, preserves raw data
   and never substitutes an empty library that can sync over it.
4. Evaluate upgrading the envelope at the existing local key against a separate
   v2 key. Prefer one authoritative envelope if old-tab write guards can be
   proved to preserve it. Specify interrupted migration, malformed input,
   session-only changes and recovery; do not leave two independently writable
   libraries that can later restore cleared data.
5. Choose an ASCII-safe Unicode encoding and measure its worst-case expansion.
   Cover JSON escaping, emoji, maximum payloads, all 64 entries, all cached
   owners and the two different local/cloud budgets. Keep the cloud snapshot
   bound unless separately reviewed. Reject oversized saves without truncating
   notes, removing records or replacing the previous persisted snapshot.
6. Compare both metadata and replay content during merge/import. Identical
   copies can coalesce. For differing metadata, recommend a reviewed choice of
   the whole metadata object per replay rather than automatic field stitching
   or a timestamp winner. Preserve payload/fingerprint/kind validation; a
   payload mismatch remains an error. Require choices before writing.
7. Review local-only/cloud-only inclusion and removal resurrection in the final
   merge summary. A title-only edit on one device and note-only edit on another
   still needs review under the whole-metadata choice. Concurrent changes after
   review invalidate the reviewed snapshot; no stale overwrite on retry.
8. Guest-to-account copy uses the same duplicate/conflict rules. Owner changes,
   offline edits, storage failure and saves completing during reload/merge
   retain existing account/request/lifecycle guards.
9. Removing a private entry removes its metadata only. Cloud clear/reset/delete
   clears notes with entries and retains revision/schema barriers. Define local
   pending recovery separately from cloud deletion; stale tabs cannot silently
   repopulate a cleared account. Public unpublish/disable leaves private copies
   independent. Existing partial-cleanup failures remain retryable and honest.

## Pass 4: thumbnail feasibility

Prototype a small Canvas 2D renderer using the accepted board palette and stone
assets, with a full 15x15 board, fixed orientation and exact prefix at the
selected opening. Use private `opening_ply` or final position, and public
`view_ply` including 0. Do not draw analysis overlays, hover/next-move markers
or animations. Keep the adjacent Replay action usable if the thumbnail fails.

Check whether a small shared projection/asset descriptor can serve both board
and thumbnail without importing Phaser into list routes. Share constants and
projection logic where appropriate; add no second legality engine. Establish
the validation boundary, cache identity and invalidation for payload/opening/
asset changes. Malformed or not-yet-validated records get an honest fallback,
not an apparently verified board. Reuse validated work without running replay
analysis or creating one engine/Phaser scene per row.

Compare integer-aligned sizes on desktop/mobile and DPR variants; verify stone
colour, grid edges and last-move visibility without relying on WebGL tinting.
Measure eight visible rows and expansion to 64 before choosing cache/lazy-draw
complexity. Opening lists/thumbnails adds no cloud documents, reads, writes,
listeners or image uploads beyond the current list load. Explicit saves retain
bounded transactions; no keystroke, scrub or per-move sync.

## Review outputs and next checkpoint

Work in this order: resolve the compatibility/budget questions on paper, prepare
the main UI composition, run only the bounded thumbnail/migration feasibility
experiments needed to decide, then review UI and contract together. Prototype
code must stay separate from production persistence changes.

The checkpoint consists of:

- A focused desktop/mobile prototype with private/public editor and conflict
  states, plus proposed labels/actions and keyboard/unsaved-draft behavior.
- An exact v2 schema/serialization example, field bounds, source/opening
  precedence and migration/rollback sequence. Record that an older deployment
  cannot safely resume private-library writes after v2 migration; do not propose
  downgrading data to make rollback work.
- A compatibility table for new/old clients and v1/v2 local/cloud data, including
  stale tabs, missing/empty markers, guest import and reset/delete races.
- A thumbnail sample and measured size/performance/cloud-operation evidence.
- A small decision ledger identifying recommendations accepted by Bryan and
  unresolved points. Use the [UI review procedure](ui_screenshot_review.md)
  and developer review tools for presentation; keep private fixtures out of
  public prepared records.

Before product implementation, settle the composition, editor entry points,
text limits/encoding, local migration, cloud one-way upgrade, duplicate conflict
choices and cleanup behavior. Then prepare the implementation sequence and
checks: codec/store/sync and emulator rules tests first; browser flows and
responsive/Canvas visual checks with local guest/disposable-account fixtures.
Real account, physical phone and human accessibility checks remain separate.
No product code, cloud rules deployment or release publication is part of this
planning pass.
