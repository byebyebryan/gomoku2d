# v0.2.4 UI Polish Notes

Status: ad hoc working notes, not a canonical design doc. Most of the
high-priority polish items in this note have now been implemented in the
current `v0.2.4` worktree.

Purpose: capture the current UI polish suggestions before and during
implementation so we do not lose track of them while `v0.2.4` is being scoped
and landed. This is explicitly a triage note, not a roadmap commitment and not
a replacement for
`visual_design.md` or `visual_review.md`.

Context:

- `v0.2.3` is the current desktop/mobile UI baseline
- `v0.2.4` is now the final small polish pass on top of the `v0.2.3` baseline
- that pass should stay small and avoid redesign
- after that, UI should be considered effectively frozen for the rest of the
  `0.2.x` line

## Current status

These items are now effectively done in the active `v0.2.4` worktree:

- quiet small labels globally
- tighten spacing rhythm around the shared `4 / 8 / 12 / 16 / 24 / 32` scale
- standardize icon/button alignment
- compress the desktop Match right rail
- tighten the desktop Replay transport group
- improve desktop Profile hierarchy

What remains from this note is optional tail work only, not a required polish
pass.

## Working rules

- board first
- status/result second
- controls third
- meta last
- no redesign
- no extra chrome
- no drift toward dashboard UI

## Best candidates for one last small polish pass

These are the suggestions that look highest-value and lowest-risk if we decide
to do a narrow UI cleanup before the UI freeze.

### 1. Quiet small labels globally

Examples:

- eyebrow labels
- section labels
- mini field labels
- table headers

Intent:

- labels should organize information, not compete with values
- desktop profile and side rails would benefit the most

### 2. Tighten spacing rhythm

Target spacing scale:

- `4`
- `8`
- `12`
- `16`
- `24`
- `32`

Intent:

- reduce the remaining “assembled section by section” feel
- make desktop Match, Replay, and Profile feel more deliberate

### 3. Standardize icon and button alignment

Focus:

- icon size
- icon-to-label gap
- left padding
- vertical centering

Intent:

- top nav and action buttons should feel like one reusable system

### 4. Compress the desktop Match right rail

Focus:

- tighten `Match` section spacing
- reduce empty vertical padding in player cards
- reduce vertical separation before `Undo`

Intent:

- make the rail read more like a HUD and less like a utility sidebar

### 5. Tighten the desktop Replay transport group

Focus:

- slightly reduce gaps between transport buttons
- make the controls feel like one transport object
- keep `Play From Here` clearly secondary to transport

Intent:

- stronger playback-console feel without changing the screen structure

### 6. Improve desktop Profile hierarchy

Focus:

- make the stats row the hero
- demote the identity/settings column
- soften the remaining “table” feel of history

Intent:

- the page should read as:
  - player record
  - stats
  - history
- not:
  - identity/settings
  - data table

This is the highest-value screen-specific polish area.

## Worth revisiting, but probably not for the freeze pass

These suggestions are reasonable, but either lower-value or more likely to
reopen layout work we just stabilized.

### Home breathing room on mobile

- slightly more space between eyebrow, title, copy, and button row

This is fine to revisit, but Home is already in decent shape.

### Replay desktop regrouping

- result
- participants
- transport
- contextual action

The intent is good, but the current issue is probably spacing/tightness more
than structure.

### Replay mobile metadata placement

- `Freestyle`
- `Move X / Y`

Worth revisiting only if it can be improved without crowding the control zone.

### Mobile top-zone tightening on Match

- smaller top nav buttons
- slightly tighter player cards
- slightly tighter gaps

Possible, but only if board size and tap comfort do not regress.

### Reset-button spacing on Profile

- keep the danger treatment
- just make sure it does not dominate through adjacency or spacing

This is a small follow-up, not a main pass.

## Suggestions to skip for now

These either do not buy enough, or they risk reopening stable UI decisions.

### Do not reduce `Place` prominence on mobile Match

`Place` is now part of the actual control model, not decorative chrome. It
should stay prominent while mobile touch placement exists.

### Do not tighten Replay mobile so much that the timeline touches the board again

We already fixed the “slider crowding the board” issue. Preserve the separation
that keeps the playback zone readable.

### Do not make mobile Profile entries more card-like if that means more boxes

The mobile history stack should get more scannable, not more card-heavy.

### Do not remove icons from desktop Home buttons just for variation

The gain is minor and does not justify a special-case system.

### Do not add accent glyphs or extra chrome to make status feel louder

If Match status needs more emphasis, do it by:

- quieting nearby labels
- tightening adjacent spacing
- preserving the current sparse HUD language

## Suggested polish order if we do a last UI pass

1. quiet labels globally
2. standardize icon/button alignment
3. tighten spacing rhythm
4. compress desktop Match rail
5. tighten desktop Replay transport grouping
6. improve desktop Profile hierarchy

## Recommendation

If `v0.2.4` becomes the last small UI polish release in the `0.2.x` line, the
best interpretation of this note is:

- keep the pass narrow
- only pull from the “best candidates” section unless a real regression demands
  more
- avoid reopening layout structure, mobile control flow, or icon expansion
