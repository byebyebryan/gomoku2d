# Developer area

This contract describes the implemented 0.6.3 source. The published release
remains 0.6.2 until its separately authorized release. Detailed implementation
and validation state belongs to the [execution record](../../working/v0_6_3_execution.md).
The [brief](../../working/v0_6_3_developer_site_plan.md) retains design decisions;
the [visual study](../../working/v0_6_3_design/README.md) is an accepted proposal,
not application/deployment evidence.

## Public route and lifecycle boundary

`src/app/App.tsx` selects two lazy trees with React Router matching:

| Tree | Routes | Effects |
|---|---|---|
| Developer | `/dev/`, `/dev/process/`, `/dev/reviews/`, curated case URLs, `/lab/`, `/visuals/` | Static references/data; no Auth, local player/history, game controller, Phaser or Wasm initialization |
| Player | Game/settings/private Profile, Rules/Guide/policies, shared replay and public player | Existing optional authentication and player lifecycle |

Shared replay and public player retain their authentication-aware boundary:
Auth is available there while private cloud-session initialization remains off.
Do not treat them as equivalent to an account-independent developer reference.

Fresh developer visits must remain independent even with persisted sign-in.
Navigating from the game unmounts its UI controllers and removes subscriptions,
but does not clear singleton stores, force a cloud flush or cancel in-flight
promises. Returning starts the ordinary controllers and re-evaluates pending
sync. A cloud test holds a private write across developer navigation; another
opens developer pages in a fresh tab with a real emulator-persisted session.
Neither test uses production accounts.

The developer frame owns one scroll region (`.dev-page`). Header/navigation
remain in normal flow and wrap. Skip-to-content and section changes move focus
to public content. Section changes reset the frame to the top and focus content
with `preventScroll`, keeping the site header/navigation visible. The explicit
skip link still scrolls to content; reload/history restore query state. Unknown developer pages
and records offer public recovery. CSS is scoped to `.dev-page` or report modules
so reference typography does not change player screens.

## Page and data ownership

| Surface | Owner | Source contract |
|---|---|---|
| Overview | `src/developer/DeveloperPages.tsx` | Direct entry to Visual design, UI design and Bot lab; no automatically rendered Markdown |
| Visual design | `AssetPreviewRoute`, `IconInspector`, `SpriteInspector` | Actual CSS classes/tokens, canonical SVGs and source sprite inventory |
| UI design | `reviews.json`, `review_record.ts`, `ReviewCase.tsx`, `ReviewChecklist.tsx` | Screen references, historical comparisons, the actual screenshot-review checklist and separate technical/human status |
| UI history | `history.json`, `history_record.ts`, `HistoryRoutes.tsx` | Release provenance, meaningful milestones, stable screen identities, named states and qualified retrospective captures |
| Bot lab | Existing report loaders, report panels and DOM proof boards | Unchanged `reports/lab/` JSON, recorded revisions/budgets and bounded analysis semantics |

`src/app/links.ts` owns the shared section labels, destinations and order:
Visual design, UI design, Bot lab. Developer navigation and footer links,
overview cards and Home consume that definition. README and documentation
link lists use the same labels/order, with Developer as their overview entry.

Visual design covers the visual system and its source assets; UI design covers
screens, layout, interaction and their evolution. UI design retains Current
screens, UI history and Design reviews as its three views. Existing route and
record identities remain stable; historical captures retain their original names.

Bot lab keeps `tab=ranking|search|analysis` and `match` deep links. Its detailed
metrics and opponent/game drill-down remain available; secondary ranking metrics
live in the expanded row. An in-flight load survives switching local views.
Do not rerun or relabel old tournaments to update presentation.

Visual design uses `view=style|icons|sprites`, with the legacy default Style. `icon` or
`compare` selects canonical names (unique, maximum four); `q` filters name/category.
`grid=off` and `surface=light` affect inspection only. Sprites accept canonical
`animation` and named `tint`; unknown state falls back safely. Query state survives
reload and Back/Forward. Catalog/native samples use the 24px source geometry;
integer enlargement exposes existing margins rather than normalizing glyphs.

Style specimens use actual action classes and runtime CSS tokens. The asset
publisher derives exported palette values from CSS while retaining editorial roles.
Sprite previews reuse `multiplyTintRgbaPixels`, preserve alpha, and check ranges
against board constants. Only the selected loop animates, offscreen animation
pauses, hidden pages stop playback, and reduced motion uses stepping. Historical
asset-only teal is a preview colour; the inspector is not Phaser acceptance.

