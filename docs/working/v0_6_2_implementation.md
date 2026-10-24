# v0.6.2 public-profile implementation checkpoint

Owner: Bryan. Started October 23, 2026 from `03765d3`.
Status: public-profile technical checkpoint complete; Bryan's application
visual review and the remaining 0.6.2 work follow. Publication is separate.

## Accepted design and scope

Bryan accepted the final [public player-card study](v0_6_2_design/README.md)
and requested a Luna goal loop for implementation. The accepted composition is:

- existing app-style Public profile header with Play, My profile and Home;
- a natural-height left card with public name, shared-game count, actual
  profile address/Copy link and the existing record disclosure;
- one right panel with the optional dated statistics above the paged games;
- compact mobile identity/count/copy, with extra profile details collapsed
  initially, and no empty statistics placeholder when sharing is off.

This is the public-profile implementation checkpoint within the
[0.6.2–0.6.4 plan](v0_6_followup_plan.md). Broader 0.6.2 player help, static
examples, documentation consolidation and release media remain separately
sequenced. The version remains the published `0.6.1` until release preparation.

## Behavioral contract

Use the existing current/legacy public records. Count shared games from the
manifest or the already-required legacy page reads; do not infer a record from
one visible page or fetch the whole catalog for a new summary. Keep My games
statistics independent, opt-in and explicitly refreshed. Dates remain snapshot
dates; no played date, rating, avatar, bio or thumbnail is added.

Public entry remains isolated from private cloud history/library controllers.
Viewing a replay does not save/import it. Saving stays explicit and a finished
continuation remains an ordinary My game. Preserve schema, strict parsing,
generation/revision validation, bounded document reads and error recovery.
There are no data-model, rules, index, dependency or service changes.

Profile-link copying must use the actual public UID and application base path,
with a usable HTTP/clipboard-denied fallback. Preserve complete accessible
names while containing long Unicode labels visually. Previous and Next keep
stable positions, including the last page.

Catalog Replay provides an explicit return to the originating public profile.
Both that action and ordinary browser Back retain logical page, scroll and
originating Replay focus. Use bounded, validated route-owned navigation state;
construct only a known internal player route from the UID/page, never an
arbitrary return URL. Copied standalone replay links retain their normal
independent behavior. Clamp a stale page to the remaining catalog and handle
disabled/changed data honestly; do not restore another UID's view state.

## Work and verification

Luna Max implemented the visitor UI, shared-viewer return integration and
navigation helper tests. The primary owned component/browser regressions,
integration corrections, final checks, evidence, local commits and preview.
Owner settings retain their existing behavior and styling.

Use the repository's proportional testing gate:

1. Full web typecheck and units, plus production build/direct public routes.
2. Three-engine guest/local smoke and changed replay/profile journeys.
3. Emulated C10/C11/C15/C16 and focused return-navigation scenarios, protecting
   private-read isolation, publication lifecycle, Unicode/reflow, sparse pages
   and legacy records.
4. Inspect actual desktop, portrait, 320px and enlarged-text captures, statistics
   on/off, empty/sparse/full and error states. Distinguish automated accessibility
   checks from human/device acceptance.
5. Verify final source/build identity, review the complete diff and update the
   manual Starship preview with byte parity and unchanged demo records.

All cloud harnesses use `GOMOKU_CLOUD_PORT_OFFSET=1000`. Ordinary Playwright
commands must use an explicitly managed free preview port and
`PLAYWRIGHT_BASE_URL`; unrelated port 8001 belongs to another application.
Keep captures headless and preserve all manual emulators, account data and the
Carbon tunnel. Preserve a copy of the prior manual build before refresh.

Local commits are authorized. Push, tag, production deployment and real-account
acceptance remain separate. The earlier 0.6.1 manual sharing/device handoffs
are not automatically closed by this UI checkpoint.

## Results

The accepted player card is implemented with existing data and controls.
The record strip, divided eight-row lists, stable pager and responsive identity
replace the old centered card. Public replay returns retain page, activation-time
scroll and trigger focus; missing entries fall back to the public name. Root
and page errors have explicit recovery. Clipboard denial opens and selects a
canonical URL after the mobile details section is visible.

Review corrected automatic retries after an initial page failure, stale-root
reload recovery, late page responses, scroll captured before rather than at
activation, portrait document scrolling, narrow metadata overlap, accessible
button naming and return-header overflow. At 320px Copy uses the established
icon-button treatment to leave room for the public name.

- Full web typecheck and **503 unit tests in 50 files** passed.
- **93** regular Chromium/Firefox/WebKit checks passed for smoke, private
  Profile, shared replay, saving/continuing and hostile/large replay labels.
- The six selected cloud journeys **C10/C11/C15/C16/C17/C18** passed in all
  three engines across the recorded runs. They cover private-read isolation,
  publication lifecycle, Unicode/reflow, sparse/legacy pages, explicit and
  native replay return, removed entries, changed manifests, empty/unavailable
  profiles and clipboard fallback. The final focused reruns validate corrections;
  this is not a claim that the earlier exploratory runs all passed.
- Production and emulator-configured builds completed, including static assets
  and direct-route entries. No dependency, schema, rules, service or index changed.
- The real manual preview passed **15** additional layout/desktop-return checks
  across three engines at 1440px, 390px, 320px and 640px with 200% root text.
  Source captures were inspected; automated axe checks remain bounded evidence,
  not human accessibility or physical-device acceptance.
- Preview bytes and Carbon's forwarded HTTP endpoints were verified. The
  existing demo root and four page documents retain their original hashes.
  Old build files are backed up and older hashed chunks remain served.

The [evidence ledger](v0_6_2_evidence/README.md) records sources, captures,
commands, preview identity and recovery boundaries. On Carbon open
**[Demo Player](http://127.0.0.1:8013/player/?id=v061-demo-player)** through the
existing tunnel. This is an emulator-backed local preview; production remains
the published 0.6.1.

Bryan's next pass is the real layout, mobile density, profile details and replay
return. Broader owner/action clarity, player help, static examples, documentation
consolidation and release media remain in the follow-up plan. Existing real-account,
phone/touch and human accessibility handoffs stay open. No push, tag or production
deployment occurred.
