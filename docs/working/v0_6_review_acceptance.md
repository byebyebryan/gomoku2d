# v0.6 unattended review and acceptance

Owner: Bryan. Started October 22, 2026. Status: active.

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

- R0: baseline verified; coverage review in progress.
- R1–R5: pending.

## Manual handoff target

The final script will cover one coherent demo: finish/open a game, share a chosen
moment, inspect it in another browser, save it, continue as practice, enable a
public name, publish, browse as a guest, unpublish and disable. Keep the manual
questions focused on comprehension and feel. Separately identify real Google
sign-in/consent, an actual phone/touch session, human accessibility and public-host
acceptance. Release/deployment preparation is a later checkpoint.