## Curated reviews and immutable media

`src/developer/reviews.json` is an explicit allowlist, not a recursive docs export.
Each typed/validated record includes:

- Stable ID, title, date, scope, revision, question and disposition.
- One or more comparisons with captions and explicit conditions; optional
  `purpose: "layout-check"` places a QA case in the secondary disclosure.
- Local media filename, source path, SHA-256, dimensions, kind and source revision.
- Findings with fixed/deferred/open/recorded disposition.
- Technical evidence, actual human review, follow-up and public source links.
- Optional source `versions` in ascending order and an authored `process` that
  groups normal comparison IDs into design steps. Each normal comparison appears
  in exactly one step; layout checks stay in their separate disclosure.

`versions` identifies the product releases covered by a case. Original notes,
later compiled release histories and each image retain their own document/build
revisions; a release revision must not be used as the revision of a later document.

The index leads with Design reviews, grouped by source minor line and ordered
latest first. Current screens and UI history remain direct sibling links;
reference galleries, including the endpoint pilot, remain available from the
index. A `retrospective review` is a newly assembled account of documented design
work and requires source versions and process steps. Its "Assembled" date is
distinct from the original decision or capture date. A `retrospective comparison`
retains its explicit "Recaptured" date and visible-change observations. Existing
dated design/review records remain directly addressable. The
[retrospective pilot](../../working/v0_6_3_retrospective_capture.md) owns the
capture protocol and its v0.2.4 / v0.6.2 qualification; it does not imply a
complete version archive or original-period acceptance.

The build validates schema, stable IDs, safe media paths, unique IDs/files and
PNG hashes/dimensions before copying selected files to `/reviews/<id>/`.
Source/evidence links use public GitHub URLs; internal report-host URLs are not
public dependencies. Only curated case routes receive direct-entry HTML.

The viewer lists all normal screen/view comparisons on one page, with jump links
rather than a screen-selection dropdown. The released reference shows six
sections: Home, Match, Replay, Settings, Profile and Public player. Landscape
and 200%-text QA cases live in a closed “Layout checks” disclosure. Legacy
`pair` deep links still jump to/focus their section; QA links open that disclosure.
Inspection query settings apply to the focused pair and never synchronize scroll
between unrelated screen sections.

Structured design cases place the relevant comparisons beneath each authored
step. Their comparison IDs and inspection query behaviour remain unchanged.
Optional `captureType: historical|retrospective` applies only to application
captures and labels their actual origin. Design mockups and asset renders keep
their own kinds; unrecorded legacy capture conditions are not inferred.

Each comparison defaults to complete captures at one shared scale. Columns are
proportional to source-image widths: a 1280px desktop capture gets a wider pane
than a 390px portrait capture, with the same source-to-display scale and no
upsampling. Narrow layouts stack the captures while preserving that shared
scale. Native-pixel inspection remains available for detail. Full captures can
be turned off for bounded scroll panes, with optional synchronized scrolling.
Top edges align even when captions wrap.
The `native=1`, `full=0` (bounded panes), `pair` and `sync=off` query selections
survive reload/history; existing `full=1` links still show complete captures. Desktop/portrait pairs and
asset-family comparisons are labelled as such; they are not invented before/after
claims. Technical success does not establish human approval. Unknown historical
conditions remain explicitly unknown. Later fixes/acceptance are dated additions;
never silently redraw a historical review from current source.

The initial catalog contains accepted action icons, public-player/mobile
checkpoints and a released 0.6.2 core-screen reference. That reference comes from
a detached exact release checkout and explicit guest/emulator fixtures, separate
from later developer-area code. Human/device acceptance remains stated per record.

The frozen portrait images use generic 390 × 844 CSS-pixel viewports in desktop
Chromium at DPR 1. The harness resized the viewport; it did not emulate Android
user-agent/touch behavior, real browser toolbars or a measured OnePlus viewport.
Their dimensions and hashes must not be changed to imply device validation.
A new Android reference should use a separately captured set with measured usable
viewport, browser/display mode, pixel ratio and source identity. Bryan's OnePlus
is the proposed real-device reference; its exact viewport remains unmeasured.

## Milestones and screen histories

