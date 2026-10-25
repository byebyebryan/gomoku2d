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
