# Project Review Before the Next Release Cycle

Reviewed 2026-10-15 against `689e310bbe4a56c46337f6b1abe30b2bc73afe16`.
Owner: Bryan. Archived initial assessment: findings below describe the reviewed
baseline, before the subsequent repairs. The reproduced defects were addressed;
see the [completed reconciliation register](v0_5_reconciliation.md) and
[closeout](README.md) for final dispositions and acceptance deferrals.
Source links are pinned to the inspected revision so later refactors do not
change the evidence cited here.

The project has a coherent foundation: immediate local play, a shared Rust
rules and tactical model, useful replay analysis, and an inspectable public
Lab. The earlier reconciliation work produced durable results. Before another
feature cycle, address four reproduced failure paths and an accessibility gap.
Then make a fresh decision about the next player benefit. The existing online
roadmap is a design proposal, not evidence that online play is the best next
investment.

## Findings to Address

P1 means address before the next release; P2 means a material correctness,
recovery, or usability issue to include in the stabilization decision.

| Priority | Finding | Evidence |
|---|---|---|
| P1 | A stale cloud flush can patch a different account's current profile | Deterministic reproduction using the real promotion store and its reset/request guards |
| P1 | Local storage write failures escape core local-profile actions | Injected quota failure throws from `ensureLocalProfile` after changing memory |
| P2 | Replay presentation turns possible escapes into definitive claims | Pure UI-mapper reproduction; public Lab example also says “can escape” for a possible outcome |
| P2 | A failed opening bot move has no visible recovery path | Rejected-runner reproduction leaves the bot to move, with placement and undo unavailable |
| P2 | The board cannot be played through the keyboard or accessibility tree | Browser tree inspection and pointer-only input code; selected settings also lack semantic state |

### Stale cloud operations cross account boundaries

