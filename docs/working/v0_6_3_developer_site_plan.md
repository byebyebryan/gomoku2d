# v0.6.3 developer area: design and implementation brief

Owner: Bryan. Planning review, October 24, 2026, against `4052d94` after the
[0.6.2 release](../archive/v0_6_2_release.md). Application implementation was authorized after Bryan accepted the visual study. Agreed direction is separated from the proposed page, data and
implementation contracts below; review these before starting work.

Bryan accepted the visual study and authorized a full implementation goal loop
on October 24, 2026. Complete C–F through local validation and a reviewable
integrated preview. Commit in stages; push, release tagging and production
deployment remain separate.

The [standalone visual study](v0_6_3_design/README.md) now makes the proposed
shared frame and representative pages reviewable. Its technical checks and
delivery are recorded separately from Bryan's visual acceptance ("looks okay to me").
The [execution record](v0_6_3_execution.md) tracks implementation and validation.

## 1. Agreed direction

Expand the existing GitHub Pages site at `gomoku2d.byebyebryan.com`, in this
repository. Lab and Visuals become part of a coherent developer area. Retain
and refine the established pixel fonts/icons, dark surfaces, semantic colours
and tactile controls, using the existing React/TypeScript/Vite setup.

Gomoku2D is built with AI under human direction, through deliberate research,
design, implementation, review, validation and release work. The workflow
continues to evolve. Record and present that process better first; retrospective
journey stories can later draw on dependable development evidence.

Markdown remains detailed agent/maintainer context. The web area is prepared
human presentation with useful inspectors and review tools. Both need accurate
sources and clear authority. This is not automatic publication of the Markdown
tree or a separate portfolio-domain site.

No new backend service, account requirement, cloud data family, persistence
schema or gameplay feature belongs to this release. Replay collection/metadata
stays in 0.6.4; curated learning follows later; multiplayer remains deferred.

## 2. Foundation and measured gaps

| Existing foundation | Consequence |
|---|---|
| Lab and Visuals share fonts, action classes, dark tokens and closely matching 1180px header/panel layouts | Generalize the established layout/navigation rather than create another visual system. |
| 41 canonical icons, seven sheets, 14 static poses, 22 animations, seven layer examples | Asset coverage is concrete and bounded; include the full current inventory. |
| The Visuals manifest repeats CSS values: muted text says `#a6a6a0` vs runtime `#b5b5ae`; danger says `#e04747` vs runtime `#ef5c5c` | Bind current reference values to source and check drift. Correcting references does not imply changing app colours. |
| The selected-icon report already has native sizes, 10x grids, axes and bounds | Generalize those capabilities; preserve actual legacy margins rather than normalizing glyphs. |
| Lab has URL-driven views, disclosures and DOM-rendered proof boards | Preserve data/meanings/links and refine how people enter and read it. |
| Lab/Visuals mount inside the game shell, with Auth/private-session controllers and local-player initialization | Establish a public developer boundary so reference visits do not initiate private work. |
| The mobile gallery captures `754ed62`; later responsive fixes are recorded at `9c4c3bb` | Distinguish historical checkpoints, follow-ups and the latest reviewed baseline. |
| Baseline contains 28 reference, 24 working and 48 archive Markdown documents, plus root/media indexes | Audit authority/relevance and current facts; retain useful dated history and incoming links. |

## 3. Proposed navigation and URL contracts

Keep the existing canonical Lab/Visuals URLs and bring them into one developer
navigation. Do not create parallel copies of those viewers.

| Navigation | Route | First-edition role |
|---|---|---|
| Overview | `/dev/` | Explain the area and link to design, experiments, reviews, Play and source. |
| Design & assets | `/visuals/` | Source-backed design/control reference, full icon inspection, sprites and layering. |
| Lab | `/lab/` | Bot ranking/search/analysis reports with clearer orientation and progressive detail. |
| Reviews | `/dev/reviews/` | Curated review index, current screen-reference baseline and dated cases. |
| Process | `/dev/process/` | Architecture context, development/validation/release workflow and its improvements. |

