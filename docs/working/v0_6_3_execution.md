# 0.6.3 implementation execution

Bryan accepted the [visual study](v0_6_3_design/README.md) and authorized full
implementation on October 24, 2026. Scope follows the
[brief](v0_6_3_developer_site_plan.md), checkpoints C–F. Commits stay local;
release/deployment remain separate. The 0.6.2 manual preview, emulators, Carbon
tunnel and demo fixtures remain owned by the existing services.

The [October 26 overall review](v0_6_3_overall_review.md) records fresh results
at `d5a602d` and five open follow-ups. The checkpoints below retain their dated
scope and evidence; they do not supersede that later review.

## Integration checkpoint C–D

The root entry now selects a lazy public developer tree or the player tree.
Fresh developer entries import no private controllers/stores. Leaving the player
unsubscribes its UI controllers without clearing singleton stores or cancelling
in-flight promises; returning re-evaluates pending synchronization. Public
profiles/shared replays keep their existing authentication-aware player boundary.

One developer frame owns scrolling, navigation, skip/focus and public recovery.
Overview and Process follow the approved study. Lab retains its recorded report
loaders, proof-board renderer and drill-down. Secondary ranking metrics move into
expanded rows; report JSON is unchanged. Report loads now survive local-view
changes rather than being cancelled into a permanently loading state.

Assets use canonical SVGs and actual CSS values. Full icon inspection is URL
based. Sprite previews reuse the pure RGB multiplication operation and runtime
black/white/overlay values, with one selected loop, stepping, reduced motion,
visibility and offscreen controls. Historical asset-only teal remains a named
preview colour, not a claim about the game renderer.

Those follow-on items were completed in the checkpoints below. Publication
remains separate from local implementation and validation.

## Curated records, release reference and export

The first implementation checkpoint is `dceef70` (local). It contains the public
boundary, shared frame, Overview/Process, full icon/sprite inspectors, Lab
refinement, three historical curated records and the shared standalone exporter.
At that checkpoint, 521 units, type checks/build, twelve three-engine developer
journeys and six signed-in/in-flight emulator journeys passed. The browser
matrix covers 64 page/viewport/text combinations per engine. These are local
results, not CI or deployed evidence.

Capture inspection found that the first public fixture omitted the publisher
UID from its replay player, so the old client correctly showed unavailable row
metadata. That is a fixture issue, not a new application defect. The reference
fixture was corrected and re-captured before its first publication; the final
run also asserts visible opponent metadata.

The core-screen set uses a separate detached `v0.6.2` checkout at `37a122d`.
Application/assets/build source stayed unchanged. Its synthetic emulator lane
used offset 1000. [Capture metadata](v0_6_3_core_reference/README.md) records the
exact source tree, browser, fixtures, dimensions and hashes. The new reference
is not a relabelled intermediate mobile review and has no new human/device
acceptance claim.

The exporter consumes the same validated record, `ReviewCase` component and
styles as the site, with selected frozen media, fonts/license and relative URLs.
The build publishes only curated media/direct-entry cases. Cloud acceptance now
publishes the same static artifacts into its own build; otherwise references
would silently be missing there despite a valid route shell.

## Documentation authority pass

| Area | Disposition |
|---|---|
| Developer routes/source/review/export | New canonical `reference/app/developer_area.md`; plan and accepted study retained as dated decisions |
| App/code/architecture | Updated lazy public/player tree, current route roster and initialization behavior; future trusted backend remains deferred |
| UI and board reference | Corrected stale muted/danger colours; current source-based inspectors and their renderer limits linked |
| Assets | Runtime CSS owns exported palette values; source SVG/font/PNG bytes and animation inventory remain canonical |
| Capture/testing/release | Reusable record/export procedure, frozen media, isolated cloud static publication and separate acceptance/deployment gates documented |
| Root/docs/working indexes | Compact operational entry and explicit local implementation vs released 0.6.2 state; active owner/handoff indexed |
| Backend and Lab detail | Existing schemas/domain/report contracts retained; prepared Process/Lab summaries link deeper, without rewriting measurements |
| Completed plans/archives | Existing paths, release receipts and historical media retained; no blanket archive rewrite or automatic Markdown publication |
| Process-story drafts | Remain archived/later scope; the first Process page presents actual procedure and examples |

The closing validation record below owns final results and remaining acceptance.

## Findings during overall review

- A generic developer button font rule overrode the canonical action specimens;
  the rule now has zero scope specificity so actual action classes keep their
  font/ink/disabled states. Specimens retain app dimensions.
