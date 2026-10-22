# Release Workflow

Purpose: release checklist for the web game and repo-level tags.

This file owns release sequencing. Test commands live in
[`testing.md`](testing.md). Tournament/report generation lives in
[`tournament.md`](tournament.md).

## Public Host

Canonical URL:

```text
https://gomoku2d.byebyebryan.com/
```

GitHub Pages serves the custom domain root. Production builds use
`GOMOKU_BASE_PATH=/`.

## Local Preview

Use a foreground preview on port `8001` (stop it with Ctrl-C):

```sh
wasm-pack build gomoku-bot-lab/gomoku-wasm --target bundler
cd gomoku-web
npm ci
npm run build
npm run preview -- --host 127.0.0.1 --port 8001 --strictPort
```

Open `http://localhost:8001/`. Verify the listener before a test that sets
`PLAYWRIGHT_BASE_URL`; that option delegates server/build ownership to you.
For an automated run with owned startup/cleanup, leave the variable unset and
run `npm run playtest`. Do not terminate an unrelated preview to claim its port.

## Release Prep

1. Decide whether screenshots/OG/README hero need refresh.
2. If captures should show the new version, run:

   ```sh
   scripts/set-web-version.sh x.y.z
   ```

3. Run relevant checks from [`testing.md`](testing.md).
   Production npm and Rust dependency audits are required for every release.
4. Refresh curated reports only when bot/analyzer/report behavior or source data
   changed; use [`tournament.md`](tournament.md).
5. Update `CHANGELOG.md` with an empty `[Unreleased]` section and dated release
   section.
6. Record the available account, device, and accessibility acceptance evidence
   and any explicit deferrals. Bryan's [October manual-check deferrals](../../archive/reconciliation_2026_10/README.md#deferred-acceptance)
   remain in force; carry their limits into the release decision rather than
   recording them as passes. When a real Google sign-in or two-device
   settings/history/reset round trip is performed, use disposable test data.
   Emulator and no-config checks establish a different level of evidence.
7. Review `git diff` for accidental generated or scratch output. Stage only the
   reviewed paths, commit and push the release preparation, and verify CI on
   that exact commit before tagging. The release script refuses dirty trees
   and never stages or commits files for you.

## Report Artifact Gate

Before a release that includes reports:

- curated report JSON is committed under `reports/lab/`;
- report provenance says `"git_dirty": false`;
- `GOMOKU_ALLOW_MISSING_REPORTS=1` is not used for the production build;
- web build copies `/bot-report/report.json`, `/analysis-report/report.json`,
  and all SPA route entries configured in `publish_spa_routes.mjs`.

## Push And CI

```sh
git push origin main
gh run list --branch main --limit 5 --json databaseId,headSha,status,conclusion
gh run watch <run-id> --exit-status
```

Confirm the successful run has `headSha == git rev-parse HEAD`. If a run fails:

```sh
gh run view <run-id> --log-failed
```

## Cut Release

After CI passes on the exact preparation commit, run from the repo root:

```sh
VERSION=x.y.z
scripts/release.sh --check "$VERSION"
scripts/release.sh "$VERSION"
git show "v${VERSION}"
git push origin "v${VERSION}"
```

The release script validates clean, pushed main and version/changelog/tag state,
then creates an annotated local tag. It does not push. The explicit tag push
dispatches GitHub Release, Pages deploy, and Firestore rules deploy workflows.
Wait for all three workflows to succeed before treating publication as complete.
After deployment, verify the public URL with the applicable browser journeys and
read-only checks; record any remaining manual acceptance deferrals.

## Recovery

A code push is not deployment. Record the Pages deployment run/tag/SHA and the
Firestore workflow's previous/current ruleset IDs at release time. After a
release, verify Home, direct public routes, and a fresh local finish/replay/branch
against the public origin. Record an account-level cloud round trip separately
when performed; otherwise retain its explicit acceptance deferral.

For a web regression, dispatch the Pages workflow at the previously accepted
tag/ref and verify its deployment/SHA and public behavior. Do not move release
tags. For a rules regression, restore the recorded previous Firestore ruleset
release pointer through the authenticated operational path and read it back;
redeploying main is not a rollback. Coordinate web/rules compatibility before
choosing either. No rollback or deployment is performed by the reconciliation
pass. Private replay URLs use Pages' `404.html` SPA fallback and can return HTTP
404 on direct entry even when the client route works; public static routes have
copied `index.html` entries.

## v0.6 sharing rollout and compatibility

The v0.6 goal produces a local candidate only. Version bump, push, exact-head CI,
release tag, deployment and public acceptance remain separate authorizations.
The package version stays 0.5.5 and notes remain Unreleased until release prep.

For the first sharing release:

1. Record the currently served web SHA and previous Firestore ruleset ID.
2. After an authorized push and exact-head CI, manually dispatch the existing
   Firestore workflow on reviewed `main` **before** publishing the new web app.
   This additive ruleset retains schema-5 profile writes and opens the two
   schema-1 sharing families. Verify its live release pointer and scoped access.
3. Apply the reviewed `firestore.indexes.json` exemptions through an authorized,
   scoped index deployment. The existing rules-only workflow does not deploy
   indexes. Record their actual applied state; checked-in JSON is not evidence
   of deployed exemptions. New functionality uses direct document gets and does
   not require composite indexes, but payload indexing affects retained cost.
4. Publish the web release using the established workflow. Tag workflows run
   independently, so do not rely on their scheduling to provide rules-first
   order. Verify direct HTTP success for `/replay/shared/` and `/player/`, then
   disposable-account publication, guest viewing and cleanup against the host.

Compatibility and rollback:

- No version bump of `profiles/{uid}` schema 5 or `SavedMatchV2` schema 2.
  Completed continuations enter normal history and totals with additive optional
  `replay_origin` metadata; existing validators and promotion preserve it. Older
  clients ignore the new Firestore families and local library key. They can view
  continuation matches but their export path does not include the new origin.
  Earlier preview library completions remain readable in the current My games UI.
- An old client trying to delete a profile while public identity or a nonempty
  library remains receives a rules denial. It must update/reload and use the
  current cleanup flow. Accounts without sharing data retain prior deletion.
  Rules may charge up to two additional dependent document reads for deletion.
- Empty public/profile-page/private markers must remain. Do not delete them
  administratively to reclaim a tiny amount of storage: stale revision barriers
  depend on them. Re-enable starts empty; retained local copies require explicit
  merge after cloud removal.
- Prefer a compatible forward fix or a previous v0.6 candidate that retains the
  new cleanup controls. If rolling web back to v0.5, keep additive rules and
  explain that old tabs cannot manage the new library/publication families.
  Never bypass the deletion guard to make old Delete Cloud appear successful.
- For a public incident, first set `publicSharingEnabled()` to `false` in the
  reviewed rules source and deploy/read back that ruleset through the same
  authorized path. This blocks visitor reads and new enabling/publication while
  leaving owner disable/clear and the private library usable. Portable URLs
  still work. The switch is not a traffic quota or a revocation mechanism.
- Restoring the old v0.5 rules closes new families and prevents current cleanup;
  it is not a compatible sharing rollback. Record and verify the selected
  web/rules pairing instead of treating either deployment alone as recovery.

Local fixtures and limits live in the [candidate evidence](../../working/v0_6_evidence/README.md).
Real OAuth, physical devices, human screen-reader use and public-host acceptance
remain separate from emulator and desktop-engine automation.