Case pages use `/dev/reviews/<stable-id>/`. The developer brand returns to
`/dev/`; Play returns to the game. Avoid calling both destinations Home.
Game Home gains a deliberate developer entry; existing Lab/Visuals links remain
usable. Rules and Guide retain their player-facing roles.

Use real links for section navigation. Preserve Lab's `tab` and `match` links.
Visuals gains URL-driven Design/Icons/Sprites selection and bounded canonical
icon selection/comparison state. Direct entry, reload and Back/Forward restore
the view. Unknown values fall back safely; unknown developer pages get public
recovery. Provide direct-entry HTML for public routes and curated case URLs.

## 4. Proposed shared visual frame

Start with PixelOperator/PixelOperatorBold. Improve reading measure, line height,
hierarchy and spacing before considering a different body font. Keep established
colours and controls, with less repeated framing. Light/dark asset inspection
surfaces do not introduce a whole-site theme feature.

```text
GOMOKU2D / DEVELOPER                              [ PLAY ] [ SOURCE ]
Overview   Design & assets   Lab   Reviews   Process

Section title
A short statement of what this page helps you inspect
[local views / tools where useful]

Readable explanation or a wider inspector/report workbench
Source / capture context and deeper references where relevant
```

Use one shared navigation/header/content frame. Navigation stays in normal flow
and wraps on narrow screens; no persistent left rail is proposed initially.
Give prose a comfortable measure and workbenches wider space. Define one scroll
owner, skip/focus behavior and usable section anchors. Avoid permanent decorative
status dashboards and repeated metric-card introductions.

The visual study shows Overview, the icon inspector and a representative Lab
view together using actual content, plus one adaptation of the existing review
comparison into that frame. Review desktop, 390px portrait, 320px and enlarged
text. Settle title/body scale, panel rhythm, navigation, density and technical
disclosure placement before implementation.

## 5. Design and asset page contracts

### Design

Palette values derive from real CSS tokens; editorial copy explains their roles.
Font references use actual source files/licenses. Control specimens use app
classes and real focus/hover/pressed/disabled behavior without game/account
side effects. Show representative layout patterns with links to reviewed screens;
do not reimplement Profile or replay business logic for a demonstration.

Keep source identity visible but quiet. Site revision, reviewed game release and
historical capture version can differ and must not be conflated.

### Icons

- Browse all 41 icons by name/category and expose individual deep links.
- Inspect one icon or a small comparison set (proposed maximum four).
- Native 1x/2x samples, integer enlarged view, grid, centre axes, ink bounds,
  margins, light/dark surfaces and exact canonical SVG source/download links.
- Distinguish original sheet cells and standalone additions. Registered/available
  does not establish that a particular app control uses the icon.
- Show actual geometry. Reset, transport and other legacy shapes have unequal
  margins; no automatic glyph redesign or universal equal-margin assertion.

### Sprites and board language

Retain the full sheet/frame/animation/layer inventory. Offer native/integer zoom,
play/pause and frame stepping where useful. Source black/white and overlay colours
from runtime definitions. Reuse the existing pure RGBA tint operation or prove
equivalent pixels; preserve alpha and validate ranges/rates against runtime.
A preview approximation is not engine-renderer acceptance.

Respect reduced motion and avoid offscreen/unselected animation work. These
inspectors need no Phaser game, Wasm computation or new legality model.

## 6. Lab refinement contract

Preserve ranking, telemetry, analyzed games, proof boards, drill-down and links.
Recorded JSON, engine revisions, budgets, dates and bounded analysis meanings
remain unchanged. Do not rerun tournaments for presentation changes.

Improve reading order: question answered → report identity/limits → primary
results → optional metrics/proof → full provenance. Make dense columns and
expansion controls readable on desktop/mobile, with keyboard and error/recovery
behavior. Retain the current DOM proof-board renderer over recorded data.

The study should show a real ranking/search row and expansion, plus enough
analysis content to demonstrate proof-board layout. Historical measurements
must not acquire current-engine labels.

## 7. Review record and viewer contracts

Current screen references and dated reviews have distinct roles. The review
index uses scannable rows: title, scope, date, disposition and a Review action.
The small initial catalog does not need ornamental search/filter controls.

