# Testing Guidelines

## Command Matrix

Use the smallest command that protects the contract you changed. Before a broad
release or after core/rules/search work, run the full relevant lane.

| Area | Command |
|---|---|
| Rust formatting | `(cd gomoku-bot-lab && cargo fmt --all --check)` |
| Rust lint | `(cd gomoku-bot-lab && cargo clippy --workspace --all-targets -- -D warnings)` |
| Rust workspace tests | `(cd gomoku-bot-lab && cargo test --workspace)` |
| Rust dependency audit | `(cd gomoku-bot-lab && cargo audit)` |
| Tactical scenarios | `(cd gomoku-bot-lab && cargo run --release -p gomoku-eval -- tactical-scenarios)` |
| Lethal scenarios | `(cd gomoku-bot-lab && cargo run --release -p gomoku-eval -- lethal-scenarios)` |
| Web typecheck | `(cd gomoku-web && npm run typecheck)` |
| Web unit tests | `(cd gomoku-web && npm test)` |
| Web production dependency audit | `(cd gomoku-web && npm audit --omit=dev)` |
| Firestore rules | `(cd gomoku-web && npm run test:rules)` |
| Production build/direct routes | `(cd gomoku-web && GOMOKU_BASE_PATH=/ npm run build)` |
| Local browser smoke | `(cd gomoku-web && npm run playtest:smoke)` |
| Full local browser flows | `(cd gomoku-web && npm run playtest)` |
| Chromium / Firefox / WebKit flows | `(cd gomoku-web && npm run playtest:engines)` |
| Emulated cloud application continuity | `(cd gomoku-web && npm run test:cloud)` |
| Cloud conflict/account/reset repetition | `(cd gomoku-web && npm run test:cloud -- --grep 'C[467]' --repeat-each=5)` |
| Opt-in browser profiling | `(cd gomoku-web && npm run profile:browser)` |

Install the Rust audit command once with `cargo install cargo-audit --locked` if
it is not already available.

With `PLAYWRIGHT_BASE_URL` unset, ordinary browser commands use port 8001.
Locally, Playwright builds and starts a preview unless it finds an existing
server to reuse; verify that an existing listener serves the intended build.
Set `PLAYWRIGHT_BASE_URL` only when managing the target preview yourself. CI
starts its own preview of the preceding production build and refuses reuse.
The full browser lane includes keyboard completion/replay, denied
quota/corrupt/newer storage, and repeated bot-worker failure/recovery on desktop/mobile.
CI runs the full journeys across Chromium, Firefox, and WebKit against its
preceding production build and retains failure screenshots/traces for seven days.
Install those engines with `npx playwright install --with-deps chromium firefox webkit`.
Engine automation is distinct from physical mobile-browser acceptance. Real
OAuth, physical-device continuity, and a real screen reader still require their
own acceptance evidence.

The cloud command owns a demo-only Auth/Firestore emulator pair, disposable
accounts, the actual repository rules, and a separate `dist-cloud` preview on
port 8013. It refuses occupied ports, stops its own process groups, and blocks
non-loopback identity/database requests. The SDK still loads Google's static
GAPI library; cosmetic widget assets and generic telemetry/connectivity probes
are fulfilled locally. There are no production account/data writes. The lane runs all
C1-C14 journeys and supported embedded redirect in three engines, plus five
fresh-state repetitions of conflict, account-switch and reset races. The
[unattended evidence ledger](../../archive/reconciliation_2026_10/v0_5_unattended_evidence.md) records
scenarios, fixture-only timestamp eligibility, and the difference from real OAuth.

On Linux hosts missing WebKit libraries, `npm run test:cloud -- --container`
keeps Java/emulators on the host and runs Playwright in the qualified Ubuntu
container. Docker and the host's matching installed browser cache are required;
`PLAYWRIGHT_BROWSERS_PATH` overrides its default `~/.cache/ms-playwright` location.
The pinned Ubuntu image supplies libraries; mounted browsers match the lockfile.
Ordinary browser output uses `test-results/browser`; cloud output uses
`test-results/cloud`, allowing independent lanes without clearing each other’s
artifacts. See [v0.6 evidence](../../working/v0_6_evidence/README.md) for exact
candidate commands, logical storage operations and scope limits. Routine browser
captures stay in test output; set `GOMOKU_WRITE_EVIDENCE=1` only when intentionally
refreshing the checked-in candidate screenshots.

