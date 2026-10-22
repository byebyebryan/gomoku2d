# v0.6 unattended review and acceptance

Owner: Bryan. Started October 22, 2026. Status: unattended review complete;
manual review in progress.

The [acceptance report and manual walkthrough](v0_6_review_evidence/README.md)
contain the unattended results, selected inspected screenshots and build identity
for application source `f1bc7ee`. The manual UI follow-up below is a later,
separately checked increment. The new commits remain local and unpushed.

## Objective and boundaries

Take the shared replay/public profile candidate at `0995af8` to a documented
ready-for-manual-review checkpoint. Review the implemented contracts, close
meaningful coverage gaps, fix concrete defects and preserve the existing UI,
gameplay, data and low-infrastructure architecture. Commit reviewed slices
locally as work progresses. New pushes, release/version changes, deployment,
production data writes and traffic/load experiments are outside this goal.

Completion requires all planned automated checks to pass, all findings to have
a recorded disposition, no known blocking defects, evidence identifying the
tested source/build, and a short manual acceptance script. Automated evidence
does not establish real OAuth, physical-device or human accessibility acceptance.

## Accepted baseline

- Clean `main` at `0995af835a5e81377f1270b19d0cc780dce44f36`, already pushed.
- [Exact-commit CI](https://github.com/byebyebryan/gomoku2d/actions/runs/39725010863)
  passed: 45 files / 457 web tests, 43 Firestore rules checks, 192 browser
  checks across three engines, 36 cloud journeys and 45 race repetitions.
  Types, production build, Rust formatting/lint/workspace tests also passed.
- The immediately preceding production npm audit reported zero vulnerabilities.
  Known Firebase CLI development-only advisories remain documented in
  [dependency maintenance](dependency_maintenance.md).
- Earlier candidate receipts remain historical evidence in
  [the implementation ledger](v0_6_evidence/README.md). They will not be silently
  relabeled as evidence for later source changes.

## Plan and acceptance matrix

| Stage | Work and gaps to resolve | Acceptance evidence |
|---|---|---|
| R0: Baseline and coverage | Identify existing contracts/tests and avoid duplicating the successful CI matrix | Source/CI identity, this plan, named gaps |
| R1: Source review | Trace portable input, private/public data separation, publication eligibility, account changes, async operations, library conflict/clear behavior, and legacy cleanup | File/contract review record; each concrete finding reproduced, fixed or explicitly dispositioned |
| R2: Adversarial inputs and persistence | Check canonical/oversized/mutated payloads, hostile labels, maximum supported records, cache failures and recovery where current coverage is insufficient | Bounded reproducible fixtures/corpus; no invalid import, private-data disclosure or unbounded work |
| R3: Exploratory browser/emulator | Exercise 320px reflow, enlarged text, long unbroken/Unicode names/titles, keyboard/focus, error/empty states and fresh signed-in public entry variants | Targeted production-build journeys across three engines; screenshots inspected by the primary; automated accessibility and overflow results |
| R4: Cost and compatibility | Trace explicit reads/writes, absence of per-move/view listeners, bounded document shapes, old-client coexistence, index exemptions and rollback ordering | Source/operation review, scoped receipts and any new emulator checks warranted by findings; no live-capacity or billing claims |
| R5: Final acceptance | Review fixes, run checks appropriate to changed contracts, inspect resulting UI and clean up owned processes/artifacts | Final source/build identities, pass/fail counts, findings/resolutions, clean worktree and manual script |

Execution is sequential under the primary. Review findings determine the exact
new tests. Existing suites remain acceptance evidence unless code changes or a
new concern warrants rerunning them. A broad final lane is required if shared
state, rules or lifecycle changes affect multiple journeys; otherwise use the
affected suites and explain the narrower scope. Do not invent tests that only
mirror implementation details or start another bot research/performance phase.

Use disposable local/demo-emulator identities and production builds. Browser
automation stays headless, preview auto-open stays disabled, and routine captures
remain in test output. Retain only selected reviewed evidence in the repository.
Use 320 CSS pixels and enlarged-text checks as bounded layout evidence; record
their actual mechanism rather than claiming physical-device or browser-zoom
acceptance. Preserve unrelated work and user browser tabs.

## Execution and findings

- R0: baseline verified; existing broad CI accepted. Named gaps are failed
  recovery after cleanup, real signed-in route variants/public entry, and
  maximum/hostile labels at narrow/enlarged-text layouts.
- R1: portable codec/adapters, library store/sync/actions and cloud data APIs
  traced. Public/private documents are separate; inputs are bounded before
  Wasm; portable exports allowlist metadata and retain untrusted provenance.
  Remaining UI/rules/cost review continues in R3/R4.
- F1 (P1, fixed and validated): a failed reload changed
  `status` from conflict to error while retaining an incoming cloud snapshot.
  Ordinary Sync checked only status and could restore retained copies after
  cloud clear without explicit merge. A late failed sync could similarly
  overwrite a newer cleanup state. Reproduced before the fix in three failing
  sync scenarios. Sync/import now require the pending-review flag to be clear,
  affected buttons remain disabled during review, and superseded failures cannot
  patch newer sync state. Five added sync/action/UI regressions pass; focused
  library lane passes 22 checks.
- F2 (P2, fixed): native `maxlength=80` counted UTF-16 units, truncating valid
  80-code-point public names/titles to 40 supplementary Unicode characters.
  Reproduced in Chromium using the actual form. Both public-name forms and the
  publication title now allow the UTF-16 envelope and enforce 80 code points.
  C15 verifies enabling, editing, publishing and guest rendering at the limit,
  plus rejection of an additional ASCII character.
- F3 (P2, fixed): screenshot review found 80-character player labels expanding
  the mobile replay cards to hundreds of pixels and displacing the board.
  Labels now display at most two lines while retaining complete accessible text
  and a title. Reviewed the corrected 320px capture; the board is visible below
  compact player cards. Three-engine checks protect the visual bound and text.
- R2: six noncanonical equivalent encodings now have explicit rejection cases
  (whitespace, duplicate keys, alternate numbers, escaped labels and field
  order). Existing legality, size, Unicode, cache and malformed-input coverage
  was reviewed; no further decoder defect found. The full web lane passes
  45 files / 468 tests.
- R3 targeted checks: 9/9 exploratory browser checks and 6/6 C10/C15 emulator
  checks pass across three engines. These include maximum 225-move draw,
  hostile labels/redaction, 320px reflow, 200% root-text scaling at 640px,
  maximum public fields and eight fresh signed-in shared/public route variants
  per engine with no private targets or history promotion. The primary inspected
  replay, owner, visitor and enlarged-text screenshots. Broad final lanes remain.
- R4: source review confirms explicit server reads/transactions, no sharing
  listeners/timers, separate owner-only library/private profile documents,
  known-UID public reads without listing, bounded payloads and retained markers.
  Index exemptions and rules-first rollout/compatible rollback contracts match
  the implemented families. Fresh emulator rules/SDK-operation lane passes
  43/43, including method counts, sizes and stale-write/deletion barriers.
  No rules/schema/backend implementation changes were required.
- R5: complete. Final types/build, 468 web tests, 43 rules/operation checks,
  201 browser checks and 39 cloud journeys pass. C13/C14 library deletion/reset
  checks pass 18/18 across three fresh runs in all three engines. The baseline's
  45 older C4/C6/C7 repetitions remain accepted for unchanged legacy sources;
  the new repetitions target the library behavior changed here. Production npm
  and Rust audits pass. No known blocking defects remain after this bounded
  review. Real OAuth/devices/human accessibility and deployment remain separate.

## Outcome

Three findings were resolved in scope: the P1 cleanup review-barrier bypass,
the P2 Unicode input mismatch and P2 replay-card expansion. No backend/schema,
Rust/bot, dependency, release-version or production changes were needed. Final
evidence distinguishes the successful pushed baseline CI from local final-source
acceptance. The primary inspected the selected runtime captures; source/bundle
hashes and unchanged compatibility inputs are recorded. Owned previews and
emulators are stopped at closeout. No user browser tabs were created by this goal.

## Manual handoff target

The final script will cover one coherent demo: finish/open a game, share a chosen
moment, inspect it in another browser, save it, continue as practice, enable a
public name, publish, browse as a guest, unpublish and disable. Keep the manual
questions focused on comprehension and feel. Separately identify real Google
sign-in/consent, an actual phone/touch session, human accessibility and public-host
acceptance. Release/deployment preparation is a later checkpoint.

## Manual review follow-up — October 22

Bryan confirmed share-replay functionality and accepted the refined presentation:
sharing replaces the replay sidebar, with segmented opening-position/name
choices, an explicit recipient preview and a prominent copy action. Back to
replay and Escape restore the controls and selected move; opening sharing pauses
autoplay, preserves the board and retains the sharing choices. The approved
icons are the centred share arrow with its bottom tip row trimmed, overlapping
copy pages and a floppy disk on Save replay. The additions use the original
24-pixel grid and are identified as standalone additions in the visual guide.

Scoped validation passed: types, production-style demo build, 18 replay/library
unit tests and 36 browser checks across Chromium, Firefox and WebKit, including
narrow/enlarged-text layouts, automated accessibility, keyboard return/focus,
clipboard fallback and practice/library behavior. Icon inspection also checked
integer coordinates, symmetry and crisp native/2× rendering. The final icon-only
trim was rebuilt, checked for its exact four-pixel removal and symmetry, then
visually rechecked in the served preview. Earlier broad acceptance counts above
remain evidence for their original source, rather than this UI increment.

Open finding from the manual demo: the shared board renderer uses a white sprite
and tint for Black, but Phaser's Canvas renderer ignores that tint. The demo
Chrome session selected Canvas and showed both sides white despite correct game
data. The same tint-only implementation exists in v0.5.5. A Canvas fallback fix
and appropriate rendering coverage remain required before release acceptance.

The saved-library, practice and public-profile manual journeys remain open.
The demo preview/emulators are running for that review; production publication
and real OAuth/device acceptance remain separate.

Bryan also accepted the supporting-control refinement: the guide uses a neutral
help button, play-from-here setup uses compact side/bot fields, private-save
scope is a labelled row, and link visibility sits inside the recipient preview.
Types, 23 unit tests, 60 browser scenarios across three engines and a narrow /
enlarged-text accessibility capture passed for that increment.
