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
to public content; reload/history restore query state. Unknown developer pages
and records offer public recovery. CSS is scoped to `.dev-page` or report modules
so reference typography does not change player screens.

## Page and data ownership

| Surface | Owner | Source contract |
|---|---|---|
| Overview | `src/developer/DeveloperPages.tsx` | Direct entry to asset inspection, Lab and UI review tools; no automatically rendered Markdown |
| Assets | `AssetPreviewRoute`, `IconInspector`, `SpriteInspector` | Actual CSS classes/tokens, canonical SVGs and source sprite inventory |
| Lab | Existing report loaders, report panels and DOM proof boards | Unchanged `reports/lab/` JSON, recorded revisions/budgets and bounded analysis semantics |
| UI review | `reviews.json`, `review_record.ts`, `ReviewCase.tsx`, `ReviewChecklist.tsx` | Screen references, historical comparisons, the actual screenshot-review checklist and separate technical/human status |

Lab keeps `tab=ranking|search|analysis` and `match` deep links. Its detailed
metrics and opponent/game drill-down remain available; secondary ranking metrics
live in the expanded row. An in-flight load survives switching local views.
Do not rerun or relabel old tournaments to update presentation.

Visuals uses `view=style|icons|sprites`, with the legacy default Style. `icon` or
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

## Reusable standalone export

The site and an ad-hoc export use the same `ReviewRecord` contract, React
`ReviewCase` component and styles. From `gomoku-web`:

```sh
rtk proxy npm run review:export -- mobile-records-062 test-results/review-exports/mobile-review-01
```

The destination must be new and under `test-results/review-exports/`; it cannot
overwrite source, the ordinary build or a prior export. The selected media,
fonts/license, record, source context, styles and comparison controls are bundled
with relative URLs. No private store, auth, game engine or backend is included.

Publish only that prepared export with the shared report publisher for private
review. Adding a record to the public catalog remains a deliberate source change
and goes through the normal release process. Ordinary Pages publishing uses the
same repo/package/artifact and custom domain; no new host or credentials.

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

The developer navigation focuses on working tools: Overview, Design & assets,
Lab and UI review. The earlier prose Process page is removed; `/dev/process/`
redirects to `/dev/reviews/` without initializing the player tree. The review
index and cases expose the screenshot-review checklist alongside real references
and findings, rather than presenting a generic development manifesto.
