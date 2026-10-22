# v0.6 Implementation Roadmap

Owner: Bryan. Started October 21, 2026. Status: M0–M5 implemented under the
authorized Luna goal loop; all locally runnable M6 checks pass. The execution
record owns the acceptance evidence and separate release gates. The selected release combines public profiles and
shareable replays. The [design draft](v0_6_profiles_and_replays.md) owns the
product rationale and proposed behaviors; this plan owns order and acceptance.

October 22 manual-review revision: completed continuations now use regular
My games, statistics and history sync, retaining their origin internally.
This supersedes the earlier separate practice-category acceptance policy.

## Readiness and Scope

The milestones below preserve the implementation sequence and acceptance
contract. Product defaults and storage/wire bounds are settled in the execution
record and canonical reference contracts; they no longer await implementation.
The complete v0.6 release includes both a useful guest replay journey and an
opt-in public profile with explicit game publication.

Preserve the existing UI language and ordinary local play. Reuse the current
Rust/Wasm rules and analyzer, static hosting, Firebase Auth, and Firestore.
Multiplayer, server authority/analysis, public rankings, global discovery,
social features, uploaded avatars, and a broad UI redesign remain deferred.

The goal-loop instruction accepts the recommended scope below, including
own-game publication and signed-in Saved replays sync. The
[execution record](v0_6_execution.md) tracks decisions, commits, and evidence.
Public profiles and catalogs do not depend on cross-device private-library sync.

## Milestone Sequence

| Milestone | Result visible to a player | Exit evidence |
|---|---|---|
| M0: Contracts and decisions | Agreed behavior for sharing, saving, practice, and publication | Bounded contracts and compatibility decisions recorded |
| M1: Portable replay core | A valid finished game can become a portable replay and back | Versioned fixtures, legal replay round trips, input limits |
| M2: Share and guest viewer | Copy a chosen moment; another browser opens and studies it | Production-build guest demo with no replay/profile database calls |
| M3: Local library and continuations | Save a study copy separately; played continuations join regular My games | Reload, deduplication, origin retention, and normal played-game statistics |
| M4: Public profile and publishing | Share a player page containing explicitly published games | Public/private access rules, publication lifecycle, measured operations |
| M5: Account library sync | Saved copies follow the signed-in account | Two-client sync, account isolation, offline deletion/conflict recovery |
| M6: Release acceptance | Complete solo demo and compatible release package | Browser, cloud, cost, compatibility, and release evidence |

M0 through M3 form the first end-to-end demo. M4 and M5 both build on that
foundation. Public profiles come first so both headline features can be demoed
before private-library sync; M4 does not require the M5 storage implementation.
Integrate changes in small reviewed commits, keeping each milestone runnable.
These milestones are checkpoints within v0.6, not promises of separate public
releases.

## M0: Settle Contracts Before Their Writers Exist

Keep this a bounded contract pass, not another general research phase.

| Decision | Recommended starting point | Needed before |
|---|---|---|
| Public publication scope | Own played games, including continuations; imported curation deferred | M4 |
| Private library persistence | Guests local; signed-in account sync in M5; guest import is explicit | M3 storage contract / M5 |
| Public name | Confirm an independent public name, prefilled from the in-game name; no automatic provider publication or coupled renaming | M4 |
| Continuation statistics | Completed continuations count as regular played games; keep source/starting ply as metadata, with no badge or separate list | M3 |
| Visibility lifecycle | Explicit publish; disabling hides and removes publications; re-enabling starts empty | M4 |

Before M1, specify the versioned portable record, legal game/opening contract,
UTF-8/encoded-size bounds, player/bot metadata, optional practice origin,
canonical identity, and handling of unsupported input. Viewing ply does not
change saved replay identity. Decide how changed attribution affects repeated
saves. Build representative size fixtures before freezing the link limit.

Before M3/M5, choose a versioned private library and practice representation,
retention caps, guest/account namespaces, merge/deletion rules, and migration
behavior. Preserve existing history and archived totals unless the chosen
migration can reconstruct their provenance. Saved-library records stay separate
from ordinary played-history promotion.

Before M4, use actual encoded fixture sizes to compare bounded slots against
capped catalog pages/manifests. Select field/query limits, write coordination,
publication invalidation, deletion cleanup, and a way to disable public access.
Storage rules must enforce the bounds, including batched or stale writes.

