# Mobile layout pass

October 23, 2026. Baseline `ce5335d`; runtime/test implementation `754ed62`.
Bryan requested a mobile pass, noting that the 0.6.1 mobile presentation had
not received his manual review. This checkpoint covers those sharing/profile
flows and the new 0.6.2 public player. It does not mark human visual acceptance
or the full 0.6.2 release complete.

**[Open the before/after mobile gallery](http://starship:8787/reports/104645db902f603145c3045b8ad9ee107fbee1c921ece6f86753f0a23a536142/).**
The gallery also includes Share, Publish and owner-public-settings captures.
It is a static report on the existing server; the application is separate.

## Changes

- Mobile Profile keeps the editable name, account badge and sign-in/out action
  visible. Secondary cloud information and Reset controls use Profile controls.
  Errors expand that group. Wide layouts retain expanded controls.
- My games, Saved replays and Public profile share one stable tab row. On a
  390px screen the tabs start at 220px, versus approximately 333px before.
- Saved-replay library actions use consistent full widths; each entry's Inspect
  and Remove actions share a row. Full accessible labels remain intact. The
  My games sync badge is shown with its own tab instead of unrelated records.
- Public games use compact metadata and the established 48px mobile Replay
  icon action. Long titles/opponents retain complete accessible/title text.
- Short-landscape Replay keeps board and transport together, with a scrollable
  control pane. The rule covers 560–980px wide landscape viewports up to 500px
  high; 667×390 and 844×390 were exercised. Portrait keeps its existing stack.

The new rotation/enlarged-text checks found two additional presentation defects:
native details could remain closed on entering desktop, and four fixed summary
columns could overflow at 320px/200% text. Details now have explicit responsive
modes, and summary columns reflow with their available width and text size.

The first auth-driver attempt assumed a popup, then assumed redirect in every
touch-emulated engine. The driver now follows the observed touch/coarse-pointer
environment. Application auth policy was unchanged; the Chromium touch journey
exercised redirect. This is not a physical-phone OAuth acceptance claim.

## Verification

- Full web typecheck, **504 unit tests / 50 files**, and production/manual builds
  passed. No dependencies, icons, renderer, storage, rules or indexes changed.
- **96 distinct ordinary browser cases** passed across three engines. The first
  run passed 93/96 and exposed the responsive defects above; the corrected
  compact-profile case then passed 3/3. It covers name accessibility, account
  controls, cancellation, rotation, header overlap and 200% text.
- **27 cloud cases** passed in the broader three-engine regression selection:
  M01, C10/C11/C12/C14/C15/C16/C17/C18. The final M01 rerun passed 3/3 after
  the responsive corrections. Coverage includes signed-in/guest records,
  saving, publishing, library sync, reset/delete guards, Unicode, public/private
  isolation, sparse/legacy pages and replay return.
- The gallery's four comparisons, image loads, full-height toggle and narrow
  rendering were checked in Chromium with no page errors. All **15 gallery
  files** match served bytes, and Carbon can reach its hostname URL.
- All **145 manual-build files** match the refreshed preview. Carbon's forwarded
  index/entry hashes match Starship. The demo root plus four catalog documents
  retain their original hashes. Old chunks remain available for existing tabs.

[evidence.json](evidence.json) maps results to raw logs, source/capture hashes,
publication and delivery receipts. Raw artifacts and prior preview backup stay
ignored under `gomoku-web/test-results/v062-mobile/`. Every cloud harness used
`GOMOKU_CLOUD_PORT_OFFSET=1000`; manual emulators/data and port 8001 were preserved.
No user-browser tabs were opened. Application commits remain local.

## Capture provenance and authoring

Profile and landscape before/after pairs use matching guest/game content. Public
player uses the preserved 18-game demo. Saved-replay samples use equivalent
synthetic sessions with differing timestamps; its earlier image already includes
the first compact Profile iteration. Other owner forms use synthetic accounts
in the isolated emulator. Captures are application renders, not design mockups.

The source gallery is [gallery.html](gallery.html). Prepare the intentionally
small export with:

```sh
rtk proxy python3 docs/working/v0_6_2_mobile/prepare_export.py
```

The export contains the HTML, selected PNGs and licensed runtime fonts. Changes
publish to a new immutable report URL. The earlier
[public-player design study](../v0_6_2_design/README.md) and
[implementation review](../v0_6_2_review/README.md) retain their own snapshots.

## Manual handoff

On Carbon use **[Profile](http://127.0.0.1:8013/profile)** and
**[Demo Player](http://127.0.0.1:8013/player/?id=v061-demo-player)** through the
existing tunnel. Check Profile controls and all three tabs, compact list rows,
Share/Publish opening and return, and Replay rotation. On a physical phone also
check keyboard entry, browser chrome and the sign-in round trip.

Human visual/phone/touch/accessibility acceptance and the earlier real-account
sharing handoff remain open. Remaining 0.6.2 help, examples, documentation
consolidation and release media stay separately sequenced. Production remains
0.6.1; this pass performs no application push, tag or production deployment.
