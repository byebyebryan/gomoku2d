# Replay examples

The Guide's Example replays section bundles two small, complete games. Home's
Examples link opens `/guide/#examples`; the Guide section buttons also reach
the tactical overview and Replays & sharing help. Direct section entry and
repeated section activation restore heading focus and scroll into view.

Examples are ordinary portable v1 links opened by the existing shared replay
viewer. There is no example backend, public account/catalog seed, new replay
format or example-specific persistence. The runtime manifest is
[`replay_examples.ts`](../../../gomoku-web/src/replay/replay_examples.ts).

| Stable content ID | Display title | Full game / initial position | Source entry |
|---|---|---|---|
| `finished-game` | Review a finished game | Freestyle, 28 plies, White wins; opens at the finished board | `match_0095__search-d1__vs__search-d3_pattern-eval` |
| `another-line` | Try another line | Freestyle, 36 plies, White wins; opens at ply 26, Black to move | `match_1129__search-d3_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval_corridor-proof-c16-d8-w4` |

Both source games are in the committed
[`analysis-report.json`](../../../reports/lab/analysis-report.json), whose
embedded provenance records the June 1, 2026 source tournament and analysis.
The source tournament commit is `61999cc021c5`; the analysis commit is
`a8a407f37fa5`. These are historical bot-generated games, not user submissions.
Easy/Normal and Normal/Hard portable bot presets preserve the published
participant specifications. There are no user UIDs or invented human players.
Only these two move sequences and small content labels ship in the manifest;
the Guide does not import the full report.

The corpus unit scenarios run every move through the current Wasm legality
validator, compare the result/moves/bot specifications with the source report,
round-trip the portable URL and verify the selected continuation is eligible.
Changing the displayed ply does not change canonical identity. All moves after
a terminal result remain invalid. Cropped or synthetic Guide diagrams are not
eligible substitutes for these complete games.

Opening an example is ephemeral. Save explicitly keeps a private study copy;
Share exports a standalone portable link; an imported example cannot be
published as a game the viewer played. Play From Here uses the selected bot
and the existing active-game replacement guard. A completed continuation is an
ordinary My game with origin metadata. No separate practice badge or category
is introduced. Recomputed bounded analysis can differ from the historical
report; an example is not a graded puzzle or a promise of a winning move.

No private-history controller or Firestore request is introduced by opening
the standalone viewer. Guide retains the app's existing ordinary-route session
behavior; the static content itself adds no cloud operation. Save/publish and
private account sync retain their explicit existing behavior and limits.

This corpus can support later learning work, but 0.6.2 adds no tactical prompts,
answer grading, progress tracking, thumbnails or authored lesson editor.
