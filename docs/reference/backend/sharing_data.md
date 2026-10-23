# Sharing and replay library data

This document describes the bounded Firestore projection used by the public
player page and the private replay library. It is separate from the existing
schema-5 `profiles/{uid}` record and its played-match history.

## Public profile and catalog

`public_profiles/{uid}` is the public identity document:

| Field | Contract |
|---|---|
| `schema_version` | `1` |
| `uid` | Owning Firebase UID |
| `display_name` | Enabled: 1–80 Unicode characters and at most 320 UTF-8 bytes. Disabled: empty string. |
| `enabled` | Public page visibility |
| `generation` | Positive monotonic integer, at most 2,147,483,647 |
| `revision` | Positive monotonic integer, at most 2,147,483,647 |
| `updated_at` | Firestore server timestamp |
| `shared_stats` | Optional/null means off. Otherwise exactly `matches`, `wins`, `losses`, `draws`, and `updated_at`; each counter is a nonnegative integer up to 2,147,483,647, `matches` equals the three outcomes combined, and the nested update time is a server timestamp. |
| `catalog_page_counts` | Optional four-entry manifest in storage-page order. Each count is an integer from 0 through 8 and must match the corresponding page's entry count. |

New roots write `shared_stats: null` and `catalog_page_counts: [0, 0, 0, 0]`.
Readers accept legacy roots with only the original required fields and current
roots with either or both known optional fields; other fields are rejected.
Legacy roots without a catalog manifest remain readable. Their first catalog
mutation reads the four page markers, validates their generations, and writes
derived counts with the changed page. It performs no separate migration write.
A mismatch between a manifest and its page is an error that requires reload or
repair, not an empty catalog.

The owner can fetch their own document while it is disabled. A visitor can
fetch a known UID only while public sharing is enabled globally and that
profile is enabled. Directory queries are denied. The four fixed documents
`public_profiles/{uid}/pages/{0,1,2,3}` each hold at most eight entries, for a
maximum of 32 publications. Visitors use the manifest to fill logical pages
across empty storage pages and fetch only physical pages needed for the
selected logical page. Legacy roots without a manifest use one bounded read of
all four pages to derive counts. Pages are fetched directly by ID, with no
listener.

Each page entry has exactly `fingerprint`, `payload`, `title`, and `view_ply`.
Fingerprints are 43 base64url characters. Payloads are unpadded base64url and
at most 8,192 ASCII characters. Titles are at most 80 Unicode characters and
320 UTF-8 bytes. `view_ply` is an integer from 0 through 225. The client API
rejects duplicate fingerprints within one page; Firestore rules enforce entry
shape and count, but do not check uniqueness. The portable replay decoder owns
the legality and format checks for payload contents.

Create, visibility changes, and explicit clear operations write the profile
and all four empty page markers atomically. A visibility transition increments
the generation and each page revision. Re-enabling starts with empty pages.
An enabled-name change increments only the root revision and preserves the raw
statistics snapshot, including its exact Firestore timestamp. Catalog mutations
compare the expected root revision, page revision, and generation, then
atomically update both the page and root manifest/revision while preserving
name and statistics. They return both updated snapshots for owner state.
Ordinary catalog updates use two transaction reads and two writes. Initializing
a legacy manifest reads at most five documents (root plus four pages) and
writes two. The rules validate manifest/page agreement, prevent removal of an
existing manifest, and reject page-only writes that would desynchronize it.
Empty page documents are retained; pages and profiles cannot be deleted.

Statistics sharing is a separate explicit owner action. `setPublicProfileStats`
compares the expected root revision, advances it, and writes one root document
with a server timestamp; it does not change catalog generation or pages. A
null snapshot opts out and removes public counters. Name and catalog writes
preserve the existing snapshot and update date. Disabling or clearing public
sharing nulls statistics and resets page counts. Deleting the legacy profile
is blocked while a disabled public root still contains statistics.

The `publicSharingEnabled()` Firestore rule helper is the operational switch.
Changing its literal result to `false` blocks visitor reads, new statistics
snapshots, and new enabling or
nonempty publication writes. Owners can still disable and clear their data.

## Private library

`replay_libraries/{uid}` is owner-readable only and has exactly:

| Field | Contract |
|---|---|
| `schema_version` | `1` |
| `uid` | Owning Firebase UID |
| `revision` | Positive monotonic integer, at most 2,147,483,647 |
| `updated_at` | Firestore server timestamp |
| `entries_json` | ASCII JSON string, at most 600,000 characters |

The JSON snapshot is an array capped at 64 records. Each record has exactly
`fingerprint`, `payload`, `kind` (`saved` or `practice`), and `saved_at` (a
canonical ISO timestamp). The Firestore rule bounds the opaque snapshot bytes;
the client decoder validates its JSON and record schema. The document is not
created by a read. A save compares the revision loaded by the caller, replaces
the full snapshot, and increments the revision. Removal writes an empty array
and retains the marker. Concurrent/offline changes produce a reload-and-review
conflict instead of a merge or last-write-wins restore.

New saves use `kind: saved`. The `practice` kind remains readable for completions
created by early v0.6 previews; these appear in the owner's My games and displayed
statistics. New Play From Here completions use regular played-game history and
its existing sync instead of adding private-library entries.

## Client API and operational behavior

`src/cloud/sharing_public.ts` exports `loadPublicProfile`, `loadCatalogPage`,
`setPublicProfile`, `setPublicProfileStats`, and `saveCatalogPage`.
`saveCatalogPage` returns `{ profile, page }` so callers can refresh both root
and page revisions. `src/cloud/sharing_private.ts`
exports `loadPrivateLibrary` and `savePrivateLibrary`. Both accept an optional
injected Firestore instance for tests and use the configured Firebase client
otherwise. Loads use explicit server reads; mutations use transactions and do
all reads before writes. `src/cloud/sharing_lifecycle.ts` exports
`clearSharingData`, which atomically empties all four public pages and the
private library while retaining the public/private revision barriers. A
concurrent write can make the clear's stale revision fail Firestore rules
before the SDK retries its transaction; that failure leaves the concurrent
write and all other documents intact, and the caller can reload and review
before trying again. If a transaction retry does occur, clear also compares
all six documents with its initial read set and aborts on any difference.

No sharing operation is part of gameplay, replay scrubbing, or analysis.
`firestore.indexes.json` exempts the bounded payload fields from automatic
indexing because the application uses direct document reads only.

## Older-client deletion barrier

Deleting `profiles/{uid}` also requires the public identity to be absent or
cleared/disabled, and the private library to be absent or the canonical empty
snapshot. An older client cannot delete only the legacy profile while leaving
new public/private sharing data, then report cloud deletion complete. It receives
a rules error and must reload the current cleanup flow. A never-sharing account
retains its existing delete behavior. The current UI clears sharing first;
partial failures remain retryable. Up to two dependent reads can be billed for
this deletion guard. Older schema-5 writes otherwise remain unchanged.

The checked-in index configuration exempts public `shared_stats` and
`catalog_page_counts` from automatic indexing: both are read by known document
UID and are never queried. Applying these exemptions to production remains a
separate scoped deployment; local source/emulator validation does not prove
their deployed state.
