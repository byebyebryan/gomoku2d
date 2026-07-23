# Process Story Narrative Context

Purpose: shape the private process-story material into a coherent narrative
before turning transcripts, commits, reports, and screenshots into public copy.

This is an archived note, not publish-ready copy. The goal is to preserve the
personal lens and the product arc, then use the extracted conversation arcs as
evidence.

## Working Thesis

Gomoku2D is not interesting because one person used agents to make a web game.
That story is too generic.

The stronger story is:

> One developer used agents to amplify personal taste, technical judgment, and
> domain curiosity. The agents widened what could be attempted, but the project
> became distinctive because the human kept deciding what mattered.

The process story should stay between two weak extremes:

- Not anti-AI: agents genuinely changed what was practical to build.
- Not AI-utopian: agents did not replace understanding, taste, or judgment.

The core phrase for this direction is:

> amplification with ownership

## The Narrative Arc

### 1. Start With A Personal Revival

The project began as a revival of an old, long-dead, half-finished Gomoku
project. That matters because it gives the work a personal reason to exist
before AI enters the story.

The first product identity was natural:

> an old favorite, built properly

That pointed toward retro feel, careful board interaction, mobile polish, and a
proper modern stack instead of a quick clone.

### 2. Build A Foundation That Seems Bigger Than Necessary

A barebones Gomoku game does not need a Rust core, Wasm bridge, bot lab,
tournament reports, fixture corpora, or analysis pipeline.

Gomoku2D had that foundation anyway because it seemed like an interesting bet:
build the game in a way that could support serious rules logic, bot work,
reports, and later explanation features.

For a while, this was mostly hidden setup. The UI and basic product work did
not really touch the lab. The project could have stayed a polished retro
browser game with a reasonable bot.

### 3. Let Agents Bridge The Entry Barriers

Several important surfaces were outside the fastest personal path: frontend
UI, CSS, React, Phaser, Rust, Wasm, report rendering, static pages, and asset
preview work.

Agents made it possible to use those tools before spending weeks only getting
comfortable with them. That changed the project from "learn this stack first"
to "apply this stack to a real product question now."

This created two useful modes:

- Learning mode: pay attention, ask why the bridge/API/data shape works, and
  learn through concrete project pressure.
- Producer mode: hand off implementation details in areas that are not the
  main personal interest, then judge the result through product direction,
  visual review, screenshots, and iteration.

The important point is not that the human understood every line equally. The
important point is that the human chose which layer to own.

### 4. Hit The Domain Wall

Gomoku is simple on the surface: make five in a row. It is also simple enough
that it is easy to underestimate.

The project repeatedly ran into hidden semantics:

- open threes, broken threes, and closed threes do not force the same response.
- combo threats matter because multiple local threats can become one lethal
  condition.
- replay analysis depends on perspective: whose move, whose threat, whose
  escape, and which frame is being explained.
- Renju forbidden moves are about real threat continuations, not rough shape
  matching.
- a UI marker can mean candidate, proof result, actual move, forbidden move, or
  threat evidence depending on context.

This is where "just figure it out and execute" becomes weak. It can produce
working-looking code for the wrong model.

The human role was to notice semantic mismatches:

- this shape is not a real open three;
- this actual move should not be probed;
- this marker is confusing from the player's perspective;
- this Renju move is not forbidden because one branch is dead;
- this frame should trace farther back because it has not found the escape.

Agents were valuable after those observations: trace the logic, patch the code,
repair fixtures, regenerate reports, and sync the UI/docs. But the domain
counterexample had to exist first.

### 5. Choose Personal Taste Over Generic Optimality

The goal was rarely:

> build the strongest possible Gomoku bot

The more personal goal was:

> I have a model for how this should work. Build it, measure it, and see if the
> model holds up.

That shaped the product:

- A competent bot should avoid obvious tactical mistakes.
- A more interesting bot is not only stronger, but explainable.
- Replay analysis matters because learning where the game turned is more
  interesting than only seeing the final result.
- Reports are worth making public because the lab is part of the product
  identity.
- Rules and guide pages should teach without becoming verbose lessons.
- Visual polish matters because the game should feel intentional, not generic.

Agents can test taste. They should not erase it.

### 6. Find The Product Soul

The retro revival was the starting soul. The Rust/lab foundation was the early
bet. The project came to life when those threads finally connected.

