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

## Review Checklist

- Board remains dominant on match/replay screens.
- Controls stay reachable on portrait mobile.
- Replay analysis timeline/status does not push controls out of view.
- Hint markers are readable against the board.
- Static pages share nav, typography, panel rhythm, and width with the app.
- Report tables remain readable without horizontal overflow.
- Version labels and screenshots match the intended release.

## Asset Policy

Keep only current release-relevant screenshots under `docs/assets/`. Old visual
history should move to archive or be deleted once it no longer explains a
current design decision.
