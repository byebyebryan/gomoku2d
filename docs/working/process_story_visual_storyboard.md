# Process Story Visual Storyboard

Purpose: capture plan for the external making-of devlog. This is not a
screenshot review log. It is the list of visuals needed to make the process
story richer than text.

The selected proof targets live in
[`Process Story Evidence Cards`](process_story_evidence_cards.md). Use this
storyboard for capture defaults, caption tone, and cut rules.

Source of truth for screenshot QA remains
[`UI Screenshot Review`](ui_screenshot_review.md).

## Capture Defaults

- Use a production-style local build, not Vite dev UI.
- Prefer desktop captures for article readability; add mobile only when the
  point is mobile or local-first accessibility.
- Use current `0.5.x` app surfaces. Do not reuse older captures unless the
  caption explicitly says it is historical.
- Keep visuals tight: crop to the relevant panel, board, or report section
  instead of dumping full-page screenshots.
- Avoid raw transcript screenshots. Process visuals should show shipped
  artifacts, reports, commits, or docs, not private chat logs.

## Required Visual Set

- Hero: Home plus active board.
  Source: `/` and `/match/local`.
  Capture: desktop hero, optional mobile board.
  Caption angle: the project is still a playable game first.
- Product loop: Settings/Profile/Replay history.
  Source: `/settings/`, `/profile/`, replay route.
  Capture: one desktop or mobile composite.
  Caption angle: guest-first play became a real product loop.
- Lab discipline: Bot report ranking/search.
  Source: `/lab/` ranking and `/lab/?tab=search`.
  Capture: cropped report cards/table.
  Caption angle: the lab made bot work measurable instead of vibes.
- Analyzer pivot: Replay Analysis timeline.
  Source: a seeded finished replay.
  Capture: crop board, status, and timeline.
  Caption angle: the useful question is where the game turned.
- Proof surface: Lab analysis proof frame.
  Source: `/lab/?tab=analysis&match=match_0065__search-d1__vs__search-d3_pattern-eval`.
  Capture: auto-expanded `match_0065__search-d1__vs__search-d3_pattern-eval`.
  Caption angle: corridor search became replay-analysis vocabulary.
- Renju correctness: Rules complex Renju example.
  Source: `/rules/`.
  Capture: crop real double-three examples.
  Caption angle: forbidden moves depend on real threats, not rough shape.
- Public surfaces: Rules/Guide/Lab/Visuals grid.
  Source: `/rules/`, `/guide/`, `/lab/`, `/visuals/`.
  Capture: four small crops or one contact sheet.
  Caption angle: lab work became understandable public surfaces.
- Process hook: agent-assisted breadth with shipped proof surfaces.
  Source: contact sheet from `/rules/`, `/guide/`, `/lab/`, `/visuals/`.
  Capture: four tight crops or one compact contact sheet.
  Caption angle: agents widened the work surface; judgment kept it coherent.
- Process loop: Release/commit timeline.
  Source: `CHANGELOG.md`, release tags, commits.
  Capture: rendered timeline or simple graphic.
  Caption angle: agents expanded throughput; release discipline kept it coherent.

## Nice-To-Have Visuals

- Before/after reports: old generated report vs current Lab.
  Use only if the contrast is visually clear and not too noisy.
- Corridor failure: experiment notes or report metric.
  Use only if it can be shown without drowning readers in flags.
- External Renju validation: fixture/corpus excerpt.
  Use only if the example is readable in a small crop.
- Agent team loop: review/fix/commit pattern.
  Use only if it shows process without exposing raw private transcript.

## Selected Follow-Up Capture Queue

These are the exact targets selected by the current evidence-card pass:

1. Home plus active board.
2. `/lab/` ranking tab.
3. `/lab/?tab=search` with timing bars.
4. `/lab/?tab=analysis&match=match_0065__search-d1__vs__search-d3_pattern-eval`.
5. `/rules/` complex Renju example.
6. `/guide/` forced-sequence example.
7. `/visuals/` style guide overview.
8. Compact release/commit timeline graphic.

## Cut If Noisy

- Dense tournament tables with long bot specs.
- Full-page mobile captures unless the scroll behavior matters.
- Raw quote-candidate or evidence-event output.
- Large architecture diagrams that need too much explanation.
- Multiple similar replay-analysis frames. One strong frame is better than a
  sequence that requires tactical context.

## Capture Checklist

Before final publishing:

1. Serve a clean production build locally.
2. Open each source route and confirm the app version/copy matches the target
   release.
3. Capture the required visuals at consistent desktop width.
4. Crop each visual to the article point.
5. Write a one-sentence caption for each visual.
6. Verify captions avoid solver, autonomy, and bot-strength overclaims.
7. Store only release-relevant captures under `docs/assets/`; move rejected
   or historical material to archive or leave it untracked.

## Caption Drafts

- Home/match: `The surface stays simple: start a match, place stones, and keep the board first.`
- Product loop: `The product work was making local play, settings, history, and replay feel like one loop.`
- Lab ranking: `The bot lab exists so changes are measured, not guessed.`
- Replay analysis: `Replay Analysis walks backward from the ending to find the last meaningful turn.`
- Proof frame: `The same report machinery that tested bots became a way to explain finished games.`
- Renju rules: `Renju legality is about real threat branches, not just matching shapes.`
- Public surfaces: `Rules, Guide, Lab, and Visuals turn internal work into something a new reader can inspect.`
- Process timeline: `Agents widened the work loop; release checkpoints kept it from becoming scattered.`
