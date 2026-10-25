# UI history implementation

Owner: Bryan. Authorized October 25, 2026 with luna-goal-loop. Start source
`aed3467`. The primary owns design, UI layout, integration and acceptance;
Luna Max owns bounded source/capture/tool implementation. Commit cohesive stages
locally. No push, tag/version change or production publication is authorized.

## Accepted design and scope

Implement C as the history overview and B as individual screen histories from
the [accepted direction](v0_6_3_history_design/README.md). Audit every release tag
from 0.2.1 through 0.6.2 to identify meaningful UI milestones. Capture qualified
neighbouring versions, including account/settings/analysis/sharing/tool views
where meaningful, rather than stopping at the existing endpoint pilot.

Preserve existing review URLs and immutable media, current references, recorded
Lab data and the public developer boundary. The history is curated static data;
it must not initialize player history, authentication, cloud, Phaser or Wasm.
Historical capture builds may execute their own engines and isolated synthetic
emulators; the resulting reference viewer may not.

## Primary design decisions

- `/dev/reviews/history/` opens C. `version` selects a milestone; the default is
  the latest curated milestone. The desktop rail groups releases by minor line.
  On mobile it becomes a labelled disclosure, closing after selection and
  reopening appropriately when crossing back to desktop.
- `/dev/reviews/history/<screen-id>/` opens B. Visible screen links include
  retired screens. Version, screen state and portrait/desktop selection survive
  reload and history. A back-to-milestone link and browser Back restore the
  originating change, scroll position and focus within the developer scroll
  region. Invalid selection gets a safe fallback or explicit screen recovery.
- Current screens, UI history and Design reviews remain direct sibling entry
  links. The existing review index and its cases remain addressable. New history
  gets a deliberate entry in the existing index; the original pilot stays intact.
- The overview lists all changed screens and their relevant states with direct
  jump links. There is no screen dropdown. Primary content is the images and
  concise observed change. Source/build/conditions are secondary disclosures.
- Updated views use neighbouring relevant snapshots at matched pixel scale.
  Added screens use a single new capture. Removed screens use their last real
  capture, labelled with that source and the later removal version. Single
  portrait images retain their normal review scale. No empty counterpart columns.
- Replacements/merges/splits are relationships between actual screen identities.
  URL renames do not automatically create a new screen. Missing capture coverage
  means unrecorded, never absent/removed. Retired trails remain discoverable.
- B shows a chronological trail for the selected screen state, storing each
  snapshot once. The chosen change can open the same comparison controls used
  by existing reviews. Inspect change is the generic navigation action.
- Use current fonts, tokens, controls and scoped CSS. Keep full captures and
  source-width scaling; native inspection remains available. Keep technical
  acceptance distinct from Bryan/device/screen-reader acceptance.

## Catalog contract

`history.json` is an explicit allowlist, independently validated in the browser
and publisher. The primary owns `history_record.ts` and the presentation code.

1. Releases identify actual version, date and exact commit, in chronological
   order. Milestones are a UI-significant subset of those releases.
2. Screens have stable IDs, names, description and named view states (for example
   Profile record/account/reset). Route changes can retain one screen identity.
3. Snapshots identify one screen/state/source version with conditions, capture
   date/type, original manifest identity and desktop/portrait media. Capture and
   source provenance remain explicit; each image is stored once and reused.
4. A milestone contains screen changes with added/updated/removed lifecycle,
   concise observed meaning, one or more state transitions and optional
   replacement/merge/split links. Each transition has before, after or both
   snapshot references. New/retired states within an existing screen can have a
   single side without falsely declaring that the entire screen was added.
5. Snapshot version/state/screen references, lifecycle cardinality, chronological
   order, media path/dimensions/hashes and relationships must validate. Missing
   layouts have explicit notes rather than an invented image. Historical
   unmeasured conditions retain their limits.

Assets publish to `/history/`, with stable allowlisted filenames and copied
manifest files. New source routes get static entry HTML. Existing publishers
and standalone review exports retain their contracts. No whole-docs publication
or dependencies on the internal report host belong to the public catalog.

## Capture and acceptance gates

Luna first audits tagged UI/build/fixture contracts and returns the inventory to
the primary. The primary selects source-supported milestones/states and approves
adapter contracts before Luna implements capture extensions. Exact tagged
application/assets/Wasm stay unchanged; use owned detached sources, locked
dependencies where available and honest recorded resolution where not.

Use 1280×900 desktop and provisional generic 412×800 Android touch portrait at
DPR 1, CSS-pixel output, 100% text, en-US/UTC and reduced motion. Preserve the
existing 390×844 reference bytes; do not call either profile a measured OnePlus.
Synthetic data only, no production accounts or cloud writes. Historical report
rendering uses its tagged presentation and documented data identity, without
rerunning costly bot tournaments merely for screenshots.

Exit gate: source-backed milestone/lifecycle inventory; qualified capture and
manifest coverage with honest remaining technical gaps; working C/B navigation
and all lifecycle forms; data/type/unit/build and appropriate browser/reflow/
privacy/publisher/media verification; primary visual review; Carbon byte delivery;
accurate documentation; staged local commits and clean worktree. Close only
owned worktrees/previews/browsers. Preserve original manual services, 46862
managed preview/tunnel, foreign worktree and port8001.

## Execution

- Primary inspected source and accepted design; repository clean at start.
- Luna Max is auditing all tagged UI/report changes and capture adapter groups.
- Primary is defining catalog, routes, layout and navigation semantics.

### Source and capture preparation

The source audit settled 25 peeled release commits, 24 UI milestones, 15 stable
screen families and 89 screen changes. The [change matrix](v0_6_3_history_changes.json)
links 130 inspected source paths. 0.5.0 remains a provenance/report baseline;
there is no 0.6.0 tag. Search and Pairwise become named report states in 0.4.1,
not 0.4.0. Search means search-performance metrics, not a text filter.

