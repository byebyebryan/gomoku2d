# v0.6 review evidence

October 22, 2026. Local acceptance of application source `f1bc7ee`, following
pushed baseline `0995af8`. This is an unreleased candidate. No tag, deployment,
production account/data writes or real OAuth acceptance occurred in this goal.

The [review plan and findings](../v0_6_review_acceptance.md) define the scope.
[Acceptance receipt](acceptance.json) records the final completion checks.
[Build identity](build_identity.json) records source and bundle SHA-256 hashes
and the selected screenshots. The ordinary bundle has 134 files; the separate
fake-key/demo-emulator bundle has 56. [Compatibility](compatibility.json)
confirms byte-identical backend APIs, rules/indexes, legacy data models and
dependency locks against the pushed baseline. Runtime behavior has separate
test evidence below; source parity alone is not runtime or deployed parity.

Subsequent manual review changed completed continuations to regular My games
and statistics, without a Practice category. The counts and hashes below remain
historical evidence for `f1bc7ee`; later implementation and validation are
recorded in the [manual follow-up](../v0_6_review_acceptance.md).

## Completed checks

| Lane | Result and scope |
|---|---|
| Web units | 45 files / 468 pass, including five recovery regressions and six canonical-encoding scenarios |
| Types | Application, scripts and test tools pass |
| Production build | Pass; canonical shared/player static entries and reports copied |
| Firestore rules/SDK-operation checks | 43/43 pass against the emulator; includes access boundaries, stale writes, sizes and logical operation counts |
| Exploratory browser lane | 9/9 pass across Chromium, Firefox and WebKit |
| Targeted cloud additions | C10/C15: 6/6 pass across three engines |
| Complete browser lane | 201/201 pass, zero skips, unexpected failures or flaky cases; 96.2 seconds |
| Complete cloud lane | 39/39 pass across three engines; 3.9 minutes |
| Library lifecycle repetition | 18/18 pass: C13/C14 × three fresh runs × three engines; 2.1 minutes |
| Production dependency audit | Zero npm vulnerabilities |
| Rust dependency audit | Pass, 100 locked dependencies / 1,348 advisories loaded |

