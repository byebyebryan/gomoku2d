# Process Story Drafts

Purpose: rough public-copy drafts derived from the private process-story
extraction. These are not final release notes. They are working drafts for
README, itch.io, devlog, or a project writeup.

Editorial rule: for player-facing copy, lead with the game and analyzer. For
the external making-of package, the process can lead, but only if shipped
product surfaces remain the evidence and the story avoids "AI built this"
framing.

## Primary Draft: The Game After The Game

Subtitle: A paper-game favorite rebuilt as a browser game with a Rust rules
core, local-first play, and replay analysis that shows where a match turned.

Gomoku2D started with a small, stubborn idea: take an old five-in-a-row
favorite seriously.

Not seriously as in grim or overbuilt. The game still needed to feel immediate:
open the page, start a match, place stones, make trouble. Gomoku is simple
enough to teach in one sentence, but sharp enough that every shortcut
eventually leaks through the board. Rules matter. Replays matter. Bot behavior
matters. Renju legality matters. If the game was going to become more than a
nostalgic sketch, the quiet parts underneath had to be built properly.

That became the first shape of the project: a local-first browser game with a
Rust/WebAssembly rules core, a React app shell, a Phaser board, and a Rust bot
lab behind it. The result is deliberately modest on the surface. You can play
without an account, keep local history, choose Freestyle or Renju, and use the
same board-first flow on desktop or mobile. Under that small surface is a lab
that can run tournaments, preserve replay data, benchmark bot changes, and move
proven logic into the web build instead of burying game decisions in UI code.

The more interesting turn came after matches ended.

Most board games treat a finished local match as a record: winner, loser, move
list, maybe a replay scrubber. Gomoku2D started pushing that further. A
finished game should be practice material. If the match collapsed, the product
question is not only "who won?" but "when did the game become unrecoverable?"
Was there a last escape? Did the losing side miss a direct response? Did a
legal-looking defense fail because the threat had already become lethal?

That is where the bot lab stopped being just a stronger-opponent workshop and
became part of the product identity. The same report machinery used to compare
bots became a way to explain games. Corridor search, originally explored as a
tactical shortcut for live play, proved more valuable as replay-analysis
vocabulary: setup corridor, lethal onset, last escape, forced loss, missed
response. It did not need to solve Gomoku. It needed to explain a bounded,
concrete slice of a real finished match.

That boundary matters. The analyzer is not pretending to be an oracle. It walks
backward from a decisive ending, follows named tactical obligations, and marks
what it can prove. When it cannot prove a legal alternative still loses, that
is a possible escape, not a hidden certainty. This makes the feature more
honest and more useful: the game can show where the board turned without
claiming to know every future branch.

The public app now has two loops. The first is the old one: play a clean, fast
match of five-in-a-row. The second is the game after the game: open the replay,
scrub the timeline, inspect the setup corridor, and jump back into practice
from a position that mattered.

That is the product story Gomoku2D should carry. It is a simple game, built
with enough care that the replay becomes a lesson instead of a tombstone.

Suggested visuals:

- Current home screen with "An old favorite, built properly."
- Local match clip or short board animation.
- Replay Analysis timeline with last escape, setup corridor, and lethal onset.
- `/lab/?tab=analysis&match=match_0065__search-d1__vs__search-d3_pattern-eval`
  showing proof boards and failure labels.
- Compact release/commit timeline from Rust core to v0.5 public surfaces.

Claim guards:

- Do not claim the analyzer is a full Gomoku solver.
- Do not lead with "AI built this."
- Do not imply every replay is public or cloud-backed.
- Do not market bot strength as the main promise.

## Draft 1: An Old Favorite, Built Properly

Subtitle: A small browser Gomoku game with a Rust core, a local-first product
loop, and a lot more care than the board suggests.

Gomoku2D started from a simple premise: take an old favorite and build it
properly.

