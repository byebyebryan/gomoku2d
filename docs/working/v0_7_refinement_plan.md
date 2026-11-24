# Gomoku2D 0.7 refinement plan

Reviewed November 20, 2026. The baseline is the published
[v0.6.5 release](../archive/v0_6_5_release.md) at `25d67c1` and its
documentation closeout at `e6c8999`. The
[product roadmap](../reference/product/roadmap.md) owns adopted sequencing.

## Release preparation — November 24, 2026

The versioned rehearsal used `0e380ec` and the reviewed documentation diff.
It passed 42 focused browser checks, 24 cloud continuity and reset checks,
12 publication checks across Chromium, Firefox, and WebKit, and 202 delivery
checks. A separate Chromium pass covered 14 viewport checks at 200% browser
zoom or compact emulated touch. Receipts are in
`gomoku-web/test-results/v070-cut-readiness/`. The release preparation also
includes the focused Play From Here cancellation fix. CI on the exact versioned
commit remains the gate for tagging.

The live 0.6.5 Chrome check signed out of the app and returned through Google
to the existing account. It published and edited an isolated test replay,
verified the selected opening and all five statistics values while signed out,
then unpublished the test listing and turned statistics back off. The original
listing and private history were preserved. A copied portable link still
opened after removal. This check used an existing Google session and an
already-enabled profile. First-time Google consent and new public-profile
setup remain open in production. The affected production smoke checks must
run against the deployed 0.7.0 build.

Physical touch and software keyboard use, lower-powered hardware, human
screen-reader use, two-device continuity, and novice feedback retain their
recorded deferrals.

## November 21 integration snapshot

The integrated logs in `gomoku-web/test-results/v070-refactor-integration/`
were run from `2345b66` and an isolated working-tree snapshot. Its
`source-shas.json` manifest covers 34 pending paths for those runs. The
presentation checks below cover the later Dev review, media, and asset
publication work.

| Local check | Result | Receipt |
| --- | --- | --- |
| Rust workspace | 354 tests passed | `rust.log` |
| Web | Typecheck passed, 610 Vitest tests passed | `web.log` |
| Tooling | 48 checks passed | `web.log` |
| Browser journeys | 315 passed, six expected Chromium-only WebGL cases skipped in Firefox and WebKit | `browser.log` |
| Auth/Firestore emulator journeys | 75 passed across Chromium, Firefox and WebKit | `cloud-chromium.log`, `cloud-firefox.log`, `cloud-webkit.log` |
| Repeated cloud races | 45 passed | `cloud-races.log` |
| Firestore rules | 51 passed | `rules.log` |

