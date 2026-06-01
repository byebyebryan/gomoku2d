# Process Story Leads

Purpose: turn the private process-story extraction into candidate public
stories. This is not public copy yet. Treat it as an editorial shortlist.

Source material:

- `gomoku-bot-lab/outputs/process-story/process_outline.md`
- `gomoku-bot-lab/outputs/process-story/conversation_arcs.md`
- `gomoku-bot-lab/outputs/process-story/conversation_arcs/*.md`
- `gomoku-bot-lab/outputs/process-story/quote_candidates.md`
- `gomoku-bot-lab/outputs/process-story/evidence_events.jsonl`
- `gomoku-bot-lab/outputs/process-story/git_chronology.json`
- `docs/working/process_story_evidence_map.md`
- `docs/working/process_story_evidence_cards.md`

## Strongest Story Candidates

### 1. An Old Favorite, Built Properly

Core angle: this started as a personal revival of a familiar paper game, not a
generic "make a game with AI" demo.

Why it works:

- It gives the project emotional grounding.
- It explains why a simple game deserves careful engineering.
- It sets up the contrast between humble gameplay and serious internals.

Evidence hooks:

- Early core commits on 2026-04-15: Rust core, replay format, Renju variant,
  Wasm bridge, bot integration.
- Home copy settled around "An old favorite, built properly."
- v0.2 turned the prototype into local-first play with replay, settings,
  profiles, mobile layout, and visual polish.

Likely public shape:

- README intro or first devlog.
- Pair with the current home screen, a short gameplay clip, and the old/new
  architecture split.

Risk:

- If overdone, it can become sentimental rather than concrete. Keep the proof
  in the product: immediate play, clean replay, and a real rules core.

### 2. The Bot Lab That Became The Product Identity

Core angle: the bot work did not merely produce a stronger opponent. It created
the lab/report culture that now defines Gomoku2D.

Why it works:

- It reframes "bot tuning" as measurable product discovery.
- It explains why reports are public surfaces rather than internal scraps.
- It connects tournament anchors, pooled budgets, search metrics, and UI
  pages into one story.

Evidence hooks:

- 2026-05-01 commits added configurable search bot plumbing, eval harness,
  tournament reports, and published bot reports.
- Multiple v0.4 commits polished report terminology, timings, rankings, and
  curated report artifacts.
- v0.5.1 moved reports from Rust-generated HTML to web-rendered viewer/data.

Likely public shape:

- A short Lab page explanation.
- A devlog section: "The reports are not screenshots of work; they are how the
  work happened."

Risk:

- Too much bot terminology will lose normal players. Lead with "we can show
  what the bot did and why a game turned," not with search flags.

### 3. Corridor Search Failed As Magic, Then Became The Analyzer

Core angle: the most important feature came from a failed optimization path.
Corridor search was explored as a way to make bots stronger, but its durable
value was explaining finished games.

Why it works:

- It has conflict: the first plan did not work.
- It has a clear pivot: live-search shortcut to replay-analysis vocabulary.
- It is specific to Gomoku strategy, not generic AI/process talk.

Evidence hooks:

- v0.4.2 formalized threat corridors, last escape, bounded proof, and analysis
  reports.
- v0.4.3 and v0.4.4 repeatedly tested corridor portals/proof in bot search,
  then backed away from the paths that did not promote.
- Replay analysis kept the useful part: walking backward from the finish to
  find the last escape and setup corridor.

Likely public shape:

- Best standalone devlog candidate.
- Title direction: "The failed bot trick that became replay analysis."
- Pair with one analysis report frame showing last escape, setup corridor, and
  lethal onset.

Risk:

- Need to avoid overclaiming. Corridor analysis is bounded explanation, not a
  complete game solver.

### 4. Renju Rules Were Not A Regex

Core angle: the Renju forbidden-move checker looked like shape matching until
real examples proved that legality depends on recursive "real threat" proof.

Why it works:

- It is a concrete correctness rabbit hole.
- It shows engineering rigor: external examples, corpus extraction, reference
  validation, and then integration.
- It ties directly into the game's learning value because Renju legality is
  confusing for players too.

Evidence hooks:

- v0.4.7 added lethal-threat semantics, then exposed gaps in the old Renju
  forbidden checker.
- 2026-05-18/19 commits added a recursive Renju legality checker, Renju corpus,
  external example extraction, metrics, and performance fast paths.
- The rules page later translated that complexity into "real" double-three and
  double-four examples.

Likely public shape:

