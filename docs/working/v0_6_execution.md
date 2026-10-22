# v0.6 Execution Record

Started October 21, 2026. Owner: Bryan; implementation and acceptance owned by
the primary with bounded Luna Max assignments under the explicitly invoked
`luna-goal-loop` skill.

## Outcome and Boundaries

Implement M0–M5 in the [implementation roadmap](v0_6_implementation_plan.md),
then pass its locally runnable M6 acceptance. Commit reviewed slices locally.
Do not push, tag, deploy, or write production data. Preserve the current visual
language, ordinary play, account/history/settings contracts, and existing
explicit manual-check deferrals. The goal is a complete local candidate, not
a deployed release. No token budget was requested.

## Accepted Product Defaults

- Publish only the account's own played games or its clearly marked practice
  completions; importing someone else's replay does not grant publication.
- Confirm an independent public name, initially suggested from the in-game
  name. Do not automatically publish provider metadata.
- Save guest copies locally and sync signed-in libraries separately. Guest
  import is explicit; account caches and pending work are scoped by UID.
- Exclude new practice completions from ordinary win/loss totals. Do not
  reconstruct historical practice statistics from archived aggregates.
- Publish each game explicitly. Disable makes the public page unavailable and
  removes publications; re-enable starts empty. Portable copies cannot be
  recalled. Removal/reset/delete scopes are visible before destructive actions.
- Resolve routine format/layout/cap decisions from bounded tests and measured
  payloads, recording consequential choices here and in the canonical contract.

## Architecture Decisions

- Preserve existing `SavedMatchV2` and schema-5 cloud profiles. Keep imported
  replays and new practice completions in a separate versioned replay-library
  family, with practice exposed separately in My games and excluded from
  ordinary played-history aggregation/promotion. Older clients cannot strip
  provenance from a record family they never write.
- Portable canonical identity includes participant snapshots and practice
  origin, but excludes the initial viewing ply. Different attribution creates
  a separate copy; identical repeated saves deduplicate. Fingerprints establish
  content identity only, never authenticated participation.
- Public catalog: four fixed page documents, eight entries each (32 total),
  with bounded ASCII portable payloads. The initial visitor view reads identity
  plus one page, not eight separate game documents. Enable/disable rewrites the
  identity and all four pages atomically. Revisions reject stale page writes;
  generation changes invalidate previous publications. Validate this layout
  against format measurements and emulator rules before acceptance.
- Public identity and private library retain empty control documents after
  cloud removal, with no names/game payloads. Monotonic revision barriers
  reject stale writes after removal; the UI must disclose retained markers.
- Private library: a separately loaded, capped snapshot, transactionally
  compared against the revision the client actually loaded. Concurrent/offline
  conflicts require explicit reload/review; they never silently replay stale
  additions over a deletion. No listener or background polling is needed.
- Public and private state are separate. No gameplay move, replay scrubbing,
  or analysis operation uses a backend.

## Milestones

| Milestone | Status | Evidence |
|---|---|---|
| M0 Contracts | Active | Product defaults and portable format settled; backend limits under emulator validation |
| M1 Portable core | Complete | 16 actual-Wasm contract tests; [wire contract](../reference/app/portable_replay.md) |
| M2 Shared viewer | Implemented | Guest direct-load/share/fallback: 15 checks across three engines; signed-in isolation awaits cloud acceptance |
| M3 Local library/practice | Implemented | Store/session races and Chromium guest journeys pass |
| M4 Public profile/catalog | Implemented | 12/12 new cloud checks across three engines; final regression lane pending |
| M5 Private library sync | Implemented | Two-client sync/conflict journey plus reset/delete/action race contracts pass |
| M6 Local acceptance | Pending | |

## Work and Validation Log

- Preflight: branch `main`, two existing unpushed design/research commits, and
  only the known roadmap documentation edits present. No application edits.
- Luna Max available and assigned a read-only investigation of existing replay,
  Wasm validation, and bot identity contracts before its M1 implementation slice.
- Local node dependencies, generated Wasm package, and all three Playwright
  engines are present.