The game itself is intentionally familiar. It is still stones on a grid. Black
moves first. Five in a row wins. It should be easy to open, easy to understand,
and playable without an account. That part matters because Gomoku is not a game
that needs a long onboarding funnel. It should feel close to the paper version:
look at the board, place a stone, see what happened.

The project around that simple board became much more deliberate. The rules
engine is Rust. The browser runs through a WebAssembly bridge. React owns the
application shell, Phaser owns the animated board, and the bot/eval tools live
in the same Rust workspace as the game logic. That split sounds heavier than a
small game needs, but it solved the real problem: the web game, bot lab, replay
viewer, and reports all have to agree on what a legal move, a threat, and a win
actually mean.

The first product milestone was not "make the strongest bot." It was making the
game feel like a real local-first app. Guests can play immediately. Replays are
saved locally. Profiles, settings, rule choices, and match history are part of
the product instead of debug state. Cloud sign-in came later as continuity, not
as a gate in front of the board.

That framing still guides the project. Gomoku2D looks small on purpose. The
work underneath is there so the simple version can stay simple: quick play,
clean replay, readable rules, and a bot that is useful because the game can
explain itself afterward.

Suggested visuals:

- Home page with "An old favorite, built properly."
- A live board frame on mobile and desktop.
- Short replay/history clip showing local-first flow.
- Architecture sketch: React shell, Phaser board, Rust/Wasm core, Rust lab.

Claim guards:

- Do not frame this as "AI built a game."
- Do not overstate bot strength.
- Keep the product loop visible: play, save, replay, analyze.

## Draft 2: The Game After The Game

Subtitle: The most interesting move in a finished Gomoku game is often the one
that happened several turns before the win.

Most Gomoku games do not become educational at the final move. By then the
winner has already made five. The useful question is earlier: where did the
losing side run out of safe choices?

That became the product hook for Gomoku2D's replay analysis. A finished match
is not just stored as a move list. It can be walked backward from the ending
position. The analyzer looks for the point where the game entered a forced
sequence: the setup corridor, the lethal onset, and the last escape before the
loss became unavoidable under the model.

The important design choice is that the analyzer does not pretend to be a full
Gomoku solver. It works with a bounded tactical vocabulary: immediate threats,
imminent threats, counter-threats, lethal combos, forbidden Renju blocks, and
forced corridors. That is still enough to answer the question that matters most
after a practical game: did the losing side miss a response, miss an escape, or
get pushed through a sequence where every reasonable reply failed?

This changes what replay means. A normal replay answers "what happened?" The
analysis view tries to answer "why did it become lost?" The board highlights
threat positions, evidence stones, forbidden moves, last escape markers, and
the part of the timeline that belongs to the forced sequence. From there, the
player can branch from a frame and try a different move.

That is the strongest product idea in the current version of Gomoku2D: the game
does not end at the win screen. The ending becomes practice material.

Suggested visuals:

- End-game `Analyze` button.
- Replay Analysis timeline with last escape and onset markers.
- One report/game frame showing setup corridor and lethal onset.
- Branch-from-replay button.

Claim guards:

- Say "bounded analyzer" or "under the analyzer model" when needed.
- Avoid "solves the game."
- Keep the player-facing promise simple: learn where the game turned.

## Draft 3: The Failed Bot Trick That Became The Analyzer

Subtitle: Corridor search started as a shortcut for stronger bots. Its better
job was explaining where a finished game stopped being escapable.

The first version of the bot lab had a familiar goal: make the bot stronger.
Tune search depth. Cap the width. Improve move ordering. Add tactical checks.
Run tournaments. Repeat.

That work helped, but it also exposed a ceiling. A small browser game cannot
spend unlimited compute on every move, and raw strength is not the most
interesting promise for this project anyway. A competent practice bot should
avoid obvious mistakes. A more distinctive bot should help explain the game.

