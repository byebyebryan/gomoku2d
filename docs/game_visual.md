# Game Visual Design

Scope: the **Phaser canvas only**. This guide defines the board-space visual
language: board rendering, stones, pointer cues, tactical warnings, sequence
numbers, z-order, and animation semantics.

It does not define the surrounding DOM shell. For app flows and screen
contracts, see `app_design.md`. For shell styling, see `ui_design.md`. For the
React/Phaser ownership boundary, see `architecture.md`.

The low-level sprite-sheet inventory lives in
`gomoku-web/assets/sprites/README.md`. This document describes how those
assets should read in play.

For local visual inspection, open
`gomoku-web/assets/preview.html` or the sprite-specific
`gomoku-web/assets/sprites/preview.html`.

## Goal

The canvas should feel like the game object itself: tactile, readable, and
board-first. Animation is part of the interaction language, not decoration.

The player should be able to tell:

- where the current move would land
- which cells are dangerous or winning
- which moves are forbidden by rules
- which line ended the game
- what happened when a stone was placed or removed

## Visual Roles

### Board

The board is the stage.

- keep the surface stable and warm
- keep grid readability high
- avoid placing non-board UI inside the canvas
- do not use board animation as ambient motion

### Stones

Stones are the most important persistent objects on the board.

- normal stones are static
- newly placed stones use `transform-form`
- removed stones use `stone-destroy`
- the last placed stone may idle while the match is still playing
- result/replay sequence numbers sit above stones only when chronology matters

The idle loop is a focus cue, not a constant board-wide effect. Only one stone
should own the idle cycle at a time.

### Pointer

The pointer is the current actionable target.

- tint follows the current player
- it sits above surface warnings and below stones
- on mobile, pointer mode jumps to the touched cell and keeps tracking direct
  drag; touchpad mode moves the pointer relatively; both modes still use an
  explicit Place confirmation
- blocked pointer state is used for occupied mobile targets and forbidden cells

Pointer modes:

| Mode | Meaning | Animation |
|------|---------|-----------|
| `normal` | legal open cell | `pointer-idle-open`, then static delay |
| `preferred` | legal winning or threat-response cell | `pointer-idle-preferred`, then static delay |
| `blocked` | occupied mobile target or forbidden open cell | `pointer-idle-blocked`, then static delay |

### Warnings

Warnings are board-cell context. They should inform the player without
covering the pointer.

| Role | Visual |
|------|--------|
| Winning move | legal immediate win: `warning` tinted green |
| Threat move | legal immediate threat: `warning` tinted red |
| Imminent threat move | defensive reply to an opponent open/broken three: `warning` tinted pink |
| Counter-threat move | counter-threat reply that can defer defense: `warning` tinted purple |
| Forbidden move | alternating `forbidden-out` and `forbidden-in` |
| Winning line | `hover` tinted green |

Winning, immediate-threat, imminent-threat, and counter-threat hints are
device-local assistive overlays controlled by two Settings rows: immediate hints
can be off, wins only, or wins plus immediate losses; imminent hints can be off, threat
replies only, or threat replies plus counter-threats. Forbidden move overlays
are rule-legality feedback and remain always on.

Forbidden cells are not active threats for Black. If a raw Black shape looks
dangerous but the required continuation is forbidden by Renju, the live board
should render the forbidden state, not a green/red "play here" warning. If a
forbidden Black square matters as evidence for a White threat, show that in
analysis/report surfaces with an explicit `F` marker rather than by upgrading the
cell into a playable Black threat.

### Sequence Numbers

Sequence numbers are chronology aids, not live-match UI.

- show them on result/replay states where move order matters
- keep them above stones and below winning-line hover
- use whole-pixel positioning to avoid text shimmer
- keep size readable but subordinate to the stones

## Z-Order

Top to bottom:

1. winning-line hover
2. sequence number
3. stone
4. pointer
5. warning surface and forbidden warning
6. board

This order is intentional. The pointer is the actionable target, while warning
surfaces are context below it. Stones remain stronger than the pointer because
they are committed board state.

## Animation Semantics

Animation should clarify a state transition or a target state.

Use animation for:

- stone placement
- stone removal
- last placed stone focus
- current pointer target
- tactical warning cells
- forbidden cells
- winning-line result emphasis

Avoid animation for:

- ambient board decoration
- UI chrome inside the canvas
- every stone at once
- states that already read clearly as static objects

Pointer movement should not reset unrelated warning or forbidden loops. Board
state changes may rebuild overlays; pointer-only movement should not.

## Asset Pipeline

Authoritative sprite sources live under `gomoku-web/assets/sprites/`.
Matching copies under `gomoku-web/public/assets/sprites/` must stay in sync
because deployed asset URLs can read from `public`.

When changing canvas assets:

- update both source and public copies
- update `gomoku-web/assets/sprites/README.md`
- update `gomoku-web/assets/sprites/preview.html` when frame layout, z-order,
  or representative cases change
- update runtime animation constants in `gomoku-web/src/board/constants.ts`
- keep sprite roles documented here if the visual language changes

The sprite README is the frame table. This document is the meaning table.

## Implementation Boundary

Phaser owns board-space visuals and board input. It should receive declarative
state from React and emit intent events back up.

Game logic still belongs below the scene:

- win detection in `gomoku-core` / `gomoku-wasm`
- rolling-frontier threat snapshots in `gomoku-bot` / `gomoku-wasm`
- immediate win, immediate threat, imminent reply, counter-threat, and forbidden
  move sets from that snapshot

The canvas should render those facts. It should not rediscover rules by
duplicating game logic in the scene.

## Future Theme Rule

Future board themes should mostly swap canvas assets:

- board surface and frame
- grid and marks
- stones
- pointer
- warning and result effects

The DOM shell should not need a redesign for each board theme. The shell is the
cabinet; the board theme is the cartridge.
