# 0.6.3 design-review backfill

Owner: Bryan. Started October 26, 2026, from `cee9075`.
Bryan authorized the Luna goal loop after reviewing the latest-first history
and the proposed backfill. The primary owns case selection, copywriting, design,
integration and acceptance. Luna handles factual investigation, bounded media
preparation, non-editorial implementation and independent verification.

Status: local implementation and technical validation complete. The
[shared preview](http://starship:8787/reports/a2698af3696448d16662bc0ec896d809da9f4177fc547b7080a11ef23e75f603/)
is verified from Snap. Bryan's review of the new cases remains open; no push,
release tag or production deployment is part of this checkpoint.

## Outcome

Present actual design work as useful review cases. The unit is a coherent
design problem or focused review, rather than a release or an individual patch.
Keep the current-screen reference and complete UI history easy to reach.

The bounded roster is:

| Case | Source period | Work |
| --- | --- | --- |
| Board-first shell and pixel identity | 0.2.1–0.2.2 | Backfill original critique and the shell/visual decisions. |
| Portrait layouts and touch placement | 0.2.3–0.2.4 | Backfill mobile controls and the following density/polish pass. |
| Readable replay analysis | 0.4.6–0.4.8 | Backfill timeline, status and restrained annotation decisions. |
| Lab, Visuals and supporting pages | 0.5.1–0.5.2 | Backfill the move from separate report pages into the product shell. |
| Share/Publish replay sidebar | 0.6.1 | Add the missing review of the full-sidebar sharing/publishing interaction. |
| Public-player page | 0.6.2 | Expand the existing case with recorded alternatives and their selection. |

These periods identify the design work. Case version ranges also include an
earlier comparison baseline where needed.

Retain the existing action-icon and mobile-record cases, all case IDs/URLs,
current references, endpoint pilot, immutable media and latest-first history.
No multiplayer/profile features, report regeneration or new release belongs here.

## Presentation and evidence

The primary will use a short question and outcome, a small sequence of design
steps with their relevant captures, and concise findings/follow-up. Metadata and
full source identities remain secondary. The design-review index leads with
these cases, latest source period first; screen references and version history
remain sibling destinations. Existing viewer inspection/deep-link controls stay
available, and legacy case layout remains compatible.

Original critique/plan text supplies rationale. Qualified retrospective media
can illustrate an older implementation, with that origin labelled per capture.
Frozen design studies remain mockups, even when newly rendered. A new review's
assembly date is not an original review date. Missing original alternatives,
device conditions or human acceptance stay explicit; screenshots alone do not
establish a historical decision.

Prefer existing qualified images. Audit original Git-only image references
before recovering exact blobs. Render frozen prototype options only when needed
to show an actual recorded alternative. Do not alter/recolour historical PNGs,
replace selected media, use real accounts or start cloud services.

## Exit gate

- Each selected case has source-backed rationale and qualified media, or a
  clearly recorded limitation resolved by the primary.
- Copy and layout are primary-authored and fact-checked against original evidence.
- Runtime/publication validation covers the new structured records and their
  comparison references; media bytes/dimensions remain checked.
- Appropriate type, unit, build, responsive/browser, navigation/history,
  accessibility, export and static-boundary checks pass; findings are fixed.
- Commit locally in cohesive stages. No push, tag, production deployment or
  real-device acceptance claim.
- Deliver an immutable portable preview through the ordinary codex-plus report
  host, checking its actual prefix and Snap delivery when available. Preserve
  unrelated services/worktrees/tabs and clean only owned temporary resources.

## Execution

- Primary confirmed a clean starting worktree and inspected the current record,
  viewer, publisher, documentation and existing browser coverage.
- The audit binds 39 source entries to exact document blobs, with 34 text
  excerpts and five JSON observations. Primary review corrected a conflation of
  release IDs and later document revisions. A later warning that the retained
  changelogs omitted older entries was rejected after checking the complete raw
  Git blobs; the initial review output was incomplete.
- Four original active Replay images were recovered, and five frozen profile
  mockups rendered. The primary inspected all nine images and independently
  checked their hashes/dimensions and the audit's exact excerpts/pointers.
- Five new cases and the expanded public-player case are primary-authored.
  Existing action-icon/mobile cases, reference/pilot URLs and capture pixels
  remain available. Structured cases show figures before the longer step notes.
- The portable export reuses the actual developer components, validates selected
  publishers and records source/tool/file identity. Its preliminary nested-prefix
  check passed all 16 history routes plus retained developer destinations in
  Chromium, Firefox and WebKit.

## Validation and delivery closeout

The [validation receipt](v0_6_3_design_reviews/validation.json) records source
`56d041b`, tools, input hashes, counts, preservation and delivery. The full
source/media audit and capture manifest live in the
[evidence directory](v0_6_3_design_reviews/README.md).

- Type checks, the production build/publishers and 556 unit tests passed.
- The main developer/history/design suites passed 57 browser tests across
  Chromium, Firefox and WebKit. The enlarged-text/mobile audit checked 21 views,
  with zero automated accessibility violations or incomplete checks, no overflow,
  matched image scale and no source upsampling.
- The published export passed 34 views per engine, including all ten case
  records, every screen-history entry, assets and Lab. Native inspection,
  reload/Back and 200% text passed; hash-router history also restored the
  originating milestone's focus, scroll and disclosures after cross-screen and
  Native-pixel navigation. No unexpected requests or private storage appeared.
- The single-record public-player export passed its three-engine check too.
- All five original record IDs, 22 comparison IDs and their capture bytes/identity
  remain intact. History data and both raw Lab reports are unchanged.
- Snap fetched all 489 published files: 19,621,660 bytes, with complete SHA-256
  parity against the prepared export. The source was clean at export time.
- Both owned temporary check servers were stopped. The normal preview and
  shared report host remain available. No user browser tabs, worktrees, real
  accounts or cloud services were opened/changed by this loop.

The closeout documentation follows the tested source commit; its recorded
application, export, data and test inputs remain unchanged. Original image
conditions and reconstructed conversation dates keep their stated limits.

## Reconstructed Share review context

The Share/Publish case draws its iteration sequence from the development
conversation supplied by Bryan for this task. This is a transcription made
October 26, 2026; the original turns' dates and exact preview revisions are not
available here. It is separate from the contemporaneous implementation contract.

The supplied conversation records this sequence:

1. Share replay worked functionally, but Bryan described the section as detached
   from the rest of the UI and resembling a generic web control.
2. He found the expanded section crowded at the bottom and requested a button
   that would slide/swap in the whole sidebar.
3. After icon, text and presentation refinements, he said Share replay looked
   good. This approves that reviewed presentation; it does not identify a
   capture/build or establish acceptance of a later reconstruction.
4. When Publish replay appeared, he requested the same whole-sidebar behaviour
   and a similar pass over its text-heavy presentation.
5. He requested a distinct Publish icon because reusing Profile was confusing.

The 0.6.1 UI contract and implementation evidence corroborate the final shared
interaction. The selected retrospective images illustrate the shipped panes;
the initial expansion was not recovered as an image. The public case must keep
this conversation-derived attribution and its unknown dates explicit.

## Naming refinement

After a positive visual pass over the backfill, Bryan requested clearer section
names: **Visual design**, **Bot lab** and **UI design**. Navigation, overview
cards, related links and browser titles now use those names. UI design retains
Current screens, UI history and Design reviews. Route/query identities and
historical capture labels remain intact.

The production build, 56 targeted unit tests and 18 browser checks across
Chromium, Firefox and WebKit passed. Desktop and portrait inspection confirmed
that the new labels fit; the existing reflow checks include 320px and 200% text.
This is a local naming follow-up, with production publication still separate.

Bryan then selected the order **Visual design → UI design → Bot lab** and
requested reconciliation beyond the developer frame. A shared section definition
now supplies the app's navigation, overview cards, developer footer and Home
links. Home, the root/web READMEs and the docs index include UI design alongside
the other two sections; the current asset docs also drop the retired Process
entry. Historical plans, captures and quoted past names keep their original
identity.

The reconciliation passed the production build, 33 App unit tests and 12 focused
browser tests across Chromium, Firefox and WebKit. Direct inspection checked
section/card/footer order and Home links in all three engines, including 320px
and 200% text; primary review inspected desktop, portrait and both footers.
The three Markdown link lists match names, destinations and order, and their
local documentation links resolve. UI history, review records and Lab data are
unchanged by this follow-up.

## Section-navigation scroll fix

Bryan reported that entering UI design jumped past the site header. The previous
shared preview reproduced a 140px scroll at 412px in Chromium and WebKit; Firefox
and the inspected desktop size remained at the top. Section navigation reset
the frame, then its unqualified content-focus call could scroll it again.
Focus now uses `preventScroll`, keeping keyboard focus on main content while
retaining the header. Explicit skip/comparison/history inspection still scrolls
to its requested target.

The production build and 12 focused browser checks passed across all three
engines. Navigation checks now cover desktop and portrait, asserting top scroll
and a visible header; the suite also retains case inspection/reload/Back and
history focus/scroll/disclosure restoration. The starting subview is independent
of this scroll correction.

Bryan subsequently selected **Current screens** as the UI design starting view.
The shared UI entry, Home/cards/footer, Markdown links and portable preview now
open the existing core-screen reference. Section matching is independent of the
entry destination, so UI design remains active on history and all review pages.
The Design reviews index and existing record URLs remain addressable. The build,
33 App unit tests and 12 three-engine browser checks passed, including the
starting subtab, top scroll, Home link and active state across all three views.

Bryan accepted **Dev site** as the public umbrella name. Header branding is now
GOMOKU2D / DEV SITE; Home, root/web README and docs-index overview links say
Dev site. Browser title, accessible navigation and recovery links use the same
shared label. The internal first tab remains Overview, followed by Visual design,
UI design and Bot lab. Implementation identifiers and route identities stay
stable. The build, 33 App unit tests and 12 three-engine browser checks passed;
the three Markdown link groups match the accepted labels, order and destinations.

Bryan then accepted compact external navigation: Home now lists Rules, Guide,
Examples and Dev site, followed by Source, Privacy and Terms. Root/web README
top navigation also uses only Dev site for the developer area; direct tool links
remain in their explanatory sections. The dev site's own three-section
navigation/cards/footer and the docs index's full surface catalog remain useful
inside their respective contexts. The build, 33 App unit tests and nine browser
checks passed, including Home → Dev site and access to all three overview cards.
