# Dependency Maintenance Notes

Owner: manual maintenance passes. Recheck these notes when updating Firebase
or preparing a release. Last reviewed: 2026-10-27 for the `0.6.3` readiness review.

## Overrides

- `@firebase/firestore` resolves `@grpc/grpc-js` to `^1.14.6` instead of its
  upstream `~1.9.0` range. This keeps the existing 1.x API while including the
  fixes for [GHSA-m9gg-hp2v-232j](https://github.com/advisories/GHSA-m9gg-hp2v-232j)
  and [GHSA-f596-whhp-79r4](https://github.com/advisories/GHSA-f596-whhp-79r4).
  Remove the override when Firestore's own range resolves a patched version.
- The existing `uuid` override keeps the Firebase CLI's transitive `gaxios`
  dependency on the 14.x line. Recheck it when updating the CLI.

The web build uses native top-level await with its existing `esnext` target
and ES workers. The redundant `vite-plugin-top-level-await` dependency and its
plugin-specific override have been removed.

## Firebase CLI Audit Limits

The retained exception in the `firebase-tools` 15.34.0 development tree is the
OpenTelemetry issue below. The 0.6.3 readiness audit adds other actionable dev
findings in its later section. These packages are outside the production tree.

| Chain | Finding | Follow-up |
|---|---|---|
| `firebase-tools` → `@google-cloud/pubsub` 5.3.1 → `@opentelemetry/core` 1.30.1 | [GHSA-8988-4f7v-96qf](https://github.com/advisories/GHSA-8988-4f7v-96qf): unbounded baggage allocation. The fix starts at 2.8.0, outside Pub/Sub's 1.x range. | Prefer a Firebase CLI update that adopts patched Pub/Sub; review a major migration separately if needed. |

The October 23 full npm audit reports three moderate affected package entries
from this issue, including parent packages. It no longer reports the `braces`
finding listed in the 0.5.5 audit; dependency resolutions did not change during
0.6.1 packaging. The production npm audit
is a separate required gate. Local rules checks use the Firestore emulator;
passing them does not establish acceptance of other Firebase CLI features.

## Deferred Major Updates

Firebase 13, rules-unit-testing 6, Vitest 5, jsdom 30, testing-library/jest-dom 7,
and TypeScript 7 remain separate migration work. The cleanup pass updates within
the existing dependency ranges and validates the resulting product.

## 0.5.5 Release Audit

Fresh `npm audit --omit=dev --json` and `cargo audit --json` checks passed on
October 16 with zero production npm vulnerabilities, zero Rust vulnerabilities,
and no Rust informational warnings. Cargo used advisory database revision
`f28da8c5d386e507fac79153655184056a20c054` (1,329 advisories). The full npm audit
reported five development-only affected entries from the two Firebase CLI
issues recorded at that checkpoint. The release version bump changes no
dependency ranges or resolved package versions.

## 0.6.1 Release Audit

Fresh `npm audit --omit=dev --json` and `cargo audit --json` checks passed on
October 23 with zero production npm vulnerabilities, zero Rust vulnerabilities,
and no Rust informational warnings. Cargo used advisory database revision
`84d943a6deeb8cc6c6f632531aaf4f1c62275239` (1,351 advisories). The full npm audit
reports only the three moderate development-only entries described above.
The 0.6.1 version bump changes no dependency ranges or resolved versions.
Raw reports are retained under ignored
`gomoku-bot-lab/outputs/release-prep/0.6.1/`.

## 0.6.3 Readiness Audit — Follow-up Required

At `98d60f5`, fresh production npm and Rust audits pass with zero findings or
Rust warnings. The full npm audit reports five development-only affected package
entries (one high, four moderate), adding the following to the previously
recorded OpenTelemetry/Firebase CLI exception:

| Locked dependency | Finding and compatible patched line |
|---|---|
| `body-parser` 1.20.8 and nested 2.3.0 | [Compressed-body DoS](https://github.com/advisories/GHSA-27r3-jxmh-p828), high; patched in 1.20.9 and 2.4.0. |
| `fast-uri` 3.1.8 | [Scheme decoding](https://github.com/advisories/GHSA-w74x-v6f9-j3xg) and related URI-normalization advisories; the affected 3.x line is patched in 3.1.9. |

Both packages are development dependencies; the dependency lock is unchanged
from 0.6.2. Apply a narrow compatible resolution refresh and validate the affected
tooling before closing this follow-up. Do not use the audit's suggested major
Firebase CLI downgrade to dispose of the retained OpenTelemetry issue.
No dependencies were installed or updated in this review. The
[release-readiness review](v0_6_3_release_review.md) owns the findings and raw
evidence pointers.
