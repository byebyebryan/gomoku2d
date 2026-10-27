# Design-review evidence

This directory supports the [0.6.3 review backfill](../v0_6_3_design_reviews.md).
The primary selects cases and authors their copy and presentation. These files
retain factual context and media qualification for agents and maintainers;
the prepared human view is the developer site's Design reviews.

## Sources

`evidence_audit.json` binds the six cases to exact Git documents, text excerpts
or JSON pointers. A document revision and the release it describes are separate
fields. The audit also identifies 40 missing historical screenshot references
and their recoverable Git blobs. Only the four Replay images needed by the
selected cases were restored.

`media_manifest.json` records nine selected images:

- Four original 0.4.6/0.4.7 Replay PNGs, recovered byte-for-byte from Git. Their
  review captions identify last-escape frames. Original browser, DPR, runtime
  build and capture dates remain unrecorded.
- Five new renders of the original public-profile studies: A, B, C, the minimal
  two-column refinement and the player-card refinement. They are full-page
  mockups at 1280px width with fictional data, not application captures. Source
  report IDs, file inventories, supported selection state, browser, image hashes
  and unchanged-source checks are retained.

The other case images reuse already-qualified retrospective captures through
their existing paths. The old public-player implementation comparison keeps its
original media and `responsive` ID. B appears twice in the new option comparisons
as a common reference, while its source image is stored once.

## Frozen capture recipe

`capture_mockups.mjs` is the archived capture recipe. It requires the identified
frozen report outputs and their existing receipts, rather than rebuilding a
study from today's styles. It refuses to overwrite the selected images or
manifest. A later capture needs a new evidence directory and qualification.

The capture ran headlessly against the shared static report host. No user tabs,
real accounts or cloud services were used. The images and their metadata remain
usable independently of the authoring exports or report server.

## Public records

`gomoku-web/src/developer/reviews.json` owns the selected case copy, process
sequence, comparison IDs and allowlisted media. Runtime and build validation
check those references and image identities. This entire evidence directory is
not automatically exported: only explicitly selected media and typed records
enter the developer preview.