Baseline `0995af8` also passed full
[GitHub CI](https://github.com/byebyebryan/gomoku2d/actions/runs/39725010863),
including Rust formatting, Clippy, workspace tests and 45 repetitions of the
older C4/C6/C7 conflict/account/reset journeys. Rust, legacy profile/history and
dependency sources are unchanged here. The new repetition lane targets the
v0.6 library C13/C14 deletion/reset flows, where this review changed recovery.
New local commits have not been pushed and have no new GitHub CI claim.

Known development-only Firebase CLI advisories retain their documented
[disposition](../dependency_maintenance.md). Production audit is a distinct gate;
no package or lockfile changes were made during this review.

## New coverage and visual review

C10 now checks eight fresh signed-in entries per engine: canonical, case,
repeated-slash and encoded-segment forms of shared replay and public profile.
Shared entry makes zero Firestore requests. Public entry performs public reads
but requests no private profile/library targets. Pending local played history
is not promoted and no private library is created. The local SPA router is
under test; this does not claim noncanonical static paths exist on the public
host.

C15 uses real browser input and production client/Firestore rules to enable an
80-code-point supplementary-Unicode name, edit an 80-character unbroken name,
reject a further typed ASCII character, publish an 80-code-point Unicode title,
and render the result to a guest. Owner/visitor layouts reflow at 320 CSS pixels;
the visitor is also checked at 640 pixels with the root font set to 200%.
This is text-scaling/reflow evidence, not native browser zoom or a physical phone.

The ordinary exploration opens a legal 225-move draw with maximum labels/profile
UID references, checks narrow/enlarged text, accessibility and no Firestore
calls. Another scenario renders HTML/script-looking labels as literal text and
verifies default profile-reference omission, explicit inclusion and generic-name
redaction. Six equivalent-but-noncanonical JSON representations are rejected
rather than silently normalized into shared identity.

The primary inspected these actual Chromium product captures after the fixes:

| Capture | Observation |
|---|---|
| [Replay at 320px](maximum-replay-mobile.png) | Player cards stay compact; full labels remain in accessible text/title and the open Share preview |
| [Owner controls at 320px](maximum-public-owner.png) | Name/link inputs and all management controls remain reachable without horizontal overflow |
| [Guest page at 320px](maximum-public-visitor.png) | Maximum name/title wrap and the replay action remains visible |
| [Guest page with enlarged text](maximum-public-enlarged.png) | Content wraps and actions remain available at 200% root text |

Automated axe scans passed in the new replay/public states. Scans also report
inconclusive checks requiring human judgment; automated scans do not establish
actual screen-reader usability. Existing keyboard completion/branch/replay and
focus-related browser scenarios passed in the full lane; human focus/reading
experience remains part of the walkthrough.

## Artifacts and limits

Local ignored results are under `gomoku-web/test-results/review-browser-final`,
`review-cloud-final`, `review-library-races`, `review-exploration-final` and
`review-cloud-targeted`. Earlier probe failures are retained separately in
`review-cloud-probe`. Only the four selected, inspected PNGs are checked in.
The ordinary runner emitted its JSON summary to stdout; its completion stats
were 201 expected, 0 skipped, 0 unexpected and 0 flaky. Local `.last-run.json`
receipts independently record completed pass/fail state.

No sharing listeners/timers or new backend service were introduced. API method
counts and JSON byte budgets remain those in the
[sharing contract](../../reference/backend/sharing_data.md) and
[operation receipt](../v0_6_evidence/sharing_shape.json), rechecked by the rules
lane. Those counts exclude dependent rules reads, retries, transport/index
overhead and production billing. No measured free-tier traffic capacity or
unbounded load result is claimed. Rollout still requires compatible rules,
separately applied index exemptions, then web publication and live verification.

## Manual walkthrough

Use this v0.6 candidate locally or in a separately authorized staging/release
environment. The public site has not been updated by this goal. For a local
account sandbox, the already built `gomoku-web/dist-cloud` targets only the
demo Auth/Firestore ports. From `gomoku-web`, start these in separate terminals:

```sh
rtk proxy node node_modules/firebase-tools/lib/bin/firebase.js emulators:start --project demo-gomoku2d-acceptance --only auth,firestore --config ../firebase.acceptance.json
```

```sh
rtk npm run preview -- --outDir dist-cloud --host 127.0.0.1 --port 8013 --strictPort
```

Open `http://127.0.0.1:8013/`. The Auth emulator's simulated provider account is
not a real Google consent/sign-in pass. The build is disposable and rebuildable;
if missing, the existing cloud acceptance command builds it before its test run.

1. Finish a normal game and open its replay. Move to an interesting frame, open
   Share, inspect the exported names/profile-link choices and copy the URL.
2. Open the URL in a separate browser context. Confirm labels and starting move;
   try stepping, autoplay and analysis. Decide whether the controls are clear.
3. Save as a guest, reload and find it under Saved replays. Confirm it did not
   change ordinary played-game totals.
4. At an ongoing frame of at least four moves, choose Play From Here. Check the
   displayed side/bot and active-game replacement choice. Finish and inspect /
   re-share the resulting game; it appears in regular My games and totals, while
   the source replay stays unchanged.
5. Sign into a disposable demo account. Explicitly copy guest replays into the
   account if wanted, then inspect them from a second context using that account.
6. Choose and confirm an independent public name. Publish an own played
   replay with a title; open the player URL as a guest and inspect its game.
7. Unpublish and disable the profile. Confirm the listing/page changes while the
   previously copied portable game URL still works. Re-enable starts empty.
8. Judge the wording and focus flow. Use disposable demo data for optional
   Reset/Delete Cloud checks; those have automated coverage and need not dominate
   the human UX pass.

Separate remaining acceptance: real Google consent/account continuity in an
authorized configured environment, actual phone/touch behavior, human
screen-reader and native zoom use, and deployed routes/rules/indexes after an
authorized release. These remain explicit checks rather than inferred passes.