That happened in the 0.4 lab/analyzer line. Progress slowed down, but in a good
way. The problems became deeper than expected: threat semantics, corridor
search, rolling frontier, Renju legality, report clarity, and replay analysis.

This is where the earlier foundation stopped being extra machinery. The lab
became the way the product understood itself.

The product identity became:

> a retro Gomoku game with a visible lab, strategy vocabulary, and replay
> analysis that can explain where the game turned

That is the strongest product-side proof that the agent process mattered. The
agents did not merely create more output. They helped make a more unusual
project practical.

### 7. Turn Failed Work Into Product Direction

Corridor search is the cleanest story beat.

It started as a possible live-search shortcut for bots. That version did not
promote cleanly. It was too expensive and too hard to make useful under the
browser budget.

But the concept was not wasted. It moved to replay analysis, where the shape
was better: a finished game already has an actual line, so the analyzer can
walk backward and ask where the losing side still had an escape.

That pivot created important vocabulary:

- setup corridor
- lethal onset
- last escape
- missed response
- missed escape
- forced loss

This is a central process lesson: agents make it cheaper to explore, but human
judgment decides whether a failed implementation contains a useful concept.

### 8. Make The Work Inspectable

Many intermediate artifacts would likely not exist without agents:

- bot reports;
- analysis reports;
- fixture corpora;
- screenshot review notes;
- visual guides;
- release checklists;
- generated process-story evidence;
- repeated cleanup and review passes.

These artifacts changed the project. They made decisions visible. They also
made taste cheaper to apply, because reports and pages could be tuned beyond
"functional" without consuming all project energy.

The mental shift is:

> when the cost of an artifact drops, ask whether it would make the project
> easier to reason about

## Collaboration Modes

The work did not use one agent mode. It moved between modes depending on how
clear the target was and how much judgment was needed.

### Handoff Mode

Use when the task has a clear shape.

Examples:

- implement this plan;
- review and fix;
- rerun the report;
- commit and publish;
- clean up these dead flags;
- regenerate static artifacts.

Works well for plumbing, refactors, release workflow, report regeneration, and
docs sync.

Needs manual intervention when the task hides a taste decision or the agent is
finishing the wrong shape.

### Research Mode

Use when the project needs a decision before code.

Examples:

- Should corridor search be integrated into live bot search?
- Is rolling frontier ready to replace scan?
- Why did Renju legality regress performance?
- Which bot anchors should be promoted?

Works well when the agent must inspect code, run experiments, compare reports,
and produce evidence.

Needs manual intervention to choose the strategic direction: keep pursuing,
simplify, pivot, or cut.

### Pair-Logic Mode

Use when the domain model is unclear or wrong.

Examples:

- broken-three shape handling;
- combo threat and lethal onset semantics;
- replay-analysis frame perspective;
- Renju real double-three/double-four legality;
- marker meaning in analysis UI.

Works well when the human supplies counterexamples and semantic pressure, then
the agent traces and patches the system.

Needs manual intervention because workflow polish cannot replace the correct
domain model.

### Producer Mode

Use when the human cares about product result more than implementation details.

Examples:

- report layout;
- mobile settings;
- replay status copy;
- marker colors;
- rules and guide tone;
- visual guides.

Works well when there is a fast preview loop: rebuild, serve, inspect,
screenshots, revise.

Needs manual intervention for taste. The agent can accelerate iteration, but it
cannot decide what feels like Gomoku2D.

### Housekeeping Mode

Use because agent-assisted work creates surface area.

Examples:

- stale docs;
- dead experiment flags;
- confusing API seams;
- case-specific tests;
- generated artifacts;
- release prep.

Works well with broad review passes and subagents.

Needs manual intervention to define what kind of cleanliness matters: public
clarity, maintainability, faster future agent loops, or historical context.

## Transferable Lessons

### Build Receipts Early

If a project has tuning or subjective quality work, create artifacts that make
decisions inspectable.

Reports, screenshots, fixture corpora, benchmark notes, and release notes are
not just documentation. They are how the project resists drift.

> If you cannot show why a change helped, treat it as unproven.

### Use Agents To Cross Entry Barriers

Agents are useful when a project direction is blocked by a technology entry
cost. The goal is not to pretend the entry cost does not exist. The goal is to
reach meaningful work faster, then learn from real problems.

> Let agents bridge unfamiliar stacks, but keep enough attention on the result
> to learn what matters and catch wrong abstractions.