For every persisted schema change, update the reader, writer, rules, tests, and
[canonical data model](../reference/backend/data_model.md) in the same slice.
Specify old-tab behavior: preserve new fields or reject incompatible writes
with recovery; never silently downgrade or erase them. Shared wire-format
versions and private storage versions evolve independently.

Exit: M1's contract is concrete enough to implement. Later decisions have an
owner and a named dependency; no unresolved choice silently becomes a default.

## M1: Portable Replay Core

- Implement a dedicated portable model and adapters from finished saved games;
  do not serialize the private record wholesale.
- Encode/decode the complete move sequence, names, optional profile references,
  bot identity, and bounded practice origin. Keep initial viewing ply separate
  from canonical identity.
- Validate bounds before expensive work, then replay with Rust/Wasm to derive
  the result and reject illegal moves or moves after termination.
- Reuse the same validation for URL, library, and public catalog inputs.

Primary touchpoints: [saved matches](../../gomoku-web/src/match/saved_match.ts),
the [replay module](../../gomoku-web/src/replay/), and the existing Wasm bridge.

Exit: fixtures cover both rules variants, human/bot metadata, practice origin,
long/Unicode labels, largest supported payloads, invalid/unsupported inputs,
and stable identity across viewing positions. No public database is needed.

## M2: Share and Standalone Guest Viewer

- Separate replay input/presentation from the private-history loader. Preserve
  existing private replay links and the analyzer's confidence semantics.
- Add Share preview and Copy link at the final frame or current ply, with a
  preview of the names and profile references that leave the device.
- Add the fixed shared route, direct-load static entry, original player labels,
  loading/unavailable states, and existing replay transport/analysis controls.
- Gate private cloud initialization on the shared/public route family. Copying
  or viewing a portable replay must not load/promote/sync private history,
  including for a returning signed-in visitor. This does not cancel work a user
  explicitly initiated earlier; test fresh entry separately from navigation.

Primary touchpoints: [replay loader](../../gomoku-web/src/replay/use_replay_match.ts),
[replay panels](../../gomoku-web/src/routes/ReplayRoutePanels.tsx),
[app shell](../../gomoku-web/src/app/App.tsx),
[cloud controller](../../gomoku-web/src/cloud/CloudSessionController.tsx), and
[static routes](../../gomoku-web/scripts/publish_spa_routes.mjs).

Exit: a finished local game becomes a link that opens at the selected ply in a
fresh guest browser and a fresh signed-in session. Scrubbing/analysis succeeds
without import or replay/profile database traffic. Verify clipboard fallback,
desktop/mobile layout, keyboard controls, and successful direct HTTP entry in
the production build; verify the real public host separately at release.

## M3: Local Saved Replays and Continuations

- Add My games and Saved replays to the private profile without merging their
  data or statistics. Save is explicit, survives reload, and handles full or
  denied storage; repeated saves do not overwrite different attribution.
- Add local Remove and capacity/recovery behavior. Scope guest and account
  caches separately, even before cloud sync is enabled.
- Connect Play From Here to the existing legal-frame/current-bot setup, show
  that setup, and confirm replacement of an active local match.
- Save a completed continuation as a regular played game, with optional source
  and starting-ply metadata retained through private history, cloud promotion,
  and export. Include it in normal statistics without a separate badge/list.
- Keep saved study copies outside played-history promotion. Earlier preview
  library completions appear in the owner's regular My games and displayed
  totals without copying private account data into device-wide history.

Primary touchpoints: [private profile](../../gomoku-web/src/routes/ProfileRoute.tsx),
[local profile storage](../../gomoku-web/src/profile/local_profile_store.ts),
[match history](../../gomoku-web/src/match/match_history.ts), and
[local match setup](../../gomoku-web/src/game/local_match_store.ts).

Exit: open, save, reload, remove, and branch without signing in. The source
remains unchanged; imports do not count as played games. A re-shared completed
continuation still identifies its inherited prefix. Existing local history,
settings, finish/replay flow, and supported old records retain their behavior.

## M4: Public Identity, Catalog, and Full Publication Lifecycle

