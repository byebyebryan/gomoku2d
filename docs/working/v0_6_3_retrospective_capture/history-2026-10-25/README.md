# UI history: 0.2 through 0.6

Qualified retrospective references for 25 actual release sources: 24 UI
milestones, 15 screen families, 89 screen changes and 137 snapshots with 274
desktop/portrait PNGs. The [catalog](../../../../gomoku-web/src/developer/history.json)
is the public allowlist; the source inventory and change matrix are planning
evidence, not substitutes for these captures.

The viewer has a milestone overview and individual screen trails. Updates pair
neighbouring relevant versions. Introductions and retirements show one real
capture. Bot Report and Analysis Report merge into Lab; Visuals retains its
identity through the static-to-React change. 0.5.0 is a source/report baseline
without a separate UI milestone. There is no 0.6.0 tag.

## Provenance

`runs/` preserves complete original manifests and PNG bytes. Snapshot IDs bind
the selected manifest; public PNG filenames include their content digest.
Earlier successful runs remain archived when a later framing capture is selected.
`diagnostics/` retains failed attempts separately; they are not published by the
catalog. No screenshot was cropped, recoloured, stretched or repainted.

`builds/` records the exact peeled commits, source trees, dependency locks,
Wasm/dist/file identities, actual environment and tag-specific build commands.
The first three sources have no tracked Cargo lock; their real resolver outputs
are retained as `resolved-Cargo.lock`. Actual build tools were Node 26.12.1,
npm 12.2.1, Rust/Cargo 1.99.1 and wasm-pack 0.14.0. Modern Chromium 156 was used
for capture. This recreates tagged layouts; it does not reproduce the historical
browser, compiler or device runtime. Animation phase is not fixed.

Desktop is 1280×900 CSS pixels. Portrait is generic Android touch emulation at
412×800, DPR 1, en-US/UTC, reduced motion. It is not a measured OnePlus profile.
The earlier 390×844 reference and endpoint pilot remain unchanged.

## Fixtures and framing

Game references use a synthetic Reference Player and the tagged default bot,
with the same legal nine-move Freestyle finish. The first Match source has no
branch action and shows a new board; later Match views branch at ply 8. Replay
shows the final board and settled analysis where available. Reports use their
actual tagged recorded datasets, which can differ between releases; tournaments
were not rerun for screenshots.

Cloud-era views use isolated demo Auth/Firestore services and tagged security
rules. Older SDKs use the owned loopback proxy; newer tags use their native
emulator hooks. Actual SDK responses are used. Public fixtures contain all four
catalog page documents, including valid empty pages. The external Google popup
sign-in flow is outside this capture acceptance.

The unused hosted Google popup bootstrap stays blocked, with exact request and
URL-attributed diagnostics retained. A separate cold-session
[stream probe](transport-stream-proof.json) observed local SDK streams returning
HTTP 200 before cancellation. Only the two exact approved native Firestore
Listen/Write GET/fetch paths with prior HTTP 200 and `ERR_ABORTED` receive that
separate classification; other failures remain fatal. Final selected references
have no unexpected network, console or page errors.

Visual review caught states whose DOM was ready while the useful content was
below the viewport. Targeted captures now frame the actual proof, Custom controls,
Guide examples and expanded Share/Publish panes. Natural fixed headers, nested
scroll regions and end clamping remain intact. Legacy proof-document overflow
is preserved and recorded rather than repaired in historical source.

## Acceptance and access

[Qualification](qualification.json) records the selected graph;
[validation](validation.json) records checks, delivery and cleanup. The
[implementation record](../../v0_6_3_history_implementation.md) owns decisions
and execution. The previous capture-batch qualification is preserved separately.

Open the managed Starship preview at
`http://starship:46862/dev/reviews/history/`. The existing Carbon local forward
uses `http://localhost:46862/dev/reviews/history/` when Carbon is online. Its
final-build byte recheck is pending because Carbon went offline. This is local
implementation evidence, not a push, release or production deployment.

Primary visual inspection and automated checks do not establish Bryan's design
approval, screen-reader acceptance or physical-phone calibration. Automated
contrast checks also retain incomplete results for manual judgment in the
320px/200%-text stress view.
