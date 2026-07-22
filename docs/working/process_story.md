# Process Story Mining

Purpose: collect private evidence for a future devlog or public "making of"
story without turning raw agent transcripts into public copy.

This is a working note, not canonical product documentation. The generated
artifacts live under ignored output paths and should be treated as source
material for curated writing.

Status: parked after the `v0.5.3` public alpha. This material is available when
there is time and interest to publish a personal process story, but it is not a
release dependency or an active product task.

Curated story candidates live in
[`Process Story Leads`](process_story_leads.md). Rough public-copy drafts live
in [`Process Story Drafts`](process_story_drafts.md). The current external
making-of package lives in
[`Process Story Devlog Kit`](process_story_devlog_kit.md), with capture
planning in
[`Process Story Visual Storyboard`](process_story_visual_storyboard.md).
The current process-framing note is
[`Process Story Narrative Context`](process_story_narrative_context.md). The
current evidence map is
[`Process Story Evidence Map`](process_story_evidence_map.md). The active
curated proof packet for the external story is
[`Process Story Evidence Cards`](process_story_evidence_cards.md).

## Current Extraction

Regenerate from the repo root:

```sh
python3 scripts/extract_process_story.py --before-date 2026-05-30
```

The date cutoff is applied per event, so it keeps the extraction pass from
ingesting its own current session/subagent logs even when a long session spans
multiple dates. Remove or move the cutoff only when intentionally mining newer
sessions.

Generated private outputs:

- `gomoku-bot-lab/outputs/process-story/session_index.json`
- `gomoku-bot-lab/outputs/process-story/evidence_events.jsonl`
- `gomoku-bot-lab/outputs/process-story/conversation_arcs.jsonl`
- `gomoku-bot-lab/outputs/process-story/conversation_arcs.md`
- `gomoku-bot-lab/outputs/process-story/conversation_arcs/*.md`
- `gomoku-bot-lab/outputs/process-story/git_chronology.json`
- `gomoku-bot-lab/outputs/process-story/quote_candidates.md`
- `gomoku-bot-lab/outputs/process-story/process_outline.md`

The default output path is ignored by git. The extractor refuses to write raw
snippets into any trackable repo path because these outputs contain private
transcript excerpts.

Curated working notes:

- `docs/working/process_story_narrative_context.md`
- `docs/working/process_story_leads.md`
- `docs/working/process_story_drafts.md`
- `docs/working/process_story_devlog_kit.md`
- `docs/working/process_story_visual_storyboard.md`
- `docs/working/process_story_evidence_map.md`
- `docs/working/process_story_evidence_cards.md`

Latest first pass:

- Included sessions: `51`
- Evidence events: `9,936`
- Conversation arcs: `1,131`
- Git commits scanned: `587`
- Release bands: `v0.2` 2,390 events, `v0.3` 884, `v0.4` 5,656, `v0.5` 1,006
- Top topics: project references, corridor analysis, bot/lab reports, Renju
  correctness, replay analysis, product foundation, revival stack, rolling
  frontier, AI process, public release

The extractor intentionally filters out system/developer instructions, memory
summaries, current process-story self-references, and adjacent-project chatter
unless the snippet contains Gomoku-specific work. `conversation_arcs.md` is a
small index that points to split arc files; those chunks are the best place to
review narrative flow because they group high-signal user turns with nearby
assistant rationale and outcome hints.

## Narrative Spine

1. Revival: an old Gomoku project becomes a modern Rust/Wasm/browser product.
2. Product foundation: local-first play, profiles, replay, cloud continuity,
   and mobile polish make it a real app.
3. Lab turn: bot tuning shifts from vibes to tournaments, reports, benchmarks,
   and reproducible evidence.
4. Strategic pivot: corridor search fails as a broad live-search shortcut but
   becomes the vocabulary and engine behind replay analysis.
5. Hard lessons: corridor portals do not promote cleanly, rolling frontier
   pays off, and Renju legality requires recursive proof instead of rough shape
   matching.
6. Productization: replay analysis, reports, rules, guide, visuals, and release
   flow turn lab work into public surfaces.
7. Process thesis: one developer supplies taste and judgment; agents supply
   exploration, implementation throughput, review passes, and evidence mining.

## Candidate Public Angles

- Product-first: "Play instantly, then learn where the game turned."
- Lab-first: a Gomoku game that carries a visible bot/analyzer lab with it.
- Technical highlight: replay analysis is the distinctive feature; bot strength
  work matters most when it makes the game explainable.
- Process-first: for a making-of post only, one indie developer used agents as
  a small production team while still steering taste, scope, and correctness.

## Review Rules

- Do not publish raw transcript dumps.
- Use quotes sparingly, keep them short, and prefer paraphrase unless the exact
  wording matters.
- Verify any claim against current repo state, release notes, or generated
  reports before making it public.
- If the process leads, keep shipped game surfaces as the evidence. The
  agent-assisted process is production context, not a substitute for explaining
  why the game is worth trying.

## If Resumed

- Curate 5-8 concrete moments from `conversation_arcs.md`,
  `quote_candidates.md`, and `git_chronology.json`.
- Use [`Process Story Narrative Context`](process_story_narrative_context.md)
  to decide which moments are meaningfully distinctive or transferable.
- Use [`Process Story Evidence Map`](process_story_evidence_map.md) to connect
  each chosen claim to arc IDs, commits, reports, and public surfaces.
- Use [`Process Story Evidence Cards`](process_story_evidence_cards.md) as the
  active proof packet before writing final public copy.
- Use [`Process Story Leads`](process_story_leads.md) as the editorial shortlist.
- Refine [`Process Story Devlog Kit`](process_story_devlog_kit.md) into the
  chosen external format.
- Use [`Process Story Visual Storyboard`](process_story_visual_storyboard.md)
  to pair each moment with a screenshot, report excerpt, release tag, or commit.
- Keep this external-only for now; do not add an in-app story route unless the
  product navigation is revisited later.
- Archive or delete these process-story working notes after a public story has
  been written; do not let them become permanent canonical docs.
