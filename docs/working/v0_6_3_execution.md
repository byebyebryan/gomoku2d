# 0.6.3 implementation execution

Bryan accepted the [visual study](v0_6_3_design/README.md) and authorized full
implementation on October 24, 2026. Scope follows the
[brief](v0_6_3_developer_site_plan.md), checkpoints C–F. Commits stay local;
release/deployment remain separate. The 0.6.2 manual preview, emulators, Carbon
tunnel and demo fixtures remain owned by the existing services.

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
  raster refinements each passed their targeted three-engine checks.
- All 60 emulator journeys pass, including cached-sign-in isolation and a held
  private write across navigation. Five repetitions of stale-write, account-
  switch and reset races pass in all three engines (30 + 15 = 45).
- A separate synthetic production-configured/non-emulator build passes the
  fresh developer boundary in three engines with external requests blocked.
- The 66-view accessibility audit reports zero violations. Nine incomplete
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
