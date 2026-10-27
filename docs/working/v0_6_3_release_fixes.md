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