The primary built all 25 exact tagged sources in separately owned detached
worktrees, with each tag's dependency installation, Wasm build and production
postbuild. Actual tools were Node 26.12.1, npm 12.2.1, Rust/Cargo 1.99.1 and
wasm-pack 0.14.0. These are modern-runtime reconstructions, not original binaries.
All tracked source and lock inputs stayed unchanged. The first three tags have
no committed Cargo lock: their actual resolved locks are retained separately.
Original build receipts and the documented normalization of Firebase mode names
retain their identities; normalization changed no source or artifact bytes.

The first four tags run without Firebase configuration. 0.3.0 through 0.5.4
use the actual tagged SDK against owned demo Auth/Firestore services through a
loopback-only CONNECT/TLS proxy; 0.5.5 onward uses the tags' native emulator
hooks. Synthetic Google-provider users come from the local Auth emulator, and
Firestore requests use each tag's actual rules. No real OAuth/account or
production data is used, and SDK responses are not fabricated. The optional
hosted GAPI popup bootstrap remains blocked; its exact URL-attributed abort and
console observations are retained separately. This does not test Google's
external popup sign-in flow.

The first temporary emulator was started in the manual emulator's working
directory, which exposed a shared debug-log path. The primary preserved the
observed log, stopped only that temporary process, and restarted it in its owned
directory. Manual service processes, ports and data remained in place. Later
temporary services use their own working directory and logs.

The primary owns the C/B code, navigation/restoration contracts, source graph
validator and publisher. Luna owns the external capture adapters and sequential
capture execution. Source-specific readiness corrections are external: fonts
are live specimens rather than images, older labels/control roles differ,
CSS-module selectors cannot be used as literal classes, and portrait layouts
may deliberately hide attached status labels. Legacy proof views can overflow
the document while the visual viewport remains 412×800. Preserve those pixels
and report both metrics rather than altering the source to pass a check.

Each failed attempt keeps a distinct directory and diagnostics; successful
retries use new directories. Only complete runs without unexpected network,
console or page errors can qualify for the public catalog. The first four
sources' 40 PNGs passed independent SHA-256/dimension/readiness checks and a
visual review. The full 539-test web suite passed before the final source-link
guard; its expanded focused model suite passes 13 tests. Final browser,
assembled-catalog, delivery and cleanup results follow below when complete.

### Completed local implementation

The [qualified set](v0_6_3_retrospective_capture/history-2026-10-25/README.md)
now supplies all 137 snapshots / 274 selected PNGs. Forty-one selected complete
manifests qualify the graph; 43 complete runs and 19 failed attempts remain
archived after later framing captures superseded earlier selections. PNGs and
manifests were never rewritten. Snapshot identities bind their manifest, and
published image names include a content digest. The three generated early Cargo
locks and all 25 build receipts are retained before temporary-source cleanup.

Visual review drove explicit framing corrections: proof boards, portrait Custom
controls, Guide examples and portrait Share/Publish panes now appear in frame.
Natural fixed headers, nested scrolling and end clamping remain intact. The
primary also compacted the overview, removed duplicate state descriptions there,
moved its inspection controls below the images, and kept detailed source context
in disclosures. Screen inspection retains its full controls. The milestone rail
fits its remaining viewport space and keeps the selected release visible.

The synthetic public fixture initially omitted zero-count catalog documents.
The real owner UI rejected its mismatched markers; the primary corrected the
fixture to contain all four page docs, without changing the application guard.
A separate cold-session probe observed local Firestore GET streams return HTTP
200 and then cancel during normal SDK stream lifecycle. The harness now retains
only the two proven native Listen/Write cases separately under strict URL,
method, type, status and failure constraints. Other failures remain fatal. Final
selected media has zero unexpected errors; the optional hosted Google bootstrap
remains explicitly blocked and recorded.

Final checks pass: app/script/tool type checks, 540 web unit tests, 28 capture
contract tests, production build and independent manifest/source/PNG graph
verification. Thirty existing developer journeys and fifteen history journeys
passed across Chromium, Firefox and WebKit; the fifteen history journeys passed
again after the final presentation changes. All 48 milestone/layout combinations
decode their media, preserve matched scale without upsampling and avoid overflow.
Seven accessibility views have zero automated violations; the 320px/200%-text
stress view retains 95 incomplete contrast nodes for manual judgment. The existing
standalone pilot export also passes its nested-prefix/media/control/history and
public-boundary checks in all three engines. Recorded Lab JSON bytes are unchanged.

The managed Starship preview serves the new history. All 544 build files match
their served SHA-256 values, and direct entries respond on its Tailscale interface.
Carbon responded earlier, but went offline before the final-build parity run;
that last-hop recheck is explicitly pending, not a pass. Use
`http://starship:46862/dev/reviews/history/`; Carbon's retained local forward uses
`http://localhost:46862/dev/reviews/history/` when online. The primary is treating
live Carbon availability as an external delivery check rather than holding the
completed implementation open; the Starship artifact is verified and available.

All 25 owned source worktrees, the owned source-root build/dependency caches,
capture previews/proxy and synthetic emulators were removed or stopped. The
regular 46862 preview, existing manual 8013/Auth/Firestore services, existing
Carbon forwarding configuration and foreign worktree were preserved; port8001
was untouched. No user browser tabs opened. No push, version change, release or
production deployment occurred. Bryan's visual review, measured phone calibration,
screen-reader/contrast judgment and final Carbon byte recheck remain separate.

The [validation record](v0_6_3_retrospective_capture/history-2026-10-25/validation.json)
owns exact counts, input/artifact identities, cleanup and acceptance boundaries.
