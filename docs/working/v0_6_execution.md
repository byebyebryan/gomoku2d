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

October 22 manual review revised continuation behavior as described below.
The milestone receipts later in this record describe the earlier implementation;
the [review follow-up](v0_6_review_acceptance.md) records subsequent checks.

- Publish only the account's own played games, including replay
  completions; importing someone else's replay does not grant publication.
- Confirm an independent public name, initially suggested from the in-game
  name. Do not automatically publish provider metadata.
- Save guest copies locally and sync signed-in libraries separately. Guest
  import is explicit; account caches and pending work are scoped by UID.
- Completed continuations are regular My games and count in ordinary win/loss
  totals. Keep the source and starting ply as metadata, with no Practice badge
  or separate section. Existing archived aggregates are retained.
- Publish each game explicitly. Disable makes the public page unavailable and
  removes publications; re-enable starts empty. Portable copies cannot be
  recalled. Removal/reset/delete scopes are visible before destructive actions.
- Resolve routine format/layout/cap decisions from bounded tests and measured
  payloads, recording consequential choices here and in the canonical contract.

## Architecture Decisions

- Keep saved study copies in a separate versioned replay library. New completed
  continuations use normal history, promotion and statistics, with optional
  SavedMatchV2 `replay_origin` metadata carried into portable export. Schema
  versions remain 2 and 5. Earlier preview `kind: practice` library records are
  projected into the current owner's My games and displayed totals without
  moving them into device-wide history.
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
| M0 Contracts | Complete | Versioned portable/library/public contracts and bounded limits validated |
| M1 Portable core | Complete | 16 actual-Wasm contract tests; [wire contract](../reference/app/portable_replay.md) |
| M2 Shared viewer | Complete | Guest direct-load/share/fallback plus signed-in isolation validated in three engines |
| M3 Local library/practice | Complete | Store/session races and three-engine guest/practice journeys pass |
| M4 Public profile/catalog | Complete | Three-engine publication/visibility, rules and measured operations pass |
| M5 Private library sync | Complete | Two-client sync/conflict and reset/delete/action race contracts pass |
| M6 Local acceptance | Complete locally | 433 units, 43 rules, 189 browser, 36 cloud and 45 repeated race checks; build/audits/static routes pass |

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

Implementation and local acceptance evidence are recorded here and in the
[evidence ledger](v0_6_evidence/README.md). No production deployment, real
OAuth/device/screen-reader acceptance or free-tier traffic envelope is claimed.
Those checks remain separate, with existing explicit manual deferrals retained.

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

- Final browser candidate: all 189 checks pass across Chromium/Firefox/WebKit,
  including new shared/library axe scans. Final web units: 44 files / 433 pass;
  complete typecheck passes. Public manager/publish/visitor axe scans also pass
  in the new cloud journeys. Full cloud continuation scenarios are being updated
  to the new separate-practice contract before final acceptance.

- Complete final-source cloud application lane: 36/36 pass in 3.1 minutes across
  Chromium/Firefox/WebKit. C3/C5 now verify real practice finish, explicit sync,
  second-client viewing, offline reload/recovery and no ordinary-history import.
  C14 proves private reset preserves the public projection byte-for-byte, while
  Delete Cloud clears it and retains local copies behind explicit review after
  same-UID sign-in/reload. Final 45-run conflict/account/reset lane is running.

- Bryan reported unintended browser tabs during acceptance. Installed Vite resolves
  preview `open` from `server.open` when omitted, so every host preview inherited
  the development auto-open setting. Set `preview.open: false`; resolved config
  confirms preview false / development true. Closed the one remaining owned
  `127.0.0.1:8013` tab via browser controls and verified none remain, preserving
  the five other user tabs. This changes preview startup, not the web bundle.

- Final cloud race lane: 45/45 pass across Chromium/Firefox/WebKit, C4/C6/C7 five
  times each, in 3.2 minutes. This is a complete fresh-state run, replacing the
  interrupted earlier selector-debug run. No failures or required local checks
  remain. Routine test screenshots now stay in test output unless explicitly
  refreshing checked-in evidence.
- Final build after the preview-only fix: all 134 dist files are byte-identical
  to the browser-qualified candidate. Static-route, build-identity and unchanged
  schema-source receipts are in the evidence directory. Types, whitespace and
  changed-doc relative links pass. Test preview/emulator ports are closed.

## Checkpoint outcome

M0–M5 and all locally runnable M6 gates are complete. Luna Max implemented the
portable core, library/session foundation, public UI/backend and browser/cloud
acceptance slices; the primary owned integration, lifecycle/concurrency fixes,
contract review, final gates and local commits. New sharing requires no Cloud
Run service, move relay or server analysis. Production, real OAuth/devices and
human accessibility remain separate as specified above. The next checkpoint is
review and separately authorized release preparation, not more implementation.

## Post-review fixes

After `da0491a`, Bryan requested fixes for both review findings:

- Public/shared entry now uses React Router's matcher for the private-session
  guard, including case variants, repeated trailing slashes and encoded path
  segments. Component checks prove these entries never mount the private cloud
  controller and ordinary private navigation still enables it.
- Save and library messages distinguish session-only fallback from device
  persistence. They disclose reload/close loss and advise keeping the replay
  link. Confirmed cloud copies retain their account-saved message when only the
  device cache fails.

Both defects were reproduced by failing regression tests before the fixes.
Final scoped checks: 56 unit/component tests across five files, complete web
typecheck, production build and 24 shared/library browser journeys across all
three engines pass. The browser lane includes a real injected quota failure,
reload loss and mobile accessibility/overflow checks. Full earlier matrices and
checked-in build/capture receipts remain evidence of the original checkpoint;
the follow-up did not rerun the full cloud suite. No release or deployment was
performed.

## Unattended review closeout — October 22

Bryan authorized a planned goal loop before the manual pass. The
[review plan](v0_6_review_acceptance.md) and
[acceptance report](v0_6_review_evidence/README.md) record the complete scope.
Pushed baseline `0995af8` passed full GitHub CI. Local checkpoints `ae6364c`,
`7f1e1a7` and `f1bc7ee` add the plan, recovery-barrier fix and Unicode/compact-label
fixes. The primary performed this bounded review and retained four inspected
product captures; no new delegation, UI research or infrastructure was added.

Final application source `f1bc7ee`: types/build, 468 web tests, 43 rules checks,
201 browser checks and 39 cloud journeys pass. Library C13/C14 repetitions pass
18/18 across three fresh runs per engine. Production npm/Rust audits pass;
backend/rules/schema/dependency byte parity is recorded separately. No known
blocking findings remain. New commits are local, with no new push, version bump,
tag, deployment or production data mutation. The next checkpoint is the short
manual walkthrough in the report, with real OAuth/physical/human-accessibility
acceptance still explicit.