A case leads with the question, comparison and findings. Show before/after side
by side or stacked, full-size inspection and optional synchronized scrolling.
Label application captures, design mockups and asset renders accurately. Different
synthetic sessions/timestamps are not identical capture conditions.

| Record field family | Required meaning |
|---|---|
| Identity | Stable ID, title, scope, date and record revision/source. |
| Sources | Before/after application or asset commit; capture identity when different. |
| Conditions | Fixture/scenario, browser, viewport, text scale and environment/render mode. |
| Media | Allowlisted local files, hashes, dimensions, captions and alt text. |
| Findings | Observed issue, decision/change and fixed/deferred/open disposition. |
| Technical evidence | What ran, on which source, results and deeper evidence links. |
| Human review | Who reviewed what and when; actual acceptance or explicitly pending/unknown. |
| Follow-up | Superseding review or dated correction/addendum, when applicable. |

Missing historical metadata remains explicitly unknown with a reason. Do not
invent a browser/capture identity or promote an incomplete historical record
into the current qualified screen baseline.

Freeze captured pixels and reviewed asset bytes. Later acceptance/corrections
are explicit dated additions. Updating current sources must not silently redraw
old evidence. Technical success is not a human-approved badge.

Initial records: the accepted action-icon review and the 0.6.2 public-player/mobile
review sequence, identifying later fixes. Establish a current 0.6.2 core-screen
baseline from qualified captures or targeted new fixture captures: Home, Match,
Replay, Settings, Profile and Public player at desktop/portrait sizes, with
relevant landscape/enlarged-text cases. Use guest/synthetic emulator data.

Use a small static curated catalog with typed/validated records, following the
published-report pattern. Reuse referenced frozen media where possible. Publish
only selected records/media, not a recursive export of working notes, ignored
outputs, raw transcripts or private report-host links.

New ad-hoc review exports and the hosted viewer should consume the same record
contract and reusable presentation. An export must include its selected media,
styles/fonts/licenses and source context, work under its served prefix and offer
comparison controls without game/account dependencies. Draft exports can use the
existing review-report delivery path; only deliberately curated records join
the public Pages site. Routine private preview links are not public dependencies.

## 8. Process presentation and Markdown reconciliation

First Process content covers actual practice with concrete examples:

1. An architecture map connecting React/Phaser, Rust/Wasm, the native lab, browser
   state and optional cloud continuity.
2. How research/design decisions become bounded implementation and review.
3. How captures, inspections and validation produce findings, then follow-up work
   without losing the original evidence.
4. What unit, rules/SDK, browser, emulator, renderer and profiling checks establish;
   where physical-device and human acceptance remain separate.
5. Release gates, with 0.6.2 as a worked, dated example.

Explain actual ownership: the human directs scope, domain meaning, taste and
acceptance; agents research, implement, inspect and validate. Name the review
performed and its evidence rather than implying that agent review alone proves
correctness. Keep workflow/model/tool descriptions dated where they evolve.

Record workflow improvements as they happen, linked to the relevant procedure
and review. Retrospective article series remain later work.

Audit current references/working notes for authority, relevance and source facts.
Prioritize app/UI/board/assets, architecture, screenshot review, testing and release
procedures. Backend/lab contracts retain detailed Markdown with prepared summaries
where helpful. Mark future proposals, consolidate duplicate responsibilities and
index completed records without breaking evidence links. Preserve historical
measurements and release receipts; do not blanket-rewrite the archive.

## 9. Application and publishing boundary

Same repo/package, React/Vite tooling, domain and Pages artifact. Give developer
routes a shared public layout and lazy entry that does not import/mount private
game controllers/stores. Authentication-aware shared replays and public profiles
retain their existing distinct boundary; public pages have different needs.

Fresh developer visits, even with an existing sign-in, must not initialize a local
player, read/write private history/profile or start Firebase Auth. Developer
reference/report pages need no Phaser/Wasm initialization. Game-to-developer
navigation must not flush/reset data: respect operations already in flight and
preserve normal game/account behavior on return.