[`cloud_sync.ts`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/cloud/cloud_sync.ts#L83) captures a
profile's merged history and settings, awaits promotion, then checks the
*current global* promotion status and applies its captured patch to the
*current global* profile. The patch has no expected UID or operation generation.

Reproduction: start account A's flush and delay its backend response; reset the
stores; complete account B's flush; resolve A. The promotion store correctly
ignores A's stale result, but its status now reflects B's completed request.
A's caller proceeds and changes B's settings from freestyle back to Renju.
Only the network backend was stubbed in the stronger reproduction.

This establishes an in-memory account-state overwrite. History is carried in
the same unguarded patch, so mixing history is also a risk. It does not
establish a Firestore authorization bypass or an observed production data leak.

Return an operation-specific promotion result and verify the current account
and reset/request generation after awaits, including in the caller that loads
history from the returned profile. Acceptance should cover sign-out, A-to-B
switches, profile reset, and overlapping flushes finishing out of order.

### Local persistence can interrupt local play

[`local_profile_store.ts`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/profile/local_profile_store.ts#L218)
delegates storage reads/writes without a fallback; its `localStorage` lookup
also occurs before the JSON-storage factory. The quota probe made
`ensureLocalProfile()` throw after updating memory.
[`CloudSessionController`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/cloud/CloudSessionController.tsx#L64)
calls that action on app mount. Settings changes and finished-match recording
use the same persistence path; a failed finished-match save can prevent the
caller from receiving the replay ID. There is no app error boundary.

Provide a usable in-memory fallback and a visible indication when history or
settings cannot be saved. Verify denied storage access, quota errors, and
finished-match recording failures. The reproduction injected a storage write
failure; a real browser storage-denial session was not exercised.

### Replay claims discard proof confidence

[`replay_analysis_overlays.ts`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/replay/replay_analysis_overlays.ts#L87)
maps `confirmed_escape` and `possible_escape` to the same marker. Both also feed
the same escape timeline, which can say “Last chance to avoid loss” and
“Black's last escape” for a position with only a possible escape. The UI-mapper
probe reproduced exactly that result. Failure summaries similarly omit the
confidence carried by the analyzer protocol.

The public Lab's detailed legend distinguishes E from P, but
[`AnalysisProofFrames.tsx`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/routes/lab-report/AnalysisProofFrames.tsx#L436)
labels `escape_found` as “can escape.” Published game #65 visibly pairs that
decision with a `possible escape` outcome. The Guide also describes the last
escape without the analyzer's bounded-model qualification.

The Rust model already distinguishes possible, confirmed, and unresolved
outcomes. Preserve that distinction through status text, summaries, and marker
presentation. “Possible escape” is sufficient concise wording. Acceptance must
include possible-only, confirmed, mixed, and unresolved results. This is a
presentation correction; it does not require a new solver or regenerated
tournament results.

### Bot failure leaves a match unable to advance

[`local_match_store.ts`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/game/local_match_store.ts#L484)
clears `pendingBotMove` after a rejected, null, or illegal bot result without
retaining a user-visible failure or retry action. The route then displays the
bot's normal “to move” status while allowing input only for a human turn.

The injected runner rejection reproduced an empty board with Black assigned to
the bot, status `playing`, and `pendingBotMove: false`. Human placement and undo
both returned false. Starting another game is available, but the current game
offers no explanation or way to retry its turn.

Keep the existing bounded worker-crash retry. Add an explicit failed-turn state
and a recovery action that preserves the position. Cover repeated worker
failure, invalid/null output, recovery, and cancellation during recovery.

### Accessibility stops at the app shell

The local-match accessibility tree exposes navigation, player labels, clocks,
and Undo, but no operable board. [`Board.tsx`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/components/Board/Board.tsx#L143)
mounts a canvas host without keyboard or cell semantics, and board input is
pointer-based. A keyboard audit of shell controls does not establish that a
keyboard user can play.

[`SettingsRoute.tsx`](https://github.com/byebyebryan/gomoku2d/blob/689e310bbe4a56c46337f6b1abe30b2bc73afe16/gomoku-web/src/routes/SettingsRoute.tsx#L64)
uses active CSS classes for segmented selections without `aria-pressed` or
equivalent checked state. Add keyboard placement, an accessible description of
position and legal interaction, and semantic selection state. Acceptance should
include completing a game without a pointer and understanding the selected
settings through assistive technology.

## What the Previous Reboots Settled

| Phase | Lasting result | Implication for the next cycle |
|---|---|---|
| v0.1 | Rust, Wasm, and browser stack proof | The technology choice is established |
| v0.2 | Local product and React/Phaser ownership split | Keep application state outside the board renderer |
| v0.3 | Local identity and optional Firebase continuity | Private uploaded history remains client-authored, separate from trusted competition |
| v0.4 | Bot lab, tactical models, and replay analysis | Explanations are the strongest product result; several strength experiments did not earn promotion |
| v0.5.0–v0.5.3 | Reconciliation, public reports, Rules/Guide, public alpha | Public presentation and release machinery became real product surfaces |
| v0.5.4 | Follow-up reconciliation, ownership, tests, and resilience | The premature v0.5.3 closeout received a substantive follow-through |
| October maintenance | Refreshed dependencies, current CI, manual maintenance policy | The next cycle starts from a validated development baseline |

The archived plans explain decisions; they should not all become active work
again. The [current roadmap](../../reference/product/roadmap.md) correctly calls
for a fresh design checkpoint before online implementation. Parked process-story
and devlog work remains optional.

## Architecture and Tactical Model

Retain the main boundaries: React owns routes and app controls; Phaser owns
board presentation and pointer interaction; Rust owns legality, tactics, bots,
and replay analysis; the native lab evaluates those same semantics. Reports
carry provenance instead of silently presenting old measurements as new ones.

The most consequential seams are asynchronous orchestration and persistence,
as the first two findings demonstrate. Strengthen those seams with identity,
generation, and explicit failure contracts. Large files alone do not justify
another restructuring program. Scan/rolling parity implementations and fixed
corpora remain useful correctness oracles; corridor proof is an active model,
distinct from retired experimental search work.

Renju deserves an explicit authority decision before trusted online play. The
oracle uses a recursion guard of four and deliberately fails open when it cannot
resolve recursive legality. This is documented in
[Renju Rules](../../reference/lab/renju_rules.md#implementation). The 29-case
golden corpus passed, but that is not proof of completeness. No incorrect-rule
counterexample was established in this review. Decide the required semantics
and add adversarial coverage or guard-hit visibility before claiming trusted
server authority.

## Product and Performance

The clear product loop is play, finish, inspect the turning point, and try a
different line. Rules and Guide make the tactical vocabulary understandable;
the public Lab makes the underlying work inspectable. Desktop play and the
mobile Settings/Profile layouts were coherent in the fresh walkthrough.
Navigation preserved the active game. A page reload does not persist the active
game; only completed records are saved. Online reconnect will need an explicit
state model.

The home page remains intentionally minimal, and most of the differentiating
experience appears after a finished game or through the Guide/Lab. Before
expanding the feature set, observe a few new players completing the existing
loop: can they find analysis, understand its uncertainty, and use a candidate
move to learn or branch? Engineering evidence is strong; this review did not
establish player learning, retention, or demand for online play.

Browser bots use fixed depth/width without the native tournament's CPU budget.
The curated tournament contains 1,792 games; the analysis report covers 192
selected games. Their June provenance is visible and intentionally preserved.
The tournament's Ryzen 7900X and pooled CPU limits do not establish mobile Wasm
latency. Opening coverage is a bounded centered suite, not an external strength
benchmark. Measure representative browser devices, slow positions, memory,
worker startup, and cancellation before making stronger performance claims.
No slow-device failure was reproduced here.

## Operations and Evidence

The development baseline and public deployment differ. Main is the October
maintenance commit above, with [successful CI](https://github.com/byebyebryan/gomoku2d/actions/runs/38808523242).
The public site is still July's v0.5.4 at `8e47284`; this review did not deploy
the maintenance work.

Read-only infrastructure checks confirmed Firestore in `us-central1`, Google
authentication enabled, and deployed Firestore rules byte-for-byte equal to
the repository source. Rules SHA-256:
`cf125b43445b87af94b3bec8b04237706ef7e1095c14badee43e2c365a6b202a`.
Cloud Run is not enabled and there is no implemented authoritative game server.
Database delete protection is disabled; revisit recovery and deletion policy
when increasing reliance on durable cloud state. An actual Google sign-in,
cross-device sync, and cloud reset round trip remain manual acceptance checks.

Today's preceding maintenance pass passed Rust formatting/Clippy, 340 Rust
tests, tactical hard gates 12/12, lethal cases 9/9, Renju cases 29/29, Wasm build,
web typecheck, 302 web tests, 23 Firestore-rule tests, production build, and 23
browser tests. Tactical diagnostic cases were 21/36, with 15 non-gating misses;
do not present the hard-gate result as comprehensive tactical success.

This review added four isolated defect probes and a stronger account-race probe
using the real promotion guards. All reproduced the described behavior. The
temporary tests were removed. Source reading, browser observations, automated
checks, live rules readback, and unperformed manual acceptance are separate
evidence levels.

Production dependency audits are clear. Five dev-only affected dependency
entries remain in the Firebase CLI toolchain, documented with the overrides and
deferred major upgrades in [Dependency Maintenance](../../working/dependency_maintenance.md).
No pull requests are open; updates are now owner-maintained. No release tag or
deployment is part of this review.

## Proposed Sequence

1. Repair the cloud-operation identity guard and local persistence fallback.
2. Preserve analyzer confidence in all player-facing explanations and add bot
   failure recovery. Establish the keyboard-play acceptance requirement.
3. Validate the repaired full local loop, then perform the real cloud continuity
   checks before publishing a stabilization release.
4. Choose one next-cycle outcome. Replay-based practice follows the strongest
   existing differentiator. Direct challenge can justify online work if playing
   another person is the intended next benefit. Do not bundle both by default.
5. If online wins that decision, settle server authority, reconnect/timeouts,
   concurrent sessions, trust lanes, explicit publishing, operating cost, and
   ownership before implementation, as the current roadmap already requires.

The repair pass is complete when the demonstrated failure cases have focused
regression coverage and usable UI behavior. The next feature cycle is ready
when its player benefit and acceptance are explicit.
