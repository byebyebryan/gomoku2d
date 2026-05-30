# Testing Guidelines

## Command Matrix

Use the smallest command that protects the contract you changed. Before a broad
release or after core/rules/search work, run the full relevant lane.

| Area | Command |
|---|---|
| Rust formatting | `(cd gomoku-bot-lab && cargo fmt --all --check)` |
| Rust lint | `(cd gomoku-bot-lab && cargo clippy --workspace --all-targets -- -D warnings)` |
| Rust workspace tests | `(cd gomoku-bot-lab && cargo test --workspace)` |
| Tactical scenarios | `(cd gomoku-bot-lab && cargo run -p gomoku-eval -- tactical-scenarios)` |
| Lethal scenarios | `(cd gomoku-bot-lab && cargo run -p gomoku-eval -- lethal-scenarios)` |
| Web typecheck | `(cd gomoku-web && npm run typecheck)` |
| Web unit tests | `(cd gomoku-web && npm test)` |
| Firestore rules | `(cd gomoku-web && npm run test:rules)` |
| Production build/direct routes | `(cd gomoku-web && GOMOKU_BASE_PATH=/ npm run build)` |
| Local browser smoke | `(cd gomoku-web && PLAYWRIGHT_BASE_URL=http://127.0.0.1:8001 npm run playtest:smoke)` |

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