Keep route, navigation and published direct-entry inventories consistent. Preserve
existing Lab/Visuals, query links and raw asset/report URLs. Retain the normal
tag-based release/publishing process; no extra host, credentials, deployment target
or automatic public-report channel is needed. Local review exports remain separate
from public publication.

## 10. Implementation checkpoints and acceptance

| Checkpoint | Work | Exit gate |
|---|---|---|
| A: plan review, locally committed | Review page/source/public-boundary contracts and content roster | Agreed scope, with unresolved visual choices named. No application implementation yet. |
| B: visual study, accepted | Overview, icon inspector and Lab compositions plus one review-case adaptation; real content and narrow/enlarged layouts | Bryan reviews the shared look, density and hierarchy before production implementation. |
| C: public shell/navigation | Public boundary, shared frame, URLs and recovery | Direct/reload/history/keyboard paths work, no private-account effects, game/shared replay behavior preserved. |
| D: assets/design and Lab | Full source-backed inspectors and refined report presentation | Asset/token/frame/tint parity, report meanings/data/links preserved, responsive controls usable. |
| E: reviews and Process | Curated records/viewer, initial cases/baseline, process copy and affected Markdown | A new review uses reusable capture/export/render procedures; provenance and acceptance stay accurate. |
| F: overall review/release | Inspect the whole area and run proportional/final release gates | Actual validation and agreed visual pass; remaining human/account checks explicit. Publication separately authorized. |

Check data/media/route contracts, native icon geometry/raster and sprite tint/frame
parity. Exercise Chromium/Firefox/WebKit, 320px/portrait/short-landscape/desktop,
200% text, keyboard, reduced motion and error states. Prove the public boundary
with production-configured and emulator contexts; distinguish fresh direct entry
from navigation with an operation already in flight. Local cloud harnesses always
use offset 1000, preserving the manual preview, Carbon forwarding and demo.
Routine automation requires no user-browser tabs.

Run relevant game regression for shared shell/lifecycle changes, and targeted
presentation checks as slices settle. Markdown/content iteration should not need
Rust rebuilds or cloud runs. Keep the normal exact-source release gate; a broad
CI-system redesign is not a prerequisite.

The main integration risks are shared CSS leaking into player screens, eager
imports bypassing the public-page boundary, stale snapshots appearing current,
and changed report presentation hiding or changing meaning. Mitigate these with
scoped layout styles, built-browser/import checks, explicit capture identities,
and regression checks against unchanged report data and old links. Introduce
the shared frame in a bounded change and verify player surfaces before expanding
the rest of the developer area.

## 11. Decisions reserved for the visual study

- Shared header/local-view navigation, title/body scale and panel density.
- Catalog/inspector split, comparison arrangement and default geometry detail.
- Lab column hierarchy and provenance placement.
- Review framing and technical/human disposition language.

Working defaults: current fonts, wrapping top navigation, the existing 1180px page
family with readable prose measures, no new theme and no global icon redesign.
Revise these through the reviewed study.

## Source entry points

[Visuals](../../gomoku-web/src/routes/AssetPreviewRoute.tsx),
[Lab](../../gomoku-web/src/routes/LabReportRoute.tsx),
[proof boards](../../gomoku-web/src/routes/lab-report/AnalysisProofFrames.tsx),
[global CSS](../../gomoku-web/src/app/global.css),
[icons](../../gomoku-web/assets/icons/README.md),
[asset manifest](../../gomoku-web/assets/asset_manifest.json),
[sprite definitions](../../gomoku-web/src/board/constants.ts),
[RGBA tint operation](../../gomoku-web/src/board/board_sprite_resolver.ts),
[app/controller boundary](../../gomoku-web/src/app/App.tsx),
[icon review](publish_icon_review/README.md), [mobile review](v0_6_2_mobile/README.md),
[UI design](../reference/app/ui_design.md), [app design](../reference/app/app_design.md),
[board design](../reference/app/game_visual.md), [architecture](../reference/app/architecture.md),
[screenshot procedure](ui_screenshot_review.md), [testing](../reference/ops/testing.md),
[release process](../reference/ops/release.md), and
[later process-story material](../archive/process_story/README.md).
