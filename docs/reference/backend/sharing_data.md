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

The owner can fetch their own document while it is disabled. A visitor can
fetch a known UID only while public sharing is enabled globally and that
profile is enabled. Directory queries are denied. The four fixed documents
`public_profiles/{uid}/pages/{0,1,2,3}` each hold at most eight entries, for a
maximum of 32 publications. A visitor fetches the identity and only the page
they open; pages are fetched directly by ID, with no listener.

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
An enabled-name change increments only the root revision. A normal publication
write increments only its page revision and requires both the loaded page
revision and profile generation to remain current. Empty page documents are
retained; pages and profiles cannot be deleted.

The `publicSharingEnabled()` Firestore rule helper is the operational switch.
Changing its literal result to `false` blocks visitor reads and new enabling or
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

## Client API and operational behavior

`src/cloud/sharing_public.ts` exports `loadPublicProfile`, `loadCatalogPage`,
`setPublicProfile`, and `saveCatalogPage`. `src/cloud/sharing_private.ts`
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
