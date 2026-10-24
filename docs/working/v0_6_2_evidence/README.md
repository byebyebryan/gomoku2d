# Public-profile implementation evidence

October 23, 2026. Local source: implementation `865988d`, browser recovery
coverage `50ebe23`, narrow identity refinement `152674c`. The
[implementation checkpoint](../v0_6_2_implementation.md) owns scope and the
remaining manual/release boundaries. [evidence.json](evidence.json) records
source/capture hashes, browser results and served-build identity.

The [subsequent implementation review](../v0_6_2_review/README.md) records two
additional navigation fixes and the latest preview identity. This ledger and
its captures retain the original implementation snapshot.

## Inspect the application

On Carbon, through the existing tunnel:
**[Demo Player](http://127.0.0.1:8013/player/?id=v061-demo-player)**.
This is the Starship emulator-backed preview. Production remains v0.6.1.

1. Review the natural-height player card and record/list hierarchy on desktop.
2. Check mobile identity, Copy and the collapsed Profile details. At 320px,
   Copy becomes an icon button so the public name keeps useful space.
3. Change pages, open a Replay, then use Return or browser Back. The logical
   page, scroll and originating Replay control should return together.
4. Expand About these records and try Copy. If clipboard access is unavailable,
   the selected URL becomes visible for manual copying.

The demo contains 18 selected games over three logical pages. Its root and four
physical page documents were read and hash-checked without writes. Their data
predates this UI pass; the record is a public snapshot, not verified statistics.

## Captures

These are newly rendered application captures, not the design prototype.
The first four use the preserved manual demo. The final two use independent
synthetic fixtures in the isolated acceptance emulator.

| Capture | Coverage |
|---|---|
| [Desktop](desktop.png) | 1440×1000, shared statistics and eight-row list |
| [Portrait](portrait.png) | 390×844, compact identity and collapsed details |
| [Narrow](narrow.png) | 320×900, icon Copy and reflow |
| [Enlarged text](enlarged.png) | 640×900 at 200% root text |
| [Changed catalog](public-catalog-error.png) | Retained public identity, full root reload and page retry |
| [Empty and clipboard fallback](public-empty-copy-fallback.png) | Statistics off, zero games, expanded/selectable canonical URL |

All six captures were inspected. Additional statistics-off and unavailable
states were exercised in the browser journeys. Screenshots are viewport
captures; content below the fold remains scrollable.

## Verification and limits

- Web typecheck, **503 units / 50 files**, and production/manual builds passed.
- **93** ordinary browser checks passed across Chromium, Firefox and WebKit.
- **18 distinct cloud cases** passed: C10/C11/C15/C16/C17/C18 in each engine.
  This count combines successful cases and focused correction reruns; early
  runs contained failures. The JSON maps each accepted case to its raw log.
- The final manual build passed **15** extra three-engine layout/return checks.
  Desktop return restored 372px scroll in Chromium/WebKit and 379px in Firefox,
  with trigger focus and no page errors. Public-origin replay headers also fit
  390px and 320px. Functional cloud checks cover native Back and removed entries.
- Public visitor, Unicode and adverse-state axe checks passed. This is bounded
  automated evidence, not a WCAG certification or human accessibility review.
- All **145** files in the final manual build matched served bytes. Carbon's
  forwarded index and entry hashes matched Starship; Auth and the demo public
  document responded. This proves HTTP reachability, not human browser/device
  acceptance.

Raw logs, builds and the capture helper remain ignored under
`gomoku-web/test-results/v062-implementation/`. Standard browser checks used
an owned preview on 8014, now stopped. Every cloud harness used
`GOMOKU_CLOUD_PORT_OFFSET=1000`, its own preview/emulators and one test worker.
The existing manual ports, services, data and Carbon tunnel were preserved;
unrelated port 8001 was not used. No user-browser tabs were opened.

The prior manual build is backed up at
`test-results/v062-implementation/backup/dist-cloud-03765d3/` relative to
`gomoku-web`. Preview updates copy assets before replacing HTML and retain older
hashed chunks for open tabs. Recovery can restore that build with the same
ordering; emulator data does not need to change.

Human visual approval, physical phone/touch and human accessibility acceptance
remain open, alongside the earlier real-account sharing handoff. Owner/help,
static examples, documentation consolidation and release media remain separate
0.6.2 work. Commits are local; no push, tag or production deployment occurred.