`/dev/reviews/history/` groups meaningful UI releases from 0.2 through 0.6 in
a milestone rail and lists every changed screen/state for the selected release.
The rail lists minor lines and their milestones latest first.
On narrow layouts the rail is an operable disclosure. Current screens, UI history
and Design reviews remain direct sibling entries; existing case URLs and the
endpoint pilot remain valid.

`/dev/reviews/history/<screen-id>/` shows a latest-first capture trail for a named
screen state and an inspection of the selected change. Screen/view links remain
visible. `version`, `state` and `layout=desktop|portrait` survive reload and
history. Going back to the originating milestone restores its scroll region,
focused change and open source disclosures. Invalid selections fall back safely;
unknown screens offer a recovery link.

Updates pair neighbouring relevant source versions. Introductions show one new
capture; removals show the last real capture with the later retirement label.
Bot Report and Analysis Report merge into Lab at 0.5.1. Visuals retains one
identity across its static-to-React update. URL migrations alone do not create
new screen identities. Unrecorded capture coverage never means a screen was absent.

The catalog explicitly allowlists releases, screens/states, snapshots and
milestone transitions. Browser and publisher validate references, lifecycle
cardinality, chronological source versions and safe paths. The publisher also
checks original manifest identity/status, PNG dimensions/digests and absence of
unexpected capture errors. Selected media and manifests publish under `/history/`;
only allowlisted screen routes receive direct-entry HTML. No player state or
backend is needed to browse them.

The overview uses compact captions, with image inspection controls below each
comparison and detailed provenance in disclosures. Screen inspection keeps the
full captions and controls. Snapshot identities bind their original manifest;
public image filenames include their content digest so a later qualified
recapture receives a different URL.

The [history implementation record](../../working/v0_6_3_history_implementation.md)
owns the accepted design and qualification results. The
[tag inventory](../../working/v0_6_3_history_inventory.md) and change matrix keep
source evidence separate from actual captures. These are recreated tagged
applications in a modern runtime, with synthetic local/emulator data and an
unmeasured generic Android touch profile. Capture provenance and technical
success do not establish original-period or physical-device acceptance.

## Reusable standalone exports

The site and an ad-hoc export use the same `ReviewRecord` contract, React
`ReviewCase` component and styles. From `gomoku-web`:

```sh
rtk proxy npm run review:export -- mobile-records-062 test-results/review-exports/mobile-review-01
```

The destination must be new and under `test-results/review-exports/`; it cannot
overwrite source, the ordinary build or a prior export. The selected media,
fonts/license, record, source context, styles and comparison controls are bundled
with relative URLs. No private store, auth, game engine or backend is included.

For the whole developer area, including cases, history, asset inspection and Lab:

```sh
rtk proxy npm run developer:export -- test-results/developer-exports/dev-preview-01
```

This also requires a new directory. It uses the actual `DeveloperApp` with hash
routing and a runtime asset base, so an immutable report prefix can be assigned
after building. Play links to the public game. The publishers copy only their
selected public assets/data; no player entrypoint or backend is included.
`export.json` records source qualification, observed tools, counts and a payload
file inventory. A clean source tree identifies a commit; it does not claim a
fresh locked dependency installation or a production release.

Publish only that prepared export with the shared report publisher for private
review. Adding a record to the public catalog remains a deliberate source change
and goes through the normal release process. Ordinary Pages publishing uses the
same repo/package/artifact and custom domain; no new host or credentials.
Use the shared report host's browser URL across viewing machines. A static
review does not need a per-project preview port or per-device SSH forwarding.

`publish_reviews.mjs` and the asset/report/SPA publishers accept an optional build
directory. The cloud harness publishes the same static inputs into its owned
build, keeping developer checks meaningful in production-style and emulator
contexts. Local cloud checks use offset 1000 when manual services own defaults.

## Documentation roles

Markdown retains detailed contracts, setup, procedures and dated evidence for
agents/maintainers. The developer area is prepared human presentation and useful
inspection tools. Current behavior belongs in reference docs; active work in
working notes; completed snapshots stay indexed at stable paths. Release records
own publication facts. Keep the process record current as procedures improve;
retrospective journey articles are later work.

The developer navigation focuses on working tools: Overview, Visual design,
UI design and Bot lab. The earlier prose Process page is removed; `/dev/process/`
redirects to `/dev/reviews/` without initializing the player tree. The review
index and cases expose the screenshot-review checklist alongside real references
and findings, rather than presenting a generic development manifesto.
