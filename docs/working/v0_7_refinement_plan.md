# Gomoku2D 0.7 refinement plan

Reviewed November 20, 2026. The baseline is the published
[v0.6.5 release](../archive/v0_6_5_release.md) at `25d67c1` and its
documentation closeout at `e6c8999`. The
[product roadmap](../reference/product/roadmap.md) owns adopted sequencing.

Use 0.7 to make the existing game more reliable, easier to learn, and easier
to work on. **Play → Replay → Try another line** is the journey that connects
the app, the Rust bot and analyzer, the UI, and the Dev site's design evidence.
Guest play, replay and analysis stay in the browser. Keep the same Replay
Analysis surface, board-first layout, opt-in publication, and static hosting.

## Sequence

| Milestone | Work and outcome | When to proceed |
| --- | --- | --- |
| **0.7.0** | Preserve replay continuations across navigation, finalize completed rounds, and verify existing journeys. | Confirm guest and emulator behavior before release review. |
| Design, code and presentation | Improve the transitions between Match, Replay and Guide on desktop and mobile. Refine the underlying code where it simplifies a demonstrated problem. Refresh the Dev site's current references from a published build. | Select a small set of findings from the current game before committing to a release. |
| Bot and analyzer | Establish current browser and native baselines. Choose a focused improvement in tactical behavior, responsiveness, or explanation. | Baseline work can run independently. Compare the candidate with the current presets before adopting a change. |
| One lesson in Guide | Use one existing replay to test whether a new player can understand a threat, try a line, and retry. | The practice navigation contract must hold. Use the trial to decide whether to expand to the planned three to five scenarios. |

Only the first milestone has a version. Assign later point releases
when the findings and a coherent deliverable are clear. If an early correctness
fix is ready, it can ship independently of the design and bot baseline work.

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

The live account check can run against the existing release before code work.
If sharing or continuity changes, cover the change in candidate emulators and
repeat the affected live steps after an authorized deployment. If a live or
human check remains open, record the specific limit for the release decision.
[Testing guide](../reference/ops/testing.md) ·
[Release runbook](../reference/ops/release.md)

The 390 × 844 Chrome touch check for the F11 ending reached the result and
Analyze. On both v0.6.5 and the 0.7.0 Undo candidate, the page was 390 CSS px
wide before the win and 458 CSS px wide afterward. Review the finished-match
actions at this width during candidate review.

## Foundation work after the first milestone

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
  returns to play. Add plain-language descriptions of the side, threat,
  response and consequence to the Guide's forced-sequence frames. Verify
  those descriptions with a real screen reader when available.
- Refresh the Dev site's Current screens with a qualified published set
  and preserve the 0.6.2 reference at its existing URL. Add a before-and-after
  case using matching conditions and the actual findings, changes and limits.
  Use the existing Dev site, review catalog and history tools.
  [Developer area contract](../reference/app/developer_area.md)

### Bot and analysis baseline

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
