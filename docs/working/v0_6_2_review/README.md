# Public-profile implementation review

October 23, 2026. Reviewed local `895ea8a` against the accepted player-card
design and sharing contracts. Both confirmed findings are fixed in `c0cf856`.
This is the public-profile checkpoint review, not the full 0.6.2 release gate.

The subsequent [mobile layout pass](../v0_6_2_mobile/README.md) owns the newer
visual preview; this review retains its original navigation evidence.

## Findings

| Priority | Reproduction and impact | Resolution |
|---|---|---|
| P2 | Open a page-two replay, Return, go to the previous profile page, then use browser Back. Restoration remembered the old history key as already handled and could also run against the previous page's rows. The correct page appeared, but originating focus/scroll was lost. | Reset restoration on each history transition and wait until the rendered page matches the URL. Added a failing-then-passing component regression and repeated Back/Forward coverage in C17. |
| P2 | Open a replay from desktop, switch to portrait, then Return. The saved pixel offset no longer matched the row's position. The Replay control was focused below the viewport in all three engines. | Restore the saved offset, then use nearest-edge scrolling to keep the target visible when layout changes. Extended C17 with the desktop-to-portrait return. |

Before the resize fix, the target's top was approximately 1368px in Chromium,
1493px in Firefox and 1400px in WebKit, below the 844px viewport. Afterwards its
top is approximately 800px in each engine and its 44px button is visible.
The check allows one CSS pixel for native scroll rounding; Firefox exposed a
0.28px difference in the first strict assertion. This tolerance does not mask
the original offscreen result.

Compare [before](return-after-resize-before.png) and
[after](return-after-resize-after.png). These are the same public demo entry
after desktop-to-portrait navigation, not new seeded manual data.

## Overall assessment

No further blockers were found in the reviewed scope after the fixes.

- Layout, controls, fonts/icons, natural-height identity card, dated optional
  record, stable pager and mobile details follow the accepted composition.
- Name/count/snapshot semantics remain distinct. Statistics off, empty catalogs,
  unavailable profiles, malformed entries and changed manifests retain honest
  states and explicit recovery.
- Profile URLs stay canonical. Replay return accepts bounded public UID/page
  context; scroll/focus stay in history state. No arbitrary return URL is used.
- Current and legacy page reads remain bounded and cached. The fixes add only
  DOM restoration, not queries, listeners, polling or writes. Rules, indexes,
  cloud/storage modules, dependencies, replay formats and Rust were unchanged.
- Public/private controller isolation, explicit saving, publication lifecycle
  and ordinary continuation behavior retain their existing boundaries.
- The additional responsive probe covered portrait and landscape, widths around
  720px/980px breakpoints, and 200% text. It found no horizontal overflow,
  rule/move overlap or page errors. The unavailable layout was also inspected.

## Validation and delivery

- Full web typecheck and **504 tests in 50 files** passed.
- Production and manual-emulator builds passed, including static/direct routes.
- All **18 distinct selected cloud cases** passed across three engines:
  C10/C11/C15/C16/C17/C18. The broader run passed 17/18; its sole failure was
  the subpixel assertion described above. The corrected C17 rerun passed 3/3.
- The final served preview passed **33 additional checks** across Chromium,
  Firefox and WebKit: resize return, paging/history focus, eight responsive
  layouts per engine and runtime-error checks. The JSON retains before/after
  measurements rather than presenting the initial reproductions as passes.
- All **145** final manual-build files match served bytes. Carbon's forwarded
  index/entry hashes match Starship. The demo root and four page documents
  retain their original hashes; prior build files are backed up.

[evidence.json](evidence.json) records exact source/capture hashes, preview
identity, probe measurements and log mapping. Raw artifacts remain ignored in
`gomoku-web/test-results/v062-review/`. All cloud harnesses used offset 1000;
the manual preview/emulators/tunnel and unrelated port 8001 were preserved.
No user-browser tabs were opened. Commits remain local.

**[Updated Carbon preview](http://127.0.0.1:8013/player/?id=v061-demo-player)**.
The original [implementation ledger](../v0_6_2_evidence/README.md) is retained
as evidence of its earlier snapshot; this review owns the subsequent fixes.

Bryan's application visual pass, physical phone/touch and human accessibility
acceptance remain separate. The earlier real-account sharing handoff remains
open. Owner/action clarity, help, examples, documentation consolidation and media
remain in the [0.6.2 plan](../v0_6_followup_plan.md). Production remains 0.6.1;
this review performs no push, tag or production deployment.
