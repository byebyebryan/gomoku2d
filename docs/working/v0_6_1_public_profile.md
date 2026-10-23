# v0.6.1 public profile: functional pass

Owner: Bryan. October 22, 2026. Baseline `d575d40` on local `main`.
This is a local implementation and review checkpoint, with a Starship-hosted
preview accessible from Carbon. Version publication, real OAuth, deployment,
physical-device acceptance are separate gates. The existing Canvas stone-colour
finding was subsequently resolved in the
[remediation checkpoint](v0_6_1_evidence/README.md#remediation-completed--october-23-2026).

## Accepted scope

- Add Public profile as the third right-side Profile tab, beside My games and
  Saved replays. Remove the cramped sidebar manager and its nested accordion.
  The sidebar retains private identity/account/reset controls.
- Public content: chosen public name, optional player statistics, and a compact
  paged list of games explicitly published by the owner. Defer avatars, bios,
  featured games, board previews and richer statistical breakdowns.
- Statistics match My games: completed games, including continuations, count;
  saved study copies do not. Include matches, wins, losses and draws; derive
  win rate as wins / matches, showing an em dash for an empty record.
- Public statistics are off by default and independently controlled from public
  profile visibility. Enabling publishes the currently displayed complete
  summary. Refresh is explicit through Update public stats, with the snapshot's
  update date visible. Turning sharing off removes the summary immediately.
- Publishing statistics must wait for the current signed-in account's private
  history to load. Zero games is a valid complete summary; loading/error states
  must not publish an accidental zero/partial record.
- Shared games use eight compact rows per logical page. Show title, opponent,
  result, rule, move count and a Replay action, with stacked rows on mobile.
  Use a neutral unavailable state for malformed or unattributed old entries.
  Do not invent played dates absent from the portable format.
- Pagination reflects actual publications, fills pages across storage holes,
  and hides navigation when no second page exists. The 32-game cap remains.

## Data and concurrency contract

Keep the existing public identity document and four bounded eight-entry page
documents. Add two optional fields to the schema-1 public root:

| Field | Contract |
|---|---|
| `shared_stats` | Absent/null means off. Otherwise exactly `matches`, `wins`, `losses`, `draws`, `updated_at`; counts are nonnegative bounded integers, matches equals the three outcomes' sum, and update time is a server timestamp. |
| `catalog_page_counts` | Four integer counts in storage-page order, each 0 through 8. Their sum determines logical page count. |

New roots include a null statistics snapshot and four zero counts. The current
reader accepts the exact legacy required fields plus these known optional
fields and rejects unknown fields. Previously released v0.5 clients do not use
the sharing families. Earlier unreleased v0.6 strict readers reject extended
roots safely and require a page refresh/update; preserving those binaries'
public-profile reads would require a separate document and is not selected.
Root profile schema 5, SavedMatchV2 schema 2 and portable replay v1 remain as-is.

Catalog mutations compare the expected root revision, page revision and
generation, then atomically write the page and its root count/revision while
preserving name and statistics. Return both updated snapshots to refresh owner
and publisher state. Rules validate count/page agreement, prevent manifest
removal after introduction and reject page-only writes that desynchronise it.
Statistics refresh/removal compares root revision and advances it without
changing catalog generation or games. Name edits preserve the stats snapshot
and its update date. Conflicts require reload/review before retry.

Legacy roots without counts stay readable. Visitor fallback reads their four
pages once, validates generations and caches a derived manifest without writing
a migration. A first catalog mutation may initialise counts from those pages.
Modern visitors read the root plus only the physical pages needed to fill the
selected logical page; already fetched pages are reused. Empty pages are not
presented as public content. Manifest/page mismatches produce an actionable
reload state, not fabricated counts or empty navigation.

## Privacy, lifecycle and cost

- Public routes request only public identity/catalog documents, never private
  history/library documents. A snapshot contains only the four aggregate counts
  and its update timestamp, with no private IDs, settings or match list.
- Snapshot publication is an explicit owner action. No per-move operations,
  automatic stats writes, polling, persistent listener or new service.
- Private Reset preserves public identity, shared statistics and publications.
  Public disable and Delete Cloud clear statistics with the catalog, preserving
  existing generation/revision barriers. Re-enable starts empty, stats off.
- Old profile-delete paths must not bypass remaining shared statistics. Stale
  refresh/publish operations and delayed callbacks cannot restore cleared data
  or update another account's visible state.
- Ordinary catalog updates use two document reads and two writes before
  retries/rules-dependent reads; legacy count initialisation reads at most five
  documents. Snapshot refresh reads and writes one root. Public root-plus-page
  reads remain bounded and use no history fan-out.

## UI contract

Use the existing pixel fonts, action buttons, field styles, result colours and
compact metadata. This pass establishes functionality and readable desktop /
mobile layouts, with further refinement along the 0.6 line.

The Public profile tab supports direct `/profile?tab=public` entry, normal
three-tab keyboard navigation and clear signed-out/loading/error states. It
contains public name/visibility, share-statistics controls and dated snapshot,
published-game management, and Copy link / View public profile actions.
Public manager entry points from replays should land on this tab. Opening the
tab loads owner data; public visits must not initialise private state.

The owner controls follow General Settings: grouped sections with explanatory
labels on the left and bounded fields/actions on the right, stacking at narrow
card widths. Publish replay uses the dedicated globe icon and replaces the
whole replay sidebar, matching Share replay's entry animation, paused playback,
Back/Escape dismissal and focus/scroll restoration. Its compact publication
preview shows the destination profile, title, opening move and both players.
Public-copy and unverified-result notices remain concise and visible.
Statistics sharing uses General Settings' Off/On segmented buttons. Selecting
the active state does no cloud work; refreshing remains a separate action.

## Review and execution

Luna Max reviewed the code before implementation. The review confirmed the
root's exact-key compatibility limit, full-root rewrite risks, sparse-page
handling and reset/delete semantics. The primary selected the additive root
contract above and retains integration and acceptance ownership.

1. Primary: extract the current played-history counter into one shared pure
   helper, preserving My games behavior and providing the publication input.
2. Luna: implement backend/parser/rules/lifecycle contracts and focused rule /
   operation tests; primary reviews data invariants and measured operations.
3. Luna: implement the public tab and visitor list/statistics against the
   reviewed API, updating component and cloud/browser scenarios.
4. Primary: integrate fixes, inspect desktop/mobile captures and adversarial
   states, run final gates, update Carbon preview and verify served bytes.

Each accepted cohesive slice is committed locally. Preserve concurrent work
and the managed preview/emulator/tunnel services; isolated test emulators must
not clear the manual-review database. Do not push, tag or publish production.

## Exit gate

- Full relevant web type checks, unit tests, Firestore rules/SDK tests and
  production build/direct-route checks pass.
- Three-engine browser coverage proves tab keyboard/deep-link behavior, compact
  layouts, optional stats, zero record, snapshot refresh/removal, eight-row
  logical paging including holes/last-item removal, legacy roots and replay entry.
- Emulated cloud scenarios prove owner/visitor privacy, exact operation shape,
  snapshot updates, stale races, account switch, private Reset preservation,
  disable/re-enable and Delete Cloud cleanup. Run the existing relevant full
  lane and required conflict/account/reset repetitions once final changes settle.
- Automated accessibility/reflow and inspected desktop/mobile captures support
  a manual review handoff, without claiming human/accessibility/device acceptance.
- Commit source, document actual validation and limitations, preserve demo data,
  and verify the final build and all three services from Carbon.

## Implementation checkpoint

The functional candidate is implemented in local commit `7e9f803`, with final
accessible metadata semantics in `bf7d963`. The primary
review corrected the logical-page offset helper, strengthened current-account
readiness, closed the stats-sharing kill-switch gap, and fixed internal
horizontal overflow with enlarged text. Luna delivered the backend and UI
slices plus focused regressions; the primary owns the integrated acceptance.

The [0.6.1 acceptance ledger](v0_6_1_evidence/README.md) records requirements,
actual commands/results, inspected captures, operation receipts and Carbon
served-byte parity. Its automated gate is complete; public profile and replay
functionality is ready for the next human pass. Further visual refinement
and richer profile content remain later 0.6 work. Versioning, push, release,
production rules/index deployment and real OAuth remain separate from this local
checkpoint. The subsequent [review remediation](v0_6_1_evidence/remediation.json)
closes the Canvas finding and parallel test-fixture collisions.