Corridor search came from that shift. Gomoku is full of positions that look
quiet until they are not. One move creates a threat, the opponent has only a
few plausible replies, and suddenly the rest of the game feels like a hallway.
If one side creates a four, the defender has to answer. If one side creates a
real three, the defender has only a small set of direct replies or
counter-threats. In theory, those narrow branches are portals through the
search space: follow the corridor cheaply, reach a win or an exit, and make the
normal search effectively deeper.

In practice, the live-search version did not promote cleanly. Corridor portals
could accept too many positions, hit neutral exits, and resume normal search
again and again. Leaf quiescence spent work proving that most leaves were not
useful corridors at all. Several variants were tried, measured, documented, and
removed or left off by default. The idea was sound strategically, but the
implementation shape was wrong for live move selection under a browser-scale
budget.

The replay analyzer kept the part that worked. Finished games already have an
actual line. Walking backward through that line, the analyzer can ask a more
focused question: while the losing side was inside this threat corridor, was
there a viable escape? If not, keep tracing back. If yes, mark the last chance.
If the line has already reached a lethal combo, stop treating the remaining
moves as meaningful defense and mark the onset.

That pivot is why corridor search became central instead of discarded. It did
not become magic bot strength. It became vocabulary: immediate threat,
imminent threat, counter-threat, setup corridor, last escape, lethal onset. The
failed feature did not get buried. It got measured, narrowed, renamed, and
moved to the layer where it was actually true.

Suggested visuals:

- Old/new diagram: broad search tree vs narrow corridor.
- Analysis report frame for a last escape.
- Timeline segment showing setup corridor to lethal onset.
- Lab report or commit trail showing portal experiments retired.
- Current `Corridor Search`, `Game Analysis`, and `Search Bot` docs.

Claim guards:

- Say live corridor search "did not promote under browser-scale budgets," not
  that corridor search failed in every form.
- Do not imply corridor search proves global survival.
- Treat possible escapes honestly: unproved alternatives are not forced losses.
- The win is explanation and vocabulary, not brute-force strength.

## Draft 4: Renju Rules Were Not A Regex

Subtitle: Double-three and double-four are not just shapes. They only matter
when the branches are real.

Renju looks like a small rule change until you try to implement it correctly.
Black has restrictions: no overline, no double-four, no double-three. White
does not. At first glance that sounds like shape matching. Look for two threes
or two fours after Black's move, reject the move, move on.

That was not enough.

The hard part is the word "real." A double-three is forbidden because it creates
multiple real ways for Black to force a win. If one branch is blocked by the
edge of the board, by White's stones, or by another forbidden continuation, it
may only look like a double-three. The shape is present, but the threat is dead.

That distinction mattered once replay analysis and lethal-threat detection
started depending on Renju legality. A shape shortcut could mark valid defenses
as forbidden, hide candidate moves, or misread why a game became lost. The fix
was not another patch around one example. The checker had to become a recursive
Renju legality checker: test the candidate, follow the branches, and decide
whether the threats are real under Renju's own restrictions.

The project pulled in external examples, extracted a Renju corpus, validated
against reference behavior, then integrated the slower but correct model back
into search, reports, hints, and the rules page. After that came the less
glamorous part: instrumentation and fast paths so correctness did not destroy
bot performance.

That is the kind of rabbit hole a small board game can hide. The UI only needs
to show a forbidden marker. The engine has to know whether the marker is true.

Suggested visuals:

- Rules page examples for blocked/neutral double-three.
- A board where a shape looks forbidden but is legal.
- A board where the forbidden block changes the outcome.
- Before/after report frame where a candidate is no longer hidden incorrectly.

Claim guards:

- Do not drown the reader in RIF/Renju terminology.
- Keep the rule insight central: real threat proof, not rough shape count.
- Note that this is about Black restrictions under Renju.

## Draft 5: Reports Became The Lab

Subtitle: The reports started as diagnostics. They became how the project made
decisions.

The bot lab did not start as a public feature. It started as a way to stop
guessing.

