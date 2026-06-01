# Process Story Devlog Kit

Purpose: publishable external "making of" package for itch.io, GitHub, or a
blog post. This is curated public copy, not a raw transcript dump.

Status: first draft kit. It should be reviewed against the current app, release
history, and screenshots before publishing.

## Positioning

Primary audience:

- Indie/game-dev readers who care how the project was built.
- Technical players who are curious why Gomoku2D has a visible lab and replay
  analyzer.

Primary hook:

> Gomoku2D is a playable browser Gomoku/Renju game whose replay analyzer can
> show where a finished match turned.

Secondary hook:

> The making-of story is one developer using agents like a small production team,
> while the project stayed coherent because the human kept making the taste,
> scope, and correctness calls.

Working title options:

- `The Game After The Game: How Gomoku2D Became More Than A Board`
- `Building Gomoku2D With Agents In The Loop`
- `One Developer, A Small Team Of Agents, And A Gomoku Game`

Recommended title:

> The Game After The Game: How Gomoku2D Became More Than A Board

Recommended deck:

> Gomoku2D started as a revival of an old five-in-a-row project. It became a
> browser game with a Rust core, bot lab, public reports, and replay analysis
> that turns finished matches into practice material. Behind that product is a
> multi-week experiment in building like a small indie team, with agents doing
> parallel work while one person kept steering.

## Claim Guards

- Do not say "AI built this." Say agents helped with implementation,
  exploration, review, and evidence mining.
- Do not publish raw chat logs. Paraphrase decisions and use short quotes only
  when exact wording matters.
- Do not claim the analyzer solves Gomoku. It is a bounded tactical analyzer.
- Do not claim bot strength is the main product promise. The distinctive
  promise is explainable play.
- Do not imply all history is cloud-backed or public. The app is guest-first
  and local-first, with optional private cloud continuity.
- Keep failures visible. The most interesting story beats include ideas that
  did not promote.

## Article Draft

### 1. The Small Game Was Not The Small Project

Gomoku2D started from a familiar idea: take an old five-in-a-row favorite and
make it work as a real browser game.

The surface is intentionally small. Open the page, start a match, place stones,
make trouble. No account is required. The board should feel close to the paper
game: direct, readable, and fast enough that the rules disappear behind the
next move.

The work underneath was less casual. The rules engine lives in Rust. The web
game reaches it through WebAssembly. React owns the app shell, Phaser owns the
animated board, and the Rust bot lab owns tournaments, replay analysis, and
report generation. That split was heavier than a quick clone needed, but it
kept the important question in one place: what is legal, what is a threat, and
why did this match turn?

This is where agents started to matter. A solo project usually runs out of
parallel attention. There is code to write, a UI to polish, reports to verify,
docs to sync, screenshots to review, and failed experiments to clean up. Agents
made it possible to cover more of that surface area. They did not remove the
need for judgment. They increased how often judgment could be applied.

Visual target: Home page plus one active match capture.

### 2. The Product Loop Came First

Before the lab mattered, the game had to feel like a product.

The early work moved the project from prototype to local-first browser app:
guest play, profiles, settings, replay history, mobile layout, rule selection,
and eventually optional cloud continuity. The important product choice was that
sign-in should not stand in front of the board. A player should be able to
start locally, then use cloud history only when continuity matters.

That choice also shaped the process. Agents could implement flows and run
checks, but the product boundary was human-selected: guest-first by default,
cloud only as a durable layer, and replay as a normal part of playing rather
than a debug artifact.

Visual target: Settings/profile/history capture, ideally paired with a mobile
match capture.

### 3. The Lab Was Supposed To Make The Bot Stronger

The bot lab started with a straightforward goal: stop guessing about bot
changes.

Search depth, width caps, pattern scoring, tactical ordering, pooled budgets,
and report metrics all went through tournament runs. If a variant looked
promising, it had to beat anchors. If it only sounded promising, it stayed out.
This is the kind of work agents are good at: run the experiment, collect the
evidence, review the diff, update the docs, repeat.

But the lab also exposed a limit. Raw bot strength is not the most interesting
thing Gomoku2D can offer. A browser practice bot should avoid obvious mistakes,
but the project becomes more distinctive when it can explain the game instead
of only playing it.

That changed the question from "can this bot be stronger?" to "can the game
show why this position became lost?"

Visual target: Lab ranking/search report with timing or ranking cards.

### 4. The Failed Shortcut Became The Analyzer

Corridor search was first explored as a live-search shortcut.

The idea made sense. Gomoku often enters narrow tactical corridors: one side
creates a threat, the other side has only a few meaningful replies, and the
position can be followed deeper than a normal broad search. In theory, these
corridors are portals through the search tree.

In practice, the live-search version did not promote cleanly under the browser
budget. It could spend too much time proving that a leaf was not useful. It
could follow a corridor to a neutral exit and then resume normal search after
paying the cost. More knobs did not fix the shape of the problem.

The useful part survived because replay analysis has a different shape. A
finished game already has an actual line. Instead of asking the bot to search
everything forward, the analyzer can walk backward from the ending and ask a
bounded question: did the losing side have a viable escape before the position
became lethal?