- A compact technical post.
- Use the rules page diagrams and one "looks forbidden, but is legal" example.

Risk:

- This can get too deep quickly. Keep it to the rule insight: a forbidden move
  is about real lethal threats, not rough visual shape.

### 5. Rolling Frontier: The Performance Work That Actually Paid

Core angle: as threat detection became central to bots, hints, and analysis,
full-board scans became the bottleneck. Rolling frontier was the successful
infrastructure work behind later features.

Why it works:

- It demonstrates measurement-driven engineering.
- It contrasts with corridor portals: some ambitious ideas failed, boring
  infrastructure paid off.
- It explains why the browser version can carry richer hints and analysis.

Evidence hooks:

- v0.4.4 explored rolling frontier in shadow mode before promoting it.
- The implementation kept scan-backed fallback semantics while pushing more
  hot paths through threat views.
- Later optimizations cached pattern eval, tactical summaries, Renju filters,
  and stage timing.

Likely public shape:

- Internal/technical devlog, not homepage material.
- Good as a supporting sidebar in a broader "what makes the analyzer possible"
  story.

Risk:

- It is easy to oversell as user-visible. Frame it as enabling work, not a
  feature headline.

### 6. Reports Went From Dev Artifact To Product Surface

Core angle: reports started as diagnostics, became the way the project thinks,
then were rebuilt as real product pages.

Why it works:

- It shows the project becoming more legible.
- It justifies why `/lab/` exists.
- It connects repo hygiene with product presentation.

Evidence hooks:

- v0.4 produced bot and analysis report artifacts to validate tuning and
  replay-analysis logic.
- v0.5.1 replaced huge Rust-generated HTML with compact report JSON and React
  viewer pages.
- Static pages later aligned Rules, Guide, Lab, and Visuals into one public
  surface set.

Likely public shape:

- A short section in the 0.5 release post.
- Visual comparison: old report artifact vs current Lab page.

Risk:

- Avoid making it sound like a reporting tool instead of a game. Reports should
  support the "learn where the game turned" hook.

### 7. One Developer, Agents As A Small Production Team

Core angle: the interesting process is not "AI made a game." It is one
developer using agents for breadth, review, implementation throughput, and
evidence mining while keeping human judgment in charge.

Why it works:

- It is honest about what agents did well and poorly.
- It matches how the project was actually built: design checkpoints, reviews,
  subagents, release prep, screenshot review, report validation.
- It is a useful meta-story for devlog readers without displacing the game.

Evidence hooks:

- Repeated review/fix/commit loops.
- Subagent review passes for docs, frontend, static pages, analyzer, and repo
  cleanup.
- Human steering is visible in copy, terminology, model corrections, UI
  judgement, and deciding which experiments to abandon.

Likely public shape:

- "How this was built" section after the product story is clear.
- Could be an itch.io devlog or separate post.

Risk:

- If this leads, the project becomes an AI-process demo. It should be the
  second layer after the playable product and analyzer hook.

## Best Devlog Sequence

1. `An Old Favorite, Built Properly`
   Establish the project: simple game, personal reason, modern Rust/Wasm/web
   stack, local-first browser product.

2. `The Game After The Game`
   Introduce replay analysis as the unique hook: finished games become practice
   material, and the analyzer finds where the game turned.

3. `The Failed Bot Trick`
   Tell the corridor-search pivot: why stronger-bot work got less interesting
   than explainable-bot work.

4. `Renju Rules Were Not A Regex`
   Show the correctness rabbit hole and why the rules page/analyzer needed a
   real legality model.

5. `Reports Became The Lab`
   Explain why the public Lab page exists and how the project used reports to
   make decisions.

6. `One Developer, Agents As The Small Team`
   Close with process: what agents accelerated, where human steering mattered,
   and why the repo/release discipline mattered.

## What Not To Lead With

- Raw tournament strength. The bots are useful, configurable, and measurable,
  but "strongest Gomoku bot" is not the promise.
- Full solver claims. The analyzer is bounded and designed to explain the
  decisive corridor, not solve Gomoku.
- AI-assisted implementation before product value. The process is interesting
  because the product has a shape, not the other way around.
- Deep performance details. Rolling frontier and tactical ordering are
  supporting stories unless the audience is already technical.

## Next Curation Pass

- Review `process_story_evidence_cards.md` as the active proof packet.
- Capture the selected report, Renju, guide, lab, and product visuals from the
  evidence cards.
- Decide which story belongs in README, which belongs on itch.io, and which
  belongs in devlog posts.