- The broad browser lane found overflow in the icon catalog's “In comparison”
  label at 320px/200% text. Font-rule specificity and safe label wrapping were
  corrected; all 21 affected three-engine developer journeys now pass.
- The report Search grid needed flexible columns at short desktop/landscape
  widths. No telemetry values or denominators changed.
- Firefox makes a scroll container a default tab stop. The shared outer scroll
  owner now has explicit `tabIndex=-1`, keeping the skip link first while content
  remains programmatically focusable.
- The first release-reference public fixture omitted the replay publisher UID;
  its metadata therefore rendered as unavailable. Correct the fixture and
  the final harness asserts the expected opponent before freezing the reference.

## Local overall review and handoff

Implementation checkpoints C–E are complete. The accepted visual-study direction
is integrated; the full implementation has not yet received Bryan's separate
visual pass. No push, version bump, tag or app deployment was performed.

- App/scripts/tools type checks and 526 units in 53 files pass.
- The full ordinary lane plus affected reruns cover 252 distinct passing
  journeys and six intentional Firefox/WebKit WebGL qualification skips. The
  affected final lane passed 99 journeys; subsequent review-layout and native
  raster refinements each passed their targeted three-engine checks. Native
  comparison scroll synchronization and its off state also pass all three engines.
- All 60 emulator journeys pass, including cached-sign-in isolation and a held
  private write across navigation. Five repetitions of stale-write, account-
  switch and reset races pass in all three engines (30 + 15 = 45).
- A separate synthetic production-configured/non-emulator build passes the
  fresh developer boundary in three engines with external requests blocked.
- The 66-view accessibility audit reports zero violations. Eight incomplete
  contrast checks remain around layered sprite/board content and offscreen
  elements; this does not establish screen-reader or physical-device acceptance.
- Served source/media/report parity checks cover 108 routes/files. All 41
  canonical SVGs match their source bytes; 1×/2×/10× masks match without
  antialias colours. Recorded Lab JSON is unchanged. All 157 checked local
  Markdown links resolve.

The standalone reference export uses the same record, React viewer and styles.
Only its selected 26 files are included. Their delivered hashes match; all three
engines exercise comparison selection, reload/history, native/full panes, opened
technical context and 320px/200% text without errors or external requests.
The [frozen source/reference](v0_6_3_core_reference/README.md) owns its source
identity; the shared report receipt and final build hashes are recorded in the
validation JSON prepared with this handoff.

Final export review caught intrinsic sizing in the longer reference selector
and unbroken hashes in opened technical lists. A constrained grid/select and
long-word wrapping fix both in the site and the export. The builder also disables
Vite's implicit public-directory copy for standalone exports, keeping unrelated
public files out of the selected report.

One secondary browser run initially shared Playwright's artifact directory with
a parallel pixel run, causing trace-cleanup failures after assertions. The
production-configured check was rerun in its own output directory and passes.
Export image checks wait for selected DOM identity and actual lazy-image loading
before decoding, avoiding a Firefox driver race.

Integrated preview: `http://starship:46862/dev/`. The existing manual preview,
emulators and Carbon forwarding remain separate. Carbon SSH timed out during
this pass, so no fresh Carbon delivery claim is made. The reference export and
app are verified from Starship; physical/browser acceptance remains separate.

Next gate: Bryan's visual pass on the integrated developer area, then separately
authorized 0.6.3 release preparation, exact-source CI and publication checks.
The inherited real-account sharing/device handoff remains open in the 0.6.2
release record. This local implementation does not close it.

Final presentation review also puts the recorded Lab date and rule/run context
beside the primary heading. Full provenance remains available below; missing
historical dates stay unavailable rather than inheriting the current date.

