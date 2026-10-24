# v0.6.3 developer site and process presentation

Owner: Bryan. Direction agreed October 24, 2026, before the 0.6.2 release.
This records accepted intent and scope. Framework, repository/site arrangement,
URL and publication details remain to be settled; no site implementation is
part of this planning commit or the 0.6.2 release.

## Purpose

Gomoku2D is built with AI under human direction, with deliberate research,
design, implementation, review, validation and release work. The workflow
continues to evolve. Record and present that work better first; retrospective
journey stories can then draw on dependable development evidence.

The first site is an organized reference and review workbench for work already
being done. It should improve our next iteration as well as help visitors
understand how the work is conducted. Long-form journey articles come later.

## Documentation responsibilities

| Surface | Responsibility |
|---|---|
| Repository Markdown | Detailed agent/maintainer context: current contracts, architecture, procedures, decisions, evidence and history. Clear authority, status and links matter. |
| Developer site | Distilled human presentation: visual reference, demonstrations, review galleries and explanations of the development/validation workflow, linked to source and evidence. |

Reconcile current claims against code and shipped behavior. Remove duplication,
separate references from proposals/history, and index useful detailed material.
Preserve incoming evidence links, source identities and useful history. Age or
the model that wrote a document alone is not a reason to replace it. Keep the
root README compact, operational and linked to the live game and presentation.

## Initial site scope

| Area | Content |
|---|---|
| Design system | Current palette, typography, spacing, buttons/toggles and states, panels, responsive patterns and actual app examples. |
| Assets | Full icon inspector, sprites/animations, board layers, colour rules and source/download references; expand existing Visuals material. |
| UI review | Organized screen/state galleries, desktop/mobile and before/after comparisons, concise findings and accepted/pending decisions. |
| Development and validation | Research → implementation → review/fix → release, reproducible capture/report procedures, checks and their limits, human acceptance and workflow improvements. |

The whole-set icon inspector is an early concrete deliverable: native 1×/2×
samples, enlarged integer pixel grids, centre axes, margins, light/dark surfaces
and selection of icons for comparison. Generalize the existing useful ad-hoc
review instead of assembling a special page for every new glyph.

Build a reusable UI-review format from existing galleries/reports. Preserve
viewport/browser, source/capture version, conditions, findings and disposition.
Browser assertions, primary visual inspection and Bryan's acceptance remain
distinct evidence; automated results are not physical-device or accessibility
acceptance.

## Source and maintenance rules

- Derive current references from canonical icons, assets and tokens where
  practical. Avoid independent copied packs, tokens or game-rule models.
- Historical reports remain dated snapshots of the reviewed source. Updating
  current icons must not silently change an old accepted comparison.
- Use committed reviewed content and reproducible exports. Raw transcripts,
  private sessions and uncurated test artifacts are not public site content.
- Keep game/player routes useful. Plan navigation/reuse of Lab and Visuals;
  creating the site does not itself authorize breaking or removing those routes.
- Prefer static hosting/browser presentation with no required backend service,
  cloud writes, polling or per-move requests.

## Inputs and checkpoints

Start from [UI design](../reference/app/ui_design.md),
[app design](../reference/app/app_design.md), [board visuals](../reference/app/game_visual.md),
[assets](../../gomoku-web/assets/README.md) and the
[Visuals implementation](../../gomoku-web/src/routes/AssetPreviewRoute.tsx).
Reuse [canonical icons](../../gomoku-web/assets/icons/README.md),
[pixel comparisons](publish_icon_review/README.md), the
[screenshot procedure](ui_screenshot_review.md), [profile design study](v0_6_2_design/README.md),
[review](v0_6_2_review/README.md), [mobile gallery](v0_6_2_mobile/README.md),
[candidate evidence](v0_6_2_candidate.md), [testing](../reference/ops/testing.md)
and [release process](../reference/ops/release.md).
The [process-story archive](../archive/process_story/README.md) is later
editorial input, not the first site backlog.

1. Inventory references, assets, generators and reports. Identify stale claims,
   duplication and reusable pieces. Review navigation, source/delivery boundaries
   and a bounded Markdown cleanup before implementation.
2. Review an actual icon-inspector and UI-gallery prototype using current
   canonical icons and existing 0.6.x captures; settle reusable content contracts.
3. Build the accepted design/asset and development-process sections, reconcile
   affected Markdown and document capture/update procedures.
4. Validate links/assets, canonical source parity, historical snapshot identity,
   responsive/keyboard behavior and static delivery. Review the actual site.

Success: inspect the current icon set and design rules, find a screen comparison
and its decision, understand what checks ran and reproduce a new report without
assembling another one-off page. Visitors can follow that same evidence to
understand how the AI-driven work is conducted carefully.

## Sequence and boundaries

Finish 0.6.2 first. This site/documentation work becomes 0.6.3. Replay collection
and metadata move to 0.6.4 with their compatibility gates retained. Curated
learning follows later without a release number yet. Multiplayer stays deferred.
No new gameplay, replay schema, article series or complete historical-document
rewrite is required by the initial site.