- Implement the selected bounded storage/rules design in the emulator first.
  Public projection reads never grant access to the private profile/library.
- Add public-profile opt-in/name preview and editing, its stable public link,
  and the fixed public entry route with empty/unavailable/retry states.
- Add Publish preview, bounded catalog browsing, and opening any entry through
  the shared viewer. Distinguish the authenticated publisher from unverified
  participant claims; every copied replay link remains self-contained.
- Implement Unpublish, Disable/re-enable, and expanded cloud deletion alongside
  publication. Block reads before cleanup finishes; expose pending failure and
  retry. Stale tabs and reused slots must not resurrect old content.
- Record any retained deletion/control markers honestly in the data contract.
  Add the public-access disable procedure and appropriate data-visibility copy.

Primary touchpoints: [profile UI](../../gomoku-web/src/routes/ProfileRoute.tsx),
new public page/catalog modules, rules/index configuration, static routes, and
the backend/data-model/cost/operations reference docs.

Exit: an unauthenticated visitor can browse the chosen public publications and
study one; private reads, broad profile-directory queries, and public writes
remain denied. Owners can publish and remove content. Validate bounds
against malicious batches, stale operations, account switches, partial cleanup,
and delete/recreate sequences. Measure reads, writes, retained bytes, and egress
for empty/full pages and publication changes; do not infer a traffic allowance
from the per-account cap. Portable replays still work with public access off.

## M5: Private Account Library Sync

- Add lazy owner-only sync for the separate bounded library; do not piggyback
  every save on the large played-history snapshot or initialize it on a visit.
- Expose local/pending/synced/error states and explicit guest import. Use UID
  namespaces for caches, requests, and pending operations.
- Implement the chosen merge/deletion/reset contract with bounded metadata and
  compaction behavior; a stale/offline device must not restore deleted saves.
- Extend private reset/delete flows and their visible scope. Test races before
  considering this persistence complete.

Primary touchpoints: [cloud modules](../../gomoku-web/src/cloud/),
[Firestore rules](../../firestore.rules), and the profile/library UI.

Exit: two emulator clients converge on saves and removals, and recover from
offline writes, conflicts, denied requests, reset, sign-out, and account switch.
Measure the actual save/load operation shape. Ordinary guest and signed-in
shared-link entry still performs no private-library operation.

## M6: Release Acceptance and Rollout

Run this solo demonstration against the complete candidate:

1. Finish a game and copy a link at an interesting moment.
2. Open it as a fresh guest; scrub, analyze, and save a private copy.
3. Continue from a legal ply, finish into regular My games/statistics, and re-share it.
4. Enable a public profile, publish a chosen game, and share the player page.
5. Browse as a guest, open its replay, then verify unpublish/disable behavior.
6. If included, recover the saved library on a second emulator client signed
   into the same account.

Use the relevant lanes in [Testing](../reference/ops/testing.md): web types/unit
tests, rules, production build/direct routes, three-engine desktop/mobile and
keyboard journeys, and emulated cloud continuity/races. Run the existing release
checks and dependency audits; rerun Rust-specific behavioral suites if rules or
bridge changes warrant them. Unchanged bots/reports do not start a new research
or benchmark phase. Record real OAuth/device/accessibility evidence and existing
explicit deferrals separately; emulator/browser results do not replace them.

Before production, document web/rules deployment order, schema coexistence for
older open tabs, and rollback compatibility. Follow the existing
[release workflow](../reference/ops/release.md): preparation, authorized push,
CI on the exact candidate, tagging/deployment, then live route/rules and scoped
journey verification. Implementation, deployment, and live acceptance are
separate checkpoints. This roadmap authorizes no deployment or production data
mutation and claims no measured free-tier capacity.

## Next checkpoint

The later [unattended review](v0_6_review_acceptance.md) is complete. Follow the
[manual walkthrough](v0_6_review_evidence/README.md#manual-walkthrough) against the
reviewed local candidate; the report separates current evidence from the
original [implementation receipts](v0_6_evidence/README.md).
Release preparation may then choose a version/date, obtain authorized push and
exact-head CI, deploy compatible rules/indexes before the web release, and run
the public-host/account journey. This implementation goal authorizes none of
those publication steps and does not turn deferred manual checks into passes.