The [standalone reference export](http://starship:8787/reports/e6f63cc59c388bde2d824e4fff4cc862cbda98f05d2a031e6727b72a65c631ae/)
is the verified 26-file snapshot; earlier draft export snapshots are superseded.
The [machine-readable validation](v0_6_3_validation.json) records build/source
identity, artifact hashes, exact result scopes and preserved service boundaries.

## Carbon access follow-up

Carbon later became reachable over SSH. Direct access to Starship's preview
port was rejected, so a separate transient user service,
`gomoku2d-v063-carbon-preview.service`, now forwards Carbon's loopback port
46862 to the integrated preview. Open `http://localhost:46862/dev/` on Carbon.
All 194 qualified files were fetched on Carbon through this forward and matched
the recorded hashes. The existing 8013/Auth/Firestore tunnel is unchanged.
This verifies delivery from Carbon; Bryan's browser/visual acceptance remains
separate. The earlier Starship URL remains available locally.

## UI review refinement after the integrated preview

Bryan asked to show the process through useful working tools, beginning with UI
review. The generic prose Process page and its overview copy are removed. The
navigation now contains Overview, Design & assets, Lab and UI review; the old
Process URL redirects to the review index. The index separates released screen
references from historical layout/icon reviews. The actual screenshot checklist
is available in both the index and case viewer, with a link to its maintained MD
procedure. No new claims of human acceptance are introduced.

The default comparison uses source-width proportions and complete captures.
For example, desktop 1280 and portrait 390 receive columns in that proportion,
so their source pixels use one display scale. Captions share a row and screenshot
top edges align. Narrow layouts stack at the same scale; native pixels remain
available for detail. Full captures can be toggled off for bounded scroll panes.
Scroll synchronization is offered in native-pixel or bounded-pane mode.
Frozen screenshots, captions and source identities are unchanged.

Build/type checks and all 526 units pass. The developer suite passes 24 journeys
across Chromium/Firefox/WebKit. The final caption-alignment adjustment passes
six targeted viewer journeys. These assert matched image scale, complete frames,
aligned top edges, history, native pixels, scrolling, legacy redirect and narrow
layouts including 320px/200% text. The standalone export also passes all three
engines using the shared component/styles. A 27-view audit with the checklist
opened reports zero violations and 15 incomplete colour-contrast entries; these
still need manual judgment and do not establish screen-reader/device acceptance.
The initial scroll-off test raced the URL-state commit; it now waits for the off
control before moving the pane. JSDOM's missing scrolling API is stubbed only in
the route test fixture.

The 108-route/file source/media parity check passes, and all 194 final build
files were fetched through Carbon's localhost forward with matching hashes.
The integrated preview remains `http://localhost:46862/dev/` on Carbon. Owned
headless browsers and the temporary standalone-export server were closed.
Existing manual services and forwarding remain active. No push, release or
production deployment was performed.

The [refinement validation](v0_6_3_review_refinement.json) qualifies this later
build; the earlier validation record remains a dated baseline. Bryan's visual
review of the new layout is the next acceptance gate.

## Screen gallery and mobile-reference follow-up

Bryan asked to expose all screen/view references rather than switch a dropdown,
and questioned the “short/enlarged” labels and portrait height. The reference
now lists all six normal desktop/portrait pairs on one page, with jump links.
Layout checks are secondary under a closed disclosure, named landscape and
200% text. An optional validated comparison purpose owns that grouping. Existing
pair URLs, including QA deep links, still focus/jump to their section; QA links
open the disclosure. Native/full/sync inspection applies to the focused pair,
with scroll synchronization confined to that pair. Standalone exports reuse the
same gallery. No screenshot pixels or capture identities were changed.

Inspection of the frozen harness confirmed that 390 × 844 at DPR 1 was a generic
desktop-Chromium viewport resize. It was not measured on Bryan's phone and did
not emulate Android browser chrome, user agent or touch. The record now says so.
The short-landscape (844 × 390) and 200%-text (320 × 800) cases remain useful QA
inputs, separately labelled, rather than primary product views.

A OnePlus-class Android phone is a reasonable proposed real-device reference.
[OnePlus 13 specifications](https://www.oneplus.com/us/13/specs) give a physical
19.8:9 display; that does not establish usable CSS-pixel dimensions. The
[visual viewport](https://developer.mozilla.org/en-US/docs/Web/API/VisualViewport)
records the visible web area, while browser UI and display settings matter to
what fits. Browser mode and actual viewport measurements remain open inputs
for a new capture set. Do not silently resize/relabel the frozen release images.
[What is my viewport](https://whatismyviewport.com/) is an available readout for
the phone's viewport and DPR. No specific OnePlus CSS size was adopted from
conflicting third-party device tables.

Gallery validation: build/type checks and 527 units pass; the full developer lane
passes 27 journeys across three engines. A final focus-retention correction
passes nine targeted viewer journeys, including keyboard activation of the first
inspection control. The refreshed standalone export passes all three engines.
The 27-view audit reports zero violations and 15 incomplete contrast entries,
with manual judgment still required. Source/media parity covers 108 files;
all 194 final build files were fetched and hash-checked from Carbon. Temporary
export hosting and owned headless browsers were stopped. The `gallery_followup`
entry in [refinement validation](v0_6_3_review_refinement.json) records this later
source/build and its pending Android-reference measurements.

## Preview recovery, October 25

The unmanaged 46862 preview process had exited. Carbon's old reverse-forward
SSH session still held its loopback port, so reconnect attempts failed. That
session was identified by process generation and its sole listener on 46862;
only that stale preview session was closed. The existing manual 8013/Auth/
Firestore services and forwarding were retained.

The preview now runs as Starship's transient user service
`gomoku2d-v063-preview.service`, with automatic restart. Carbon owns the local
forward in its transient user service `gomoku2d-v063-preview-tunnel.service`:
`127.0.0.1:46862` forwards over SSH to Starship's `127.0.0.1:46862`. Its client
uses keepalives, a bounded connect timeout and automatic restart. The former
Starship reverse-forward unit `gomoku2d-v063-carbon-preview.service` was stopped.
This avoids depending on a remote SSH listener being released after a disconnect.
The review URL remains `http://localhost:46862/dev/reviews/core-screens-062/`
on Carbon.

Both owned service processes were terminated as a recovery probe. They restarted,
and Carbon fetched all 194 build files with matching hashes. The local receipt
is `gomoku-web/test-results/v063-preview-service-recovery.json`. No application/build,
production deployment or private-data changes were made.

## Retrospective UI pilot, October 25

Bryan authorized a bounded historical recapture pilot before backfilling more
milestones. The [pilot contract and reproduction procedure](v0_6_3_retrospective_capture.md)
cover v0.2.4 and v0.6.2, with Home, Match, Replay and Profile captured from exact
tagged applications and their generated Wasm. Luna implemented guarded capture
and build-receipt tools; the primary built the tags, orchestrated capture,
reviewed all 16 images and integrated the curated comparison.

The UI review index keeps Current screens first, adds UI evolution, and retains
the existing Design & layout reviews. The new **From local play to replay
analysis** entry lists all eight desktop/portrait version pairs. Existing frozen
evidence and public review URLs are unchanged. Both builds use synthetic local
guest data with cloud deliberately unconfigured. Portrait 412×800 is provisional
Android touch emulation, not a measured OnePlus reference. Exact source/tool/
fixture and per-image provenance live in the [frozen capture manifest](v0_6_3_retrospective_capture/pilot-2026-10-25/capture.json).

Type checks, 527 unit tests, six capture-tool checks and production build pass.
All 30 developer journeys and the standalone pilot export pass across three
engines. The 18-view audit has no overflow or automated violations; six incomplete
contrast results still require manual judgment. Delivery verifies 126 routes/
files, preserves recorded Lab reports, and hash-checks all 212 integrated build
files from Carbon. [Pilot validation](v0_6_3_retrospective_capture/pilot-2026-10-25/validation.json)
records the result boundaries and artifact identities.

Open `http://localhost:46862/dev/reviews/ui-evolution-024-062/` on Carbon. Managed
preview/forward services remain active. Temporary historical previews, export
hosting and the two owned detached worktrees were removed; original manual
services and the foreign worktree were preserved. No user browser tabs were
opened. The work is committed locally, with no push or production publication.
Bryan's presentation review and phone calibration remain open before more
history is added. Desktop Profile clipping and long-name Match truncation are
recorded product follow-ups, with the tagged captures left intact.

## UI history implementation, October 25

Bryan selected the C milestone overview and B individual screen trails, then
authorized the full source-backed history implementation with primary ownership
of design. The [implementation record](v0_6_3_history_implementation.md) and
[qualified capture set](v0_6_3_retrospective_capture/history-2026-10-25/README.md)
own decisions, original media, checks and the local handoff.

The source now covers 25 release identities, 24 meaningful UI milestones, 15
screen histories and 274 selected desktop/portrait images. Introductions,
retirements and the Lab merge use real single captures rather than empty
comparison columns. Existing cases, the endpoint pilot and current references
remain unchanged and addressable. The viewer remains static public content,
with no player/Auth/private-history/Phaser/Wasm initialization.

Type checks, 540 unit tests, 28 capture checks, three-engine history/developer
journeys, standalone export compatibility and the 48-layout media audit pass.
The Starship preview has full 544-file served-byte parity. Carbon went offline
before the final-build last-hop recheck; its forward remains configured. That
check and human/physical-device acceptance stay open. The original manual
services and foreign worktree remain in place; all 25 temporary tagged worktrees
and capture services were cleaned up. Work is committed locally without release
or production publication.
