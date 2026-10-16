# Dependency Maintenance Notes

Owner: manual maintenance passes. Recheck these notes when updating Firebase
or preparing a release. Last reviewed: 2026-10-16 for `0.5.5` release preparation.

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

The current `firebase-tools` 15.34.0 development dependency still carries two
upstream issues. These packages are outside the production dependency tree.

| Chain | Finding | Follow-up |
|---|---|---|
| `firebase-tools` → `chokidar` 3.6.0 → `braces` 3.0.3 | [GHSA-vfj7-8cjw-p6xm](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm): deeply nested patterns can exhaust the stack. No patched `braces` release is published. | Update when an upstream fix or compatible CLI dependency change is available. |
| `firebase-tools` → `@google-cloud/pubsub` 5.3.1 → `@opentelemetry/core` 1.30.1 | [GHSA-8988-4f7v-96qf](https://github.com/advisories/GHSA-8988-4f7v-96qf): unbounded baggage allocation. The fix starts at 2.8.0, outside Pub/Sub's 1.x range. | Prefer a Firebase CLI update that adopts patched Pub/Sub; review a major migration separately if needed. |

The full npm audit reports five affected package entries from these two issues
(three high, two moderate), including parent packages. The production npm audit
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
still reports the same five development-only affected entries from the two
Firebase CLI issues above. The release version bump changes no dependency ranges
or resolved package versions.
