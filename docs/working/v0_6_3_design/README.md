# 0.6.3 developer-area visual study

October 24, 2026. Owner: Bryan. Implements checkpoint B of the
[developer-area brief](../v0_6_3_developer_site_plan.md), as a standalone study.
Application integration, public Pages publication and human visual acceptance
are pending. This is a design proposal, not the shipped developer area.

[Open the interactive study](http://starship:8787/reports/3a2d77f9d8145ae00cf9b67c865e0f406f79f2f6764de2fc3a37ed6301772ada/)
on Starship's shared report host; verified reachable from Carbon. This internal
review URL is not a dependency of the proposed public site.

## What to review

| Surface | Concrete question |
|---|---|
| Overview / shared frame | Do the wrapping navigation, pixel type, flat separators and spacing feel like the same product? |
| Design & assets | Search all 41 icons; compare Share / Save / Publish; add or remove up to four. Do native samples, grid, margins and scoped light surface make inspection useful? |
| Style / sprites | Inspect actual control classes and runtime tokens. Step source frames, change tint, then play an animation. Is the hierarchy usable without an engine? |
| Lab | Open a ranking row, Search and the three recorded Analysis frames. Does the summary help while keeping dated settings and detailed meanings available? |
| Reviews | Compare Profile and Public player, switch fitted/full captures and synchronized scrolling. Are findings and follow-up/acceptance status easy to distinguish? |
| Process | Does the practical workflow, architecture and worked release provide useful orientation? Longer journey articles remain later work. |

The study uses hash state so sections, views and comparisons survive reload and
Back/Forward. These are prototype URLs; the approved brief proposes the real
`/dev/`, `/visuals/` and `/lab/` routes. Scope/source details are available in the
footer. No login, private history, engine, analytics or external resource load
is required. Ordinary outbound source/Play/report links navigate only when used.

## Source and historical identity

The export was built on planning commit `935fa3b` with unchanged application
sources. Its exact inputs/output hashes are in [source_manifest.json](source_manifest.json).
The authoring worktree's study files are qualified by their individual hashes;
`source` names the base commit, not a claim that this study was already in it.
Links to existing repository evidence use published `4052d94`, with byte parity
checked during the build, so they resolve before this local work is pushed.

- All 41 canonical SVG downloads are byte-for-byte source copies. Geometry is
  integer 24 × 24; actual legacy margins are shown without normalization.
- Current CSS provides tokens and actual action-control classes. Only its three
  font URLs are rebased. Three original font files and their license are bundled.
- Seven sprite sheets retain original bytes. The 22 animation ranges/rates are
  checked against board constants. This Canvas preview multiplies source RGB by
  the selected tint and retains alpha; it does not test Phaser.
- Lab is the recorded June 1, 2026 Renju tournament: 15 × 15, eight configurations,
  64 games per pair, pooled search budget. No experiment was rerun. Analysis uses
  one recorded 28-move entry and frames before moves 15, 25 and 28; these display
  14, 24 and 27 earlier stones. Forced lines describe the recorded model, not a
  full-game solution. Detailed run identities remain in the study.
- Review media is the October 23–24 mobile checkpoint (`ce5335d` before,
  `754ed62` after), with matching guest/synthetic scenarios and unchanged PNG
  hashes. Later responsive fixes at `9c4c3bb` are explicit. These are historical
  application captures, not a current final 0.6.2 baseline. Initial feedback was
  broadly positive; physical-phone acceptance remains unperformed.
- Process cites actual 0.6.2 release evidence. Its signed-in sharing acceptance
  remains pending, independently of the deployed automated checks.

## Validation and reproducibility

[validation.json](validation.json) records browser results and delivery evidence.
All testing used owned headless browsers; no user-browser tabs were opened.
The export is deliberately limited to 62 files, not the repository or notes tree.

Build from repository root:

```sh
rtk proxy python3 docs/working/v0_6_3_design/build_study.py
```

Serve `gomoku-web/test-results/v063-design/export` on an owned loopback port.
Run the checked-in Playwright driver with `STUDY_BASE_URL` (trailing slash),
`PLAYWRIGHT_BROWSERS_PATH` and optional `STUDY_CHECK_OUTPUT`. The validated setup
uses the existing `mcr.microsoft.com/playwright:v1.59.1-noble` Docker image,
existing cached browsers, host network and the repository mounted at `/workspace`.
No dependencies were added. The driver imports this package's Playwright.
The published copy passed Chromium, Firefox and WebKit: 64 responsive/text-size
cases per engine (192 total), history, keyboard, motion, frame/tint and native
raster checks. No browser errors or automatic external requests were observed.

Native raster checks compare Share's foreground to its canonical mask at 1×,
2× and 10× in every engine, allowing only the extra screenshot row/translation
caused by a fractional element position. Every foreground pixel must match;
antialias colours are rejected. All icon source geometry and download hashes
are checked separately by the builder.

The first pass found a genuine WebKit overflow from a long word at 320px with
200% text, plus a fixed-width finding label. Wrapping and intrinsic label sizing
corrected those. The initial keyboard driver incorrectly expected the skip link
after an in-page section transition had focused the new heading; the check now
starts from a fresh document. Reflow checks wait for the requested section; published-image checks also wait
for decoding rather than treating element visibility as completed image loading.

This evidence covers the standalone study. Production lazy-loading/privacy,
existing deep-link compatibility, shared-CSS regression, current screen baseline,
typed review catalog, Markdown reconciliation and release gates belong to later
implementation checkpoints. Physical touch, assistive technology and Bryan's
visual decision are not replaced by browser automation.