That pivot produced the vocabulary that now defines the analyzer: setup
corridor, lethal onset, last escape, missed response, forced loss, possible
escape. The failed optimization became the feature because it moved to the
layer where the model was actually true.

Visual target: Replay Analysis timeline showing last escape, setup corridor,
and lethal onset. Supporting visual: Lab Analysis proof frame.

### 5. Correctness Got Deeper Than Shape Matching

Renju looked like a small rule variant until it broke the shortcut model.

Black has forbidden moves: overline, double-four, and double-three. It is
tempting to treat those as visual shapes. Find two threes or two fours after
Black's move, mark the move forbidden, move on.

That was not enough. A forbidden double-three or double-four is about real
threats, not rough geometry. If one branch is boxed in, blocked by White, or
only continues through another forbidden move, the shape may be present while
the threat is dead.

This mattered because the analyzer and hints depend on legality. A wrong
forbidden marker can hide a valid defense or invent a loss. The fix was a
slower, more faithful Renju checker: extract reference examples, build a
fixture corpus, validate the results, then add fast paths and metrics so the
correct model remained practical.

That is a good example of the human/agent split. Agents could extract cases,
write fixtures, compare outputs, and patch implementations. The human call was
to stop treating Renju as a regex and insist on a proper model.

Visual target: Rules page real double-three examples, plus one before/after
analysis frame if available.

### 6. Reports Became A Product Surface

The reports were not designed as marketing.

They were diagnostics: tournament standings, pairwise results, search-stage
timings, proof boards, analysis failures, and report provenance. Over time they
became one of the clearest views into the project. They showed which ideas
worked, which did not, and how the bot/analyzer changed.

That made the v0.5 report rewrite feel less like cleanup and more like
productization. Rust emits compact data. The web app renders the report. The
Lab page shares the same visual language as the rest of the game instead of
shipping giant generated HTML artifacts.

This is also part of the process story. The lab is not hidden. The reports are
a receipt for the decisions.

Visual target: Current `/lab/` Ranking, Search, and Analysis tabs. Optional
before/after: old generated report artifact versus current web-rendered Lab.

### 7. What Agents Actually Changed

The useful story is not autonomy. It is leverage.

Agents helped Gomoku2D behave more like a small production team. One pass could
inspect frontend surfaces. Another could review docs. Another could run a lab
experiment. Another could dig through a failing report. The main thread could
keep integrating, challenging assumptions, and deciding what mattered.

The process worked best when the task was concrete:

- make the report easier to read;
- find why this Renju move is misclassified;
- compare scan and rolling-frontier performance;
- review the release line and cut dead paths;
- sync docs after the implementation changed.

It worked worst when the task was vague or when temporary tests and scaffolding
were left behind. That became part of the cleanup work. Process debt is still
debt: stale docs, noisy artifacts, too-specific tests, and old experiment names
make a project harder to trust.

The human role stayed central because the hardest calls were not typing calls.
When corridor portals failed, the answer was to stop tuning. When reports
became interesting, the answer was to productize them. When Renju examples
contradicted the shortcut checker, the answer was to slow down and build a
corpus. When public pages became too wordy, the answer was to cut.

Agents made the project wider. Judgment kept it coherent.

Visual target: Release timeline from v0.1 to v0.5, plus a compact "loop" card:
direction, implementation, review, verification, commit.

### 8. The Game After The Game

The current result is still modest: a small browser Gomoku/Renju game.

But it now has a second loop. You can finish a match, open replay analysis, and
ask where the game turned. Was there a direct response? A last escape? A lethal
combo? A Renju block that was not legal? The analyzer does not claim to solve
the whole game. It explains a bounded slice of a real finished match.

That is the project identity that came out of the process. Gomoku2D is not
trying to be the strongest Gomoku engine. It is trying to be a playable game
with enough understanding of itself to help you learn from the board.

Visual target: End-game `Analyze` entry point and replay frame with "last
escape" or "lethal onset" status.

## Short Version

Gomoku2D is a small browser Gomoku/Renju game with a Rust rules core, a bot
lab, and replay analysis that can explain where a match turned. It is also a
production experiment: one developer using agents like a small team, with the
human still responsible for taste, scope, and correctness. The most important
feature came from that process: a failed bot-search shortcut became the replay
analyzer's tactical vocabulary.

## Pull Quotes

- "Agents made the project wider. Judgment kept it coherent."
- "The failed optimization became the feature because it moved to the layer
  where the model was actually true."
- "The analyzer does not claim to solve the whole game. It explains a bounded
  slice of a real finished match."
- "The reports are a receipt for the decisions."

## Publishing Notes

- Use one hero image, three to five inline visuals, and one closing screenshot.
- If publishing on itch.io, keep the article draft shorter and link to the
  GitHub README, Lab report, Rules, Guide, and Visuals pages.
- If publishing on GitHub or a blog, keep the full technical arc and use the
  visual storyboard as the capture checklist.
- Re-check screenshots after the release build is served locally; do not use
  stale `0.4.x` captures for a `0.5.x` story.