### Choose Which Layer To Own

Not every part of a project needs the same level of personal implementation
ownership. Sometimes the right role is developer. Sometimes it is producer,
editor, reviewer, designer, or domain expert.

> AI delegation works best when the human is explicit about which layer they
> are personally owning.

### Do Not Delegate The Domain Model Blindly

Simple-looking domains can hide important semantics. A generated solution may
be structurally clean and still encode the wrong model.

> If the domain has hidden edge cases, the human needs to provide or demand
> counterexamples. Otherwise the agent will optimize the wrong simplification.

### Optimize For The Project You Want

Personal projects do not have to chase the same objective function as a
benchmark or competition. The target is often a personally held theory.

> Use agents to test your taste, not to erase it.

### Find The Project Soul Early

The earlier a project identifies its unique character, the easier it is to make
good decisions. Without that, agents can still produce output, but the output
will drift toward generic competence.

> Use agents to refine the soul of the project, but do not let the project
> proceed without one.

### Promote Concepts, Not Implementations

Corridor search failed in one role and succeeded in another. The habit is to
ask whether the idea failed, or whether this application of the idea failed.

> Failed experiments can still contain the vocabulary or model the product
> needs.

### Make Vocabulary Pay Rent

Terms like setup corridor and lethal onset are useful only if they improve the
product, docs, reports, and debugging.

> Naming is not polish at the end. Naming is how a project decides what it can
> reason about.

### Protect Momentum

On personal projects, the enemy is often not inability. It is exhaustion. A
developer may understand the concepts but still abandon the work because the
middle stretch is repetitive or mentally expensive.

> Use agents not only to go faster, but to keep the project from dying in the
> boring parts.

## Evidence To Mine

When reviewing `conversation_arcs/*.md`, look for moments that prove the arc
instead of merely adding chronology.

High-value evidence questions:

- Where did the human reject an obvious/easy path because it hurt the product?
- Where did agents bridge an unfamiliar stack enough to start real work?
- Where was the human acting as producer/editor rather than implementation
  developer?
- Which artifacts only existed because agents made them cheap enough?
- Where did agent help reduce exhaustion on work the human understood but would
  likely not finish by hand?
- Where did Gomoku/Renju semantics invalidate a plausible implementation?
- Where did the project choose a personally interesting goal over the obvious
  external metric?
- Where did project identity become clearer than it had been earlier?
- Which early bets only paid off after the lab/analyzer work made them
  necessary?
- Where did a failed implementation become a useful concept?
- Where did cleanup/refactor work unlock the next feature?

Evidence to prefer:

- report screenshots;
- before/after report or UI captures;
- release notes;
- short transcript excerpts;
- commit sequences;
- fixtures or corpus examples;
- public pages that show the final product surface.

Evidence to avoid:

- long raw transcript dumps;
- generic "AI helped" claims;
- implementation details that do not support the narrative arc;
- screenshots that only show activity, not a decision or product outcome.

## Public Shapes

### Devlog: The Failed Bot Trick That Became Replay Analysis

Best first candidate. It has a clear arc: promising idea, failed live-search
implementation, conceptual salvage, visible product feature.

### Devlog: Renju Was Not A Regex

Technical correctness story. Best if paired with rules-page diagrams and one
"looks forbidden, but is legal" example.

### Devlog: Reports As Receipts

Process/product story. Explains why the public Lab exists and why reports are
part of the identity, not just diagnostics.

### Essay: One Developer, Agents As A Small Production Team

Meta-process story. Use only with enough concrete product evidence so it does
not become generic AI commentary.

## Guardrails

- Keep the game first.
- Do not frame agents as the author of the product.
- Do not imply the human outsourced understanding.
- Do not hide failures; the pivots are the interesting part.
- Do not over-explain Gomoku strategy in the process story.
- Do not publish raw transcript chunks.
- Keep the thesis balanced: amplification with ownership.

## Current Editorial Target

The cleanest public story is probably:

> Gomoku2D started as a retro revival, but the lab/analyzer work gave it a
> deeper identity. Agents made that exploration practical by bridging stacks,
> generating receipts, reducing exhaustion, and widening the work surface. The
> project stayed coherent because the human kept owning taste, domain meaning,
> and the question of what was worth making.

This is the arc to prove with extracted conversation moments, reports,
screenshots, and release history.