- Baseline `npm run typecheck`: pass. Baseline `npm test`: 39 files / 378 tests
  pass. No application implementation was present for these baseline runs.
- M1 Luna implementation reviewed by primary: strict portable codec, SHA-256
  identity, legacy adapters, and actual-Wasm rejection of forbidden Renju moves,
  invalid terminal progression, and malformed metadata/encodings. Focused
  `npx vitest run src/replay/portable_replay.test.ts`: 16 tests pass.
- Maximum fixture (225 moves, two maximum Unicode names/UIDs, practice origin):
  1,353 decoded JSON bytes / 1,804 encoded payload characters, below the v1
  6,144-byte / 8,192-character bounds. This measures payload size, not a
  cross-application URL transport or hosting acceptance pass.
- M2 primary integration: reusable replay presentation, original shared labels,
  optional profile links, share preview/redaction and clipboard fallback,
  static entry, and separate authentication/private-cloud lifecycles.
- `playtests/shared_replay.spec.ts`: five Chromium checks pass on the host;
  all 15 checks pass across Chromium/Firefox/WebKit in the supported Ubuntu
  container with the lockfile's Playwright 1.64.1 browser cache mounted. Host
  WebKit cannot launch due to missing system libraries; the cached Ubuntu image
  supplies those dependencies. This is production-build local browser evidence,
  not public deployment or real OAuth acceptance.

## Acceptance Limits

No implementation, local candidate acceptance, production deployment, real
OAuth/device acceptance, or free-tier traffic envelope is claimed at this point.
Update this record as actual evidence becomes available. Runtime test artifacts
must identify the candidate and remain distinct from manually reviewed UI and
physical-device evidence.

- M4 backend Luna handoff: bounded public and private schema-1 families, atomic
  lifecycle operations, stale revision/generation checks, public kill switch,
  index exemptions and 41 passing sharing-rule emulator checks. Primary reviewed
  the additive rules; existing schema-5 profile behavior remains unchanged.
- Primary integrated library sync and explicit conflict recovery, shared saves,
  My games/Saved replays tabs, practice presentation, public route/management and
  publication panels. Reset/delete confirmations now disclose exact scope and
  retained markers. Delayed sync and explicit-merge concurrency tests pass.

- M3 Luna store/session implementation and primary route integration: guest save,
  identity deduplication across plies, reload/removal, legal practice completion,
  source preservation, re-share, and zero played totals pass Chromium journeys.
  Async local replacement/clear tests reject stale hashes without losing new work.
- M4/M5 Luna public UI and cloud journey work plus primary sync/lifecycle review:
  C10–C13 pass 12/12 in Chromium/Firefox/WebKit, including fresh signed-in shared
  entry with zero Firestore work, 390px layouts, publication lifecycle, two-client
  removal and stale-client review. Full legacy cloud regression remains pending.
- Review found and fixed pre-clear pending Save/import intent, same-revision
  post-delete review bypass, and account-switch partial-cleanup reporting. These
  now have focused contract coverage. No user-visible success is claimed after
  a guarded cleanup abort.
- Production build and web typecheck pass; all 429 tests passed before the last
  lifecycle regressions were added. Production npm audit: zero vulnerabilities;
  Cargo audit: clean against the current advisory database. Rust/bot/core sources
  were unchanged, so no new tournament or behavioral Rust run was required.

- Final lifecycle review added a conditional old-profile deletion barrier: legacy
  deletion is rejected while new public/private data remains. Never-sharing
  accounts keep prior behavior. All 43 rules checks pass with this guard.
- Replacement hashing now accepts an operation/account guard, so reset/delete
  or account changes invalidate an already reviewed local replacement before
  it commits. Focused storage/sync/profile contracts pass 46/46 after this fix.
- Full first browser regression: 168 passed, 21 failed solely on outdated Profile
  heading selectors and practice/played-history expectations. Updated those
  scenarios to require one ordinary history record plus separate practice and
  unchanged played totals; rerunning all 189 checks, including new-state axe scans.