The candidate Dev review at `/dev/reviews/replay-continuation-070/` compares
Match → Replay → Play From Here → Source replay on desktop and compact
layouts. The [UI evidence](v0_7_ui_evidence/README.md) records its inputs and
the refreshed README captures. The original gallery is labeled 0.6.2 screen
reference at `/dev/reviews/core-screens-062/`. The Lab keeps the recorded runs
and links to the [engine baseline](#bot-and-analysis-baseline).

The local presentation build passed typechecking, 610 web unit tests, 50
tooling tests, and 114 focused browser tests across Chromium, Firefox, and
WebKit. Both standalone exports passed their browser checks from nested URLs.
The Pages build matched 495 existing payloads against v0.6.5, including Visuals
assets, the social image, historical reviews, UI history, and Lab reports.
The developer export matched the 494 applicable Visuals, review, history,
and report files.
Local receipts are in `gomoku-web/test-results/v070-presentation-closeout/`.

Use 0.7 to make the existing game more reliable, easier to learn, and easier
to work on. **Play → Replay → Try another line** is the journey that connects
the app, the Rust bot and analyzer, the UI, and the Dev site's design evidence.
Guest play, replay and analysis stay in the browser. Keep the same Replay
Analysis surface, board-first layout, opt-in publication, and static hosting.

## Sequence

| Milestone | Work and outcome | When to proceed |
| --- | --- | --- |
| Correctness checkpoint | Preserve replay continuations across navigation, finalize completed rounds, and keep the result and Analyze reachable on compact screens. | Confirm guest and emulator behavior. |
| **0.7.0 refinement** | Improve Play → Analyze → Try another line and My games → Inspect → Return. Clarify match and replay ownership, explain the Guide's existing tactics in text, and record a browser performance comparison and before-and-after evidence. | Select changes against observed friction and verify the integrated candidate. Refresh the Dev site's current references from a published build. |
| Bot and analyzer follow-up | Use measured baselines to choose a focused improvement in tactical behavior, responsiveness, or explanation. | Compare the candidate with the current presets before adopting a change. |
| One lesson in Guide | Use one existing replay to test whether a new player can understand a threat, try a line, and retry. | The practice navigation contract must hold. Use the trial to decide whether to expand to the planned three to five scenarios. |

The correctness checkpoint is part of the 0.7.0 candidate. Assign later point
releases when the findings and a coherent deliverable are clear.

## 0.7.0 candidate

### Replay launch and the active game

On v0.6.5, branch at move 26 of Guide's *Try another line*, play to move 28,
open Settings, and use browser Back. The review confirmed that this returns
the game to move 26. Replay passes a seed in browser history. Local Match
remembers consuming it in a route-local ref, so a remount can pass the same
seed to session creation and replace the active game.
[Local Match route](../../gomoku-web/src/routes/LocalMatchRoute.tsx#L232) ·
[Session creation](../../gomoku-web/src/game/local_match_session.ts#L94)

Treat **Play From Here** as a single explicit launch. Prefer calling the
existing session entry point from the confirmed branch action and navigating
after the session starts. Browser Back and Forward then return to the current
in-memory game. A new, explicit launch from the same replay and ply can start
another game, with the existing replacement confirmation when applicable.
A full page reload follows ordinary cold-start behavior. On the published
build, refreshing the same continuation at move 28 in fresh Chromium loaded a
new board at move 0. Durable unfinished-match recovery is separate work.

| Scenario | Expected behavior |
| --- | --- |
| Branch at 26, make a move, receive the bot reply, then Settings → Back → Forward → Back | Moves and side to play survive. An in-flight turn can finish once. |
| Reload the published match route directly | The route starts a new board and keeps completed history available. |
| Start a fresh match after branching, then revisit the old history entry | The newer match stays active. |
| Attempt a second branch and cancel replacement | Current moves, settings and replay attribution survive. Confirming an explicit second branch from the same ply starts a new round. |
| Reset the local profile or change its identity | The existing session reset still applies, including when revisiting prior history entries. |
| Finish a continuation | The regular My games row retains source attribution and the inherited opening. Existing links and replay/save/publish behavior continue to work. |

Extend the Guide browser journey to play before navigating away. Use session
tests for lifecycle cases below the route. Keep the established behavior of
applying settings changed mid-game to the next game.
[Guide browser test](../../gomoku-web/playtests/guide_examples.spec.ts#L82) ·
[Session tests](../../gomoku-web/src/game/local_match_session.test.ts) ·
[Settings contract](../reference/app/app_design.md#settings-during-a-match)

### Finished round and Undo

On v0.6.5, branch at move 27 of *Review a finished game*, win at F11, Undo, then
win at F11 again. The review confirmed that My games shows two matches with
distinct replay IDs. The profile store records every terminal transition as a
new game. [Profile store](../../gomoku-web/src/profile/local_profile_store.ts#L300)

For **0.7.0**, a win or draw finalizes its round and Undo is available during
active play only. After finishing, players can inspect the ending and use
Play From Here to start a continuation, which counts as an ordinary game when
finished. The match store and controls enforce the same rule.
[Match end state](../reference/app/app_design.md#match-end-state)

Private history merges by saved record ID. A public listing has its own
fingerprint and payload, and a copied URL carries its own replay. Keep issued
copies and previous history intact and count each intentionally started round
separately, even if its moves match a prior game.
[Cloud history merge](../../gomoku-web/src/cloud/cloud_profile.ts#L390) ·
[Publication lookup](../../gomoku-web/src/routes/PublishReplayPanel.tsx#L213) ·
[Portable replay encoding](../../gomoku-web/src/replay/portable_replay_codec.ts#L230)

Test the F11 reproduction, a draw, and a genuine second round. A finished
round should save one record and one result. Analyze opens the saved result
and preserves it across navigation. Undo still works during active play and
respects the inherited opening. A separately started round counts as another
game even if it ends with the same moves. Verify that local history and
signed-in emulator sync converge on one record per round after reload.
Previously published listings and copied links retain their original content.

### Existing acceptance and release checks

| Check | Evidence to capture |
| --- | --- |
| Local and CI | Focused session, history and browser checks for the changed contracts. Use the existing three-engine and Auth/Firestore emulator lanes for the candidate. |
| Real publication | With a disposable account and explicitly authorized production writes, sign in with Google, finish a game, enable a public profile, publish the selected game from My games, and opt in to statistics. In a fresh signed-out context, open the profile and replay and verify the opening and snapshot. Unpublish, turn off statistics, clean up the test profile and verify guest readback. A copied portable link should still open after unpublishing. Record the source, result and cleanup state. |
| Device and human checks | Reuse the Chrome touch/compact layout lane. Use a physical phone when available to check actual touch and the software keyboard. Check keyboard focus and real screen-reader output where the UI changes. Track two-device continuity separately using existing reset and sync scenarios. Record the scope of each check and carry real-world deferrals explicitly. |
| Release | Use the existing runbook for production dependency audits, CI on the exact commit, tag-triggered workflows and live checks. Retain the previous compatible recovery point. This plan proposes no schema, rules or index changes. |

The real-account round trip remains separate, manually authorized work. If
sharing or continuity changes, cover the change in candidate emulators and
repeat the affected live steps after an authorized deployment. If a live or
human check remains open, record the specific limit for the release decision.
[Testing guide](../reference/ops/testing.md) ·
[Release runbook](../reference/ops/release.md)

For the F11 ending, the v0.6.5 and initial 0.7.0 builds widened from 390 to
458 CSS px after the win. The disabled Place button pushed Analyze into the
narrow Undo column. The updated candidate removes Place when the round ends,
shows the result above Analyze, and lets the compact match layout wrap at
enlarged text sizes. The regression test wins the round by touch, checks the
visible result and horizontal bounds before opening the saved replay, and
covers Chromium, Firefox, and WebKit at 390 × 844 with normal text and 320 × 900
with 200% CSS text.
[Finish-screen regression](../../gomoku-web/playtests/win-screen.spec.ts)

### November 20 candidate snapshot (superseded by the current status above)

Local candidate checks passed on November 20. The full browser lane passed
312 cases, with six Chromium-only WebGL checks skipped in Firefox and WebKit.
All 72 Auth/Firestore emulator cases and 45 repeated race checks passed before
the compact result text was made visible. The updated build passed all nine
follow-up checks covering continuation sync after reload, offline recovery, and
mobile layout. The live Google account, physical-device, human accessibility,
and two-device checks remain open.

## 0.7.0 refinement work

### Observed changes

The baseline used the 607 archived v0.6.5 Pages files, verified against the
release manifest. The local captures used the same Chromium build, fixtures,
and viewport settings for the baseline and candidate.

On v0.6.5, branching from move 27 of a finished 28-move replay left no action
in Match to revisit that position. Browser Back reopened the replay at move 28
on both 1440 × 1000 and 390 × 844 Chromium viewports. The candidate's **Source
replay** action opens move 27 and focuses the Replay Analysis heading. Browser
Back from there returns to the active continuation. The match session keeps the
source and result links with the current round and clears both on the next round.
[Round and return regression](../../gomoku-web/playtests/win-screen.spec.ts)

The active turn clock previously updated state on the whole Match route at each
100 ms tick. The clock now owns its own updates. The route regression advances
the clock by 500 ms and checks that the displayed time changes without a board
render. [Match route test](../../gomoku-web/src/routes/LocalMatchRoute.test.tsx)

Inspecting a game from My games previously left keyboard focus on the document
body, where Escape could not reach Replay's return handler. The candidate
focuses the Replay Analysis heading and uses a return arrow for the My games
action. At 1440 × 900 and 320 × 900 with 200% CSS text, Escape returned focus to
the same row in a 16-match history and preserved desktop list scroll and
compact page scroll. On the enlarged replay, the player cards stack and the
rule and move count wrap without overlapping.
[Keyboard journey](../../gomoku-web/playtests/reconciliation.spec.ts)

The Guide's forced sequence now has three visible captions describing the
threat, the reply, and what remains open. The existing boards keep their visual
role alongside the captions. Human screen-reader verification is still open.
[Guide](../../gomoku-web/src/routes/GuideRoute.tsx)

### November 20 integrated snapshot (superseded by the current status above)

The integrated build passed typechecking, 588 web unit tests, 44 tooling tests,
and 312 browser tests, with six Chromium-only WebGL cases skipped in Firefox
and WebKit. All 72 Auth/Firestore emulator journeys and 45 repeated race checks
passed. The release commit and its GitHub CI remain ahead of publication.

### Code, design and Dev site

Use the same guest example to walk Home, Match, Replay, and Guide. Sample a
saved replay and signed-in My games as related return paths. Compare desktop
and compact/landscape layouts at matched scale, including enlarged text.
Log friction with a screen, an action, the observed result, and the desired
result. Choose the findings that repeatedly affect the journey.

- Review code ownership of the match session, worker lifecycle, and
  replay route state. Refactor only where a concrete finding benefits from it.
  Protect navigation, cancellation, source attribution and persistence at
  their observable entry points.
- Retain the board's visual hierarchy and existing pixel controls in the design.
  Clarify how the player reaches analysis, understands a turning point, and
  returns to play. The candidate adds plain-language descriptions of the side,
  threat, response and consequence to the Guide's forced-sequence frames.
  Verify those descriptions with a real screen reader when available.
- Use the matched v0.6.5 and candidate review to verify replay return. Qualify
  the published capture set before promoting a new screen reference. Keep the
  dated 0.6.2 screen reference and curated archives at their existing URLs.
  [Developer area contract](../reference/app/developer_area.md)

### Bot and analysis baseline

On November 20, the v0.6.5 tag and the integrated candidate from `d233bc4`
plus the working patch `037dfa38` each passed the 168-row browser profile.
Both builds used the same frozen workloads and Wasm bytes rebuilt with the
current toolchain. The runs used Chromium 156.0.8078.4 on a Ryzen 9 7900X.

| Median on this host | v0.6.5 tag rebuild | 0.7.0 candidate |
| --- | ---: | ---: |
| Board ready, fresh context | 818 ms | 818 ms |
| First replay annotation | 837 ms | 837 ms |
| Uncached 50-ply replay completion | 6.15 s | 6.14 s |
| Uncached 51-ply replay completion | 6.65 s | 6.65 s |

Each run completed 108 fresh-worker bot requests, 36 reused-worker requests,
18 uncached and 18 cached analyses, and 30 navigation cycles. Both passed
replacement and route cancellation checks. These runs support retaining the
current presets and analyzer settings. Physical-device timings remain open.

Record the source, build, device/browser, rules, position corpus and preset.
Use the existing native tactical and lethal fixtures for correctness and
the browser profiling lane for cold/warm response and uncached analysis.
Keep completion, timeout and cancellation outcomes visible. Compare candidates
on identical workloads and retain the current preset if the tradeoff is
unclear. A strength claim needs the existing tournament method. A latency
claim needs repeated browser measurements on the named device or host.
Qualify refreshed Bot lab reports only when promoted behavior or report data
changes. [Performance tuning](performance_tuning.md) ·
[Tournament runbook](../reference/ops/tournament.md)

### One testable lesson

Start with the existing full, legal 36-move Guide replay, after move 26 with
Black to play. Give the player a short, readable prompt comparing a defensive
reply with a counter-threat. The prompt remains useful while the analyzer
works. Preserve the distinction between an authored explanation, a possible
escape in bounded analysis, and what happens against the selected bot.
[Guide example source](../../gomoku-web/src/replay/replay_examples.ts)

Test whether a guest unfamiliar with the example can explain the threat,
play a response and return to the same position without help. Include a
mobile/keyboard path and usable text for tactics expressed in images. Opening
and retrying the example should leave history alone until a round is actually
completed. Record what the player got stuck on before planning more lessons.

Puzzle generation, grading, personalized challenges, cloud progress and
multiplayer remain exploration or deferred work. The pilot fits the existing
client and static content model. Use its outcome to decide whether a larger
lesson set is worth a release.