The profiling lane builds its own production preview on port 8012 and writes
JSON under ignored `gomoku-bot-lab/outputs/browser-profile/`. Set
`GOMOKU_PROFILE_OUTPUT` to retain it elsewhere. It records build/host identity,
five fresh/warm startup pairs, 108 fresh-worker bot requests across 12 validated
positions, three reused-worker positions, 18 uncached analyses across six
replays, cached revisits, active cancellation/replacement, and 30 navigation
cycles. Its frozen manifest is checked with the current Wasm core and the served
binary before timing. Diagnostic trigger breaches add five independent trials;
watchdogs and incomplete counts stay visible. Latency triggers are diagnostic
and profiling remains opt-in. It uses Chromium CDP for page CPU throttling and
main-heap metrics;
worker/Wasm/GPU heap and physical-device performance require separate evidence.
The [October review measurements](../../archive/reconciliation_2026_10/v0_5_review_evidence.md#browser-cost-and-worker-ownership)
document the samples and their provisional diagnostic expectations.

## Dependency Maintenance

Cargo, npm, and GitHub Actions updates are reviewed and applied manually during
maintenance passes and release preparation. Routine changes go directly to
`main` after local review and validation. Scheduled Dependabot version PRs are
disabled; GitHub vulnerability alerts remain enabled, and automatic security
update PRs remain disabled.

1. Review upstream release notes and update dependencies in small, scoped
   batches. Check the current Rust stable toolchain for new lint failures too.
2. Preserve the generated local `gomoku-wasm` dependency when updating npm.
   Rebuild Wasm when Rust dependencies change and refresh the web dependency.
   npm updates remain manual because Dependabot cannot resolve that generated
   local package.
3. Run the relevant checks above, including Rust and npm production dependency
   audits. Use browser smoke when an update can affect player-facing behavior.
4. Review manifest, lockfile, and workflow diffs before committing, then verify
   CI on the pushed `main` commit. Refresh curated reports only when their
   behavior or source data changes.

Current overrides and upstream audit limits are tracked in
[`Dependency Maintenance Notes`](../../working/dependency_maintenance.md).

## Test Design

Tests in this repo should protect durable behavior, not the temporary path used
to debug a bug.

Keep tests when they define one of these contracts:

- Core Gomoku/Renju rules and legality.
- Search behavior that a player or report depends on.
- Scan/rolling parity for optimized implementations.
- Wasm/public API shape.
- Replay analysis outcomes and annotation contracts.
- Saved profile, replay, and report schema behavior.

Prefer table-driven scenario coverage over one-off bug fixtures. A regression
test that starts as a narrow bug reproduction should be consolidated into the
nearest scenario table or behavior contract before commit whenever possible.

When several consumers need the same tactical or analysis semantics, keep one
owned mapping/helper below the UI or report layer and test the consumers as thin
adapters. Avoid copying role/outcome switch statements into every route, report,
and bridge path; that makes later tactical wording changes drift-prone.

Avoid tests that only lock internal implementation details:

- Helper names or private call paths.
- Exact intermediate candidate counts, unless the count is a report contract.
- Exact timing or metric values, instead of metric presence/invariants.
- Retired experiment names or compatibility paths we intentionally broke.
- Long replay fixtures when a smaller shape/scenario demonstrates the same
  behavior.

Before committing test changes, do a test hygiene pass:

- Classify new tests as `contract`, `scenario`, `regression-to-merge`, or
  `temporary scaffold`.
- Merge `regression-to-merge` cases into scenario coverage when practical.
- Remove `temporary scaffold` tests.
- Keep slow integration tests only when they protect behavior that cannot be
  covered by a smaller unit/scenario test.
