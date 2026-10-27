# 0.6.3 release-review fixes

Authorized with the Luna goal loop on October 27, 2026, from clean local
`9619214`. This checkpoint resolves RR1/RR2 from the
[release-wide review](v0_6_3_release_review.md) and its small RR3 CI follow-up.
The primary owns behavior/design decisions, integration and acceptance; Luna Max
owns the bounded implementation. Commit locally in cohesive stages. No push,
version bump, tag, release, production change or real-account write.

## Contract

- **RR1:** Disable profile danger actions while Auth is loading, preserving the
  existing signed-in profile-loading guard. Resolved signed-out/unconfigured
  guest reset stays available. Preserve UID-change confirmation clearing and
  operation/account ownership. C14 must explicitly await restored sign-in after
  both Profile navigations before starting its signed-in reset/delete sequence.
  Cover loading-to-resolved behavior and retained identity safeguards without
  arbitrary delays or retries that hide failures.
- **RR2:** Refresh only affected `body-parser` and `fast-uri` resolutions and
  their necessary compatible transitive changes. Preserve dependency ranges,
  versions of unrelated tools, generated Wasm, production schemas and release
  version. Do not apply a broad upgrade or audit-force downgrade. The known
  OpenTelemetry/Firebase CLI exception remains distinct and explicit.
- **RR3:** Add one package command for all existing Node capture/export/audit
  helper tests and invoke it in CI. Preserve the existing focused capture
  command and the rest of the workflow. This is a small coverage addition.

Historical data/media, static developer/private boundaries, account-change
safeguards, public sharing/reset semantics and existing manual services remain
intact. Primary review must inspect actual lock changes and failure paths.

## Exit gate

Run focused component/tool checks, type checks and all unit tests. Confirm the
installed graph matches the narrow lock refresh, production audit is clean and
the full audit retains only the known exception. Validate rules and build using
owned outputs/ports. Require C14 five-per-engine repetitions, the complete
three-engine ordinary/cloud lanes and five-per-engine C4/C6/C7 race coverage.
Check dev-site/media delivery and applicable standalone-export compatibility.
Reuse unchanged native/capture evidence only after verifying its source identity.
Keep failure artifacts separate from passing reruns; resolve new integration
findings before closing the goal. Update the review/maintenance records and
source-qualified evidence. Stop only owned test services and leave a clean local
worktree. Release metadata, pushed exact-source CI and human/device acceptance
remain separate gates.

## Completed checkpoint

Completed locally on October 27 at implementation source `e34129f`. The closing
documentation commit changes none of the tested implementation inputs. The
[validation receipt](v0_6_3_release_fixes/validation.json) records source hashes,
commands, fresh/reused evidence and installation qualifications.

| Finding | Result |
|---|---|
| RR1 | Auth loading now disables profile danger actions. Two added component scenarios verify loading-to-guest/account transitions, correct confirmation scope, no premature mutation and retained UID-change invalidation. C14 explicitly waits for restored sign-in at both entry points. Five repetitions per engine pass 15/15; the complete cloud lane passes 60/60. |
| RR2 | Exactly three lock entries change: body-parser 1.20.9 and nested 2.4.0, plus fast-uri 3.1.9. Root ranges, release version, unrelated resolutions and generated Wasm remain unchanged. A fresh locked install, installed-graph validation and audits pass with only the documented OpenTelemetry/Firebase CLI exception. |
| RR3 | `npm run test:tools` covers all four maintained Node test files and is invoked by CI. Primary review caught the output guard's dependency on the host's RTK wrapper; Luna changed that subprocess to native Git. All 44 tests pass in Ubuntu/Node 24 without RTK installed. |

Luna implemented the application/test, dependency and CI slices and the bounded
portability correction. The primary settled the behavior, reviewed the actual
diffs and failure paths, installed the lock, ran integration/final validation,
and accepted the result. Local commits are `da96c2a` (readiness), `3562e6c`
(dependencies) and `e34129f` (CI/native Git). No validation failure required a
later implementation change; the old review failures remain preserved.

## Final validation

- App/scripts/tools type checks and **558 unit tests** pass. The two new
  component scenarios join the existing Profile/reset/account suite.
- **44 Node tool checks** pass in the Ubuntu container's Node 24.14.1 with Git
  available and RTK absent. The CI command covers every current `.test.mjs` file.
- The production-configured build and all publishers pass from clean source,
  using the checked-in Pages workflow's public Firebase settings and base `/`.
  All **585 served files / 22,833,908 bytes** match SHA-256.
- The full ordinary three-engine lane passes **297 tests**, with six intentional
  non-Chromium WebGL-colour skips. Forced Canvas remains checked in all engines.
- Isolated demo-only rules tests pass **49/49**. The full cloud lane passes
  **60/60**, C14 repetitions **15/15**, and repeated C4/C6/C7 races **45/45**.
  Each invocation uses distinct output and emulator port ranges.
- Production npm and Rust audits report zero findings; Rust has no advisory
  warnings. The full npm audit retains only three moderate package entries from
  the documented OpenTelemetry/Firebase CLI chain. It reports no high findings.
- Both actual export CLIs create new outputs and reject repeated targets without
  changing existing receipts. The fresh standalone export passes region names,
  native/reload/Back and 320px/200%-text checks in all three engines.
- All **488 inventoried dev-site payload files** are byte-identical to the earlier
  fully inspected portable preview; only the export source/date receipt differs.
  Reuse of its 34-route-per-engine, history-return, UI-audit and Snap-delivery
  evidence is explicitly qualified. Historical/media/recorded Lab inputs and
  native code are unchanged. The earlier 348 native tests remain retained.

Installation ran on host Node 26.12.1/npm 12.2.1. The log retains superstatic's
engine-range warning and npm's default blocked-install-script notices; no
script-approval policy was changed. The installed graph, production build and
the exercised CLI paths pass. Browser/tool checks use Ubuntu/Node 24.14.1.
This does not replace the normal fresh Node-24/Wasm build on exact-source hosted CI.

## Handoff

All RR1–RR3 follow-ups are resolved within this local checkpoint. Existing
preview 46862 and shared report host 8787 remain running; owned previews
47439/47440, demo emulators and headless containers are stopped. No user tabs
or worktrees were opened/removed. Raw evidence stays under ignored
`gomoku-web/test-results/v063-release-fixes-20261027/`.

The candidate is ready for release preparation. Package metadata still says
0.6.2; the 0.6.3 version/changelog/status updates, authorized push, exact-source
CI, tag/publication and deployed-artifact verification remain separate work.
No push, version bump, tag, release, production change or real-account write
occurred. Inherited human/device/account acceptance and inconclusive contrast
checks retain their recorded limits.