If a bot change seemed better, it had to run against anchors. If a search
variant looked promising, it had to show up in tournament results. If replay
analysis looked wrong, it needed a report frame that could be inspected. That
created a lot of internal artifacts: tournament tables, per-stage timing,
analysis boards, proof details, and generated HTML.

At first those reports were clearly developer tools. They were too dense, too
large, and too tied to Rust-generated HTML. But they also became one of the
project's most interesting surfaces. They showed the lab thinking in public:
which bots were tested, what the analyzer saw, where the proof stopped, how
long search stages took, and which ideas failed promotion.

That made the v0.5 report rewrite less like cleanup and more like
productization. Rust now exports structured data. The web app renders the Lab
viewer. Published artifacts are compact JSON, not giant blocks of generated
HTML and CSS. The report pages share the same visual language as the game,
rules, guide, and visuals pages.

The point is not that every player will read tournament tables. The point is
that Gomoku2D has a visible lab under the board. The reports are a receipt for
how the bot and analyzer got here, and a way to keep future changes honest.

Suggested visuals:

- Current `/lab/` report page.
- Search timing bar or ranking table.
- Analysis report board frame.
- GitHub diff/stat before and after the viewer/data rewrite.

Claim guards:

- Reports support the game; they are not the game.
- Avoid exposing every diagnostic as public meaning.
- Keep "lab" framed as transparency and learning, not leaderboard theater.

## Draft 6: One Developer, Agents As A Small Production Team

Subtitle: The useful story is not that AI wrote the project. It is that agents
made a one-person project behave more like a small team.

Gomoku2D was built by one developer making product decisions, taste calls, and
correctness calls. Agents changed the throughput, not the responsibility.

The working loop settled into a pattern. The human set direction: preserve
guest-first play, make the analyzer explain decisions, keep reports presentable,
do not overclaim solver strength, clean up dead paths instead of stacking more
knobs. Agents explored code, implemented slices, ran tests, regenerated
reports, reviewed diffs, and summarized risk. Subagents were useful when the
work split cleanly: one pass for frontend, one for docs, one for lab logic, one
for static surfaces.

That process worked best when the goal was concrete. "Make the report easier to
read." "Find why this Renju move is misclassified." "Compare scan and rolling
frontier." "Review the 0.4 line and cut dead paths." It worked worst when the
task was vague or when tests were created as temporary scaffolding and then
left behind. Part of the 0.5 cleanup was admitting that process debt matters
too: docs drift, generated artifacts bloat, tests become too specific, and old
experiments leave confusing API names.

The human role stayed central because the hard calls were not just code calls.
When corridor portals failed, the right answer was not "try more knobs" forever.
When reports became interesting, the right answer was to productize them. When
Renju examples contradicted the shortcut checker, the right answer was to slow
down and build a corpus. When public pages got too wordy, the right answer was
to cut them back.

That is the process story worth telling: agents made it possible to push a
small project wider than one person normally would, but the project only stayed
coherent because someone kept saying what mattered.

Suggested visuals:

- Release timeline from v0.1 to v0.5.
- Example review/fix/commit loop.
- Subagent review summary screenshot, if appropriate.
- Before/after docs or report cleanup.

Claim guards:

- Do not claim autonomy.
- Do not publish raw chat logs as content.
- Keep the story grounded in shipped surfaces and decisions.

## Draft Package Recommendation

For a first public push, use three layers:

1. README/itch headline: `An old favorite, built properly. Play instantly, then
   learn where the game turned.`
2. Main external devlog: use
   [`Process Story Devlog Kit`](process_story_devlog_kit.md) to lead with the
   game-after-the-game product hook, then explain the
   one-developer-plus-agents process as the making-of layer.
3. Technical followups: publish Draft 3 and Draft 4 as separate posts if the
   audience wants implementation depth.

The report and agent-process stories should support the product promise. They
can lead a making-of post, but should not replace the basic game pitch.
