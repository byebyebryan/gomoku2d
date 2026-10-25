# UI history: presentation and layout study

October 25, 2026. Owner: Bryan. Standalone design exploration following the
[retrospective capture pilot](../v0_6_3_retrospective_capture.md).
Bryan wants the history from 0.2 through 0.6, including every release with a
meaningful UI change, and neighbouring comparisons as the main way to inspect
it. He requested a design pass before expanding capture or integrating a new
history interface. None of these layout options is accepted yet.

**[Open the interactive layout study](http://starship:8787/reports/f06db4103bfc9b1e3d9418f64d17c641784506f6e1f2caee0160f7cdbb59fabc/).**
All 32 published files were fetched from Carbon and matched the export hashes.
The report and browser qualification are recorded in [validation.json](validation.json).
The authoring files live here; the builder
exports only intended assets under `gomoku-web/test-results/v063-history-design/export`.
This study does not change the app, existing preview or public Pages site.

## Shared structure

Retain the developer area's existing fonts, tokens, tactile controls, flat
separators and navigation. Within UI review, propose three sibling destinations:
Current screens, UI history, Design reviews. Current released references stay
easy to reach; historical evolution gets a dedicated browsing surface rather
than an increasingly long list of unrelated review records. Existing case URLs
remain valid. Exact new routes and integration are still implementation design.

History leads with captured screens, release/version identity and a short
description of what changed. Explain a decision when the original review or
design record supports it; do not invent a decision narrative from screenshots.
Keep build/source/fixture details in secondary disclosures. New retrospective
captures must be distinguishable from original-period evidence and mockups.

## Alternatives

| Option | Layout and interaction | Tradeoff |
|---|---|---|
| A: Change journal | Five phase anchors and chronological release rows. Open a neighbouring comparison directly after its milestone; all changed screens are listed with jump links. | Strong overview and change context, but a longer page with more scrolling between distant steps. |
| B: Screen history | Visible screen buttons lead to a chronological capture trail. Each capture appears once. A chosen neighbouring step opens a focused comparison below the trail. | Best for seeing a screen evolve; provides less context about simultaneous changes elsewhere. |
| C: Milestone browser | A compact grouped milestone list sits beside a comparison gallery. Previous/next positions stay stable. The rail becomes a disclosure on mobile. | Fast repeated inspection; gives less width to images and introduces a scrolling rail on desktop. |

Initial recommendation: A as the chronological entry with B available for
screen-focused exploration. This is a proposal for Bryan to review. Do not ship
three independent production viewers by default. C tests whether quicker
switching is valuable enough to justify its navigation and width cost.

All options expose the same content rather than merely changing colours or
card borders. Portrait/Desktop controls switch actual source files; selected
layout/version/screen/viewport survive reload and browser history. Navigation
and comparison activation move focus deliberately. Changed-screen lists use
direct links, avoiding a dropdown that hides the available views.

## Content and capture model

The [proposed milestone roster](milestones.json) has 20 entries across 0.2–0.6,
drawn from the changelog. It establishes realistic navigation density, not the
final inclusion audit. Review actual source diffs before deciding whether to
include smaller 0.4 report changes or the 0.5 reconciliation patches separately.
Do not treat a lab-heavy release as having no UI changes without inspecting its
report surfaces. Conversely, a release number alone does not justify repeating
unchanged screen captures.

The eventual model needs three related identities:

- **Version snapshot:** exact tagged build, environment and capture condition.
  Store each source capture once, with its provenance and original bytes.
- **UI change:** previous relevant baseline, new version, affected screens,
  concise description and original review/design evidence where available.
- **Screen trail:** only that screen's meaningful changes. Its previous step
  can differ from the previous milestone in the overall history.

Compare neighbouring meaningful milestones by default. Arbitrary distant
comparisons can remain secondary. A newly introduced screen has an explicit
introduction state instead of a fabricated before image. Missing capture
coverage is a different state. Preserve the original 0.2.4→0.6.2 pilot and its
existing URL, while making the complete sequence the main history presentation.

Capture coverage must expand beyond the pilot's four idle guest screens:
signed-in/import/reset/sync Profile states for 0.3; Settings for 0.4.5; meaningful
replay analysis positions and overlays; Lab/report and Visuals changes; sharing
panels, saved records and public player pages for 0.6. Common scenes should use
matched synthetic fixtures and consistent CSS viewports. Phone calibration
remains separate. Do not infer that idle guest captures show all relevant UI.

The study bundles 21 existing PNGs without alteration:

- 16 qualified retrospective pilot images from exact v0.2.4 and v0.6.2 sources.
- Four 0.6.2 release reference images: Public player and Settings, each in
  desktop and portrait. These are the earlier generic Chromium references.
- One 0.6.1-era public-profile development image. Its exact source commit is
  unestablished here. It and the 0.6.2 public reference use different synthetic
  content; they illustrate the layout, not standardized release parity.

Each figure labels its actual source status. Explicit empty frames occupy
uncaptured milestones rather than relabelling another version's screenshot.
These placeholders are a study aid, not a proposal to publish an empty history.
The final site should expose qualified coverage and honest gaps without a wall
of empty panels. Source manifests and original MD remain detailed agent context.

## Reproduce and inspect

From repository root:

```sh
rtk proxy python docs/working/v0_6_3_history_design/build_study.py
```

Serve the explicit export on an owned loopback port. The checked-in browser
driver defaults to `http://127.0.0.1:46865/`; override `HISTORY_STUDY_URL` and
`HISTORY_STUDY_CHECK_OUTPUT` for an independent run. It imports the repository's
Playwright and uses installed cached browsers. The qualified setup runs in the
existing Ubuntu Playwright Docker image with the repository and browser cache
mounted, host networking and user 1000. No dependencies or user tabs are added.

```sh
rtk proxy node docs/working/v0_6_3_history_design/check_study.mjs
```

The browser pass covers all three layouts at desktop, 390px, 320px and
320px/200% text across Chromium, Firefox and WebKit. It checks paired image
dimensions and display scale, page overflow, source switching, milestone/screen
navigation, direct entry, history/reload, keyboard skip/focus and mobile rail
behaviour. Automated accessibility is bounded evidence; human visual choice,
phone and assistive-technology acceptance remain separate.

The first pass found that the rail could stay closed when resizing from mobile
to desktop. A breakpoint listener now updates its open state, and the driver
checks the transition. Header controls were also shortened after visual review
so the study does not spend excessive space before the actual history.
Large-text checks caught fixed study-option columns overflowing their labels;
the grid now wraps according to text size. Focus checks also caught duplicate
heading targets, which now have distinct identities. The driver waits for
hash-driven view changes and font/layout readiness before assertions.

The final pass succeeds in all three engines: 36 layout/width/text combinations
plus the navigation, source-switching, history, reload and keyboard journeys.
No horizontal overflow, duplicate targets, page/console errors or automatic
external requests were observed. Automated accessibility reports zero violations
and ten incomplete contrast checks requiring manual judgment. The agent visually
reviewed all three desktop and portrait compositions. These results qualify the
standalone study, not human selection or physical-device acceptance.
The [source manifest](source_manifest.json) identifies the exact export inputs
and bytes. Temporary preview hosting was closed; the shared report host and
existing app/manual preview services remain available. No user tabs were opened.

## Next decision

Review the three actual compositions and choose the primary browsing model.
Then settle the milestone inventory and scene coverage, adapt the reusable
capture harness, and backfill neighbouring snapshots in manageable batches.
Implementation of the selected interface follows that design choice. This pass
does not authorize a production release, push or bulk historical recapture.
