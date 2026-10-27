# UI Screenshot Review

Purpose: current visual QA checklist for screenshots and release captures.

Historical screenshot critique belongs in the archive or release notes. This
file is the active review procedure that complements
[`ui_design.md`](../reference/app/ui_design.md).

## When To Run

Run a focused screenshot review when a release changes:

- match/replay/settings/profile layout;
- board renderer or sprites;
- public static pages;
- report/visual-guide surfaces;
- mobile spacing or controls.

For docs-only/backend-only releases, record that screenshots are unchanged and
skip capture refresh.

## Capture Set

Use current production-style build unless intentionally testing dev behavior:

- Home desktop/mobile
- Match desktop/mobile
- Replay analysis desktop/mobile
- Settings desktop/mobile
- Profile desktop/mobile when profile/history changed
- Rules/Guide/Lab/Visuals only when those pages changed

Prefer consistent mobile viewport height for comparisons. Very tall full-page
mobile captures are useful for scroll audits but poor as release references.
Use measured usable browser dimensions for a named device: CSS viewport,
visual viewport, DPR, display/text scaling and address-bar/full-screen state.
A resized desktop viewport is a generic layout reference, not Android-device
validation. Keep landscape and large-text stress checks secondary to the normal
screen gallery; label their differing conditions explicitly.

## Review Checklist

- Board remains dominant on match/replay screens.
- Controls stay reachable on portrait mobile.
- Replay analysis timeline/status does not push controls out of view.
- Hint markers are readable against the board.
- Static pages share nav, typography, panel rhythm, and width with the app.
- Report tables remain readable without horizontal overflow.
- Version labels and screenshots match the intended release.

## Asset Policy

Keep current release media under `docs/assets/`. Dated review/reference media
stays frozen at its indexed evidence path. Do not move or delete files used by
a curated record without a deliberate migration that preserves bytes and links.
Unselected working drafts can be archived separately.

## Curated Records And Reusable Exports

Use the [developer-area contract](../reference/app/developer_area.md) for the
record schema and application boundary. The web area is prepared presentation;
Markdown keeps detailed maintainer/agent context and deeper evidence.

1. State the source, question, viewport/text scale, browser, renderer/environment,
   fixture and exact before/after relationship before capture. Use guest or
   disposable emulator data. Existing manual services require the offset-1000
   cloud lane; do not replace its demo or use real private data in a gallery.
2. Capture a production-style build on an owned endpoint. Record source/build
   identity, date, image dimensions/hashes and observed conditions. A missing
   observation stays unknown with a reason. Prefer matching content and viewport;
   label differing conditions instead of implying an identical comparison.
3. Freeze media in a dated working/evidence directory. Write an explicit typed
   record in `gomoku-web/src/developer/reviews.json`: comparison, conditions,
   media identity, findings, technical checks, actual human review and follow-up.
   A current screen reference and a historical fix have distinct dispositions.
4. Run type/unit/build checks. The publisher checks the curated allowlist, safe
   paths, SHA-256 and dimensions. Inspect the default full captures at matched
   scale: source widths determine pane proportions, so desktop and mobile
   content have the same visual scale. Check native pixels and optional bounded
   scroll panes separately. Review narrow/enlarged text, keyboard, synchronized
   scrolling and source/error recovery. Display scaling must not alter frozen
   media or imply that enlarged-text captures used the same text setting.
5. Prepare an ad-hoc export from the same record/viewer with `npm run review:export
   -- <id> test-results/review-exports/<new-name>` in `gomoku-web`. The new directory
   includes selected media, fonts/license and source context. Check it under its
   actual served prefix before sending a review link through the shared report
   publisher. For the complete developer preview, use `npm run developer:export
   -- test-results/developer-exports/<new-name>` instead. Its hash routes and
   runtime asset base work under the immutable report prefix, without a separate
   preview port or device-specific forwarding. Never export the whole working-notes tree.
6. Record acceptance and later corrections explicitly. Do not redraw old pixels
   from new sources or convert an automated pass into a human-approved badge.
   Curated public publication goes through the ordinary separately authorized
   release; a private report export is not a Pages deployment.

For a complete process example, the
[released 0.6.2 reference](v0_6_3_core_reference/README.md) records a detached
release checkout, six player screens, exact fixture/layout identities and media
hashes. The [0.6.3 execution record](v0_6_3_execution.md) records integrated site,
standalone-export and private-boundary validation separately.
