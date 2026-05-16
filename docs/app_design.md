# App Design

Scope: information architecture, player-facing flows, and screen-level UI
contracts for the current web app. Paired with `architecture.md` (runtime
boundary), `ui_design.md` (DOM shell style system), and `game_visual.md`
(Phaser canvas visual language).

This document is the canonical UI/product-design guide for the local-first app
baseline established in `v0.2`. It is intentionally focused on the playable
surfaces we have now. The `v0.3` cloud-continuity line extends Profile and
persistence, but cloud sync, published replays, and online play should not leak
into the default local play flow.

## Product frame

The `v0.2` baseline is about four things:

- move the web app onto a proper FE stack
- make the UI scalable beyond one Phaser scene
- establish consistent DOM-shell and canvas visual languages
- deepen local play with richer records, replay, and rules handling

The app remains local-first by default:

- landing on the site should feel instant
- the first meaningful interaction creates a local profile
- no sign-in should be required for the core play loop

## Core design rules

### 1. One-click play

Do not put a setup flow in front of every match.

- `Play` should start a local match immediately.
- Rule selection should stay lightweight and persistent.
- The shell should feel closer to a game title screen than a form flow.

### 2. Board first

The board is the center of gravity on match and replay screens.

- Live match UI is a slim HUD, not a dashboard.
- Replay UI is transport-first, not record-sheet-first.
- No move history panel in live match.
- No move history panel in replay.

### 3. Local record before cloud profile

The app should already feel complete as a local game.

- `Profile` is really the player's local record screen.
- Stats, preferred rules, and recent matches all work without sign-in.
- Cloud-backed identity extends this record when the player signs in; it should
  not redefine the local game.

### 4. Stable shell, swappable board themes

The DOM shell should not overfit to one exact sprite pack.

- The shell stays visually compatible with retro pixel-art board themes.
- Board sprites, stones, and effects may change later as theme sets.
- Shell layout and styling should not depend on one exact board palette.
- Canvas-specific visual rules live in `game_visual.md`.

## Routes

Current canonical surfaces:

| Route | Purpose |
|---|---|
| `/` | Home / title screen |
| `/match/local` | Local match against the current local bot preset |
| `/replay/:matchId` | Saved match replay viewer |
| `/settings` | Saved game, bot, hint, and touch-control settings |
| `/profile` | Local/cloud identity, reset/delete actions, stats, and history |

Future dedicated cloud or online routes can be added later. They are not part
of the default local-play contract unless they preserve one-click play.

## Persistent local model

The local-first model should stay simple and visible in the UI:

- a local profile is created on first meaningful interaction
- current rule, bot config, touch control, and hint preferences are persisted
- finished local matches are persisted locally
- replay reads from saved local match history

When cloud sync is enabled, it should extend these flows rather than replace
them. Local profile-to-cloud promotion copies finished local matches into private cloud
history while leaving the local copies on-device.

## Screen contracts

### Home (`/`)

Home is a title screen with immediate intent, not a setup panel.

Primary elements:

- title
- short playful subtitle or tagline
- primary `Play` action
- secondary `Profile` / record action

Optional lightweight context:

- current opponent label, e.g. `Practice Bot`
- current preferred rule, if it is worth surfacing as passive context

Rules:

- `Play` starts immediately
- no separate confirm screen
- do not let Home grow into a full configuration dashboard
- leave negative space; the page should feel like a game title screen

### Local match (`/match/local`)

The live match screen is two visual regions:

- large board
- slim side HUD

The HUD should stay compact and game-like. Keep it to three groups:

#### A. Status

This is the loudest text after the board itself.

Examples:

- `YOUR MOVE`
- `PRACTICE BOT THINKING`
- `PRACTICE BOT WINS`

Use player-based language where possible instead of stone-only language.

#### B. Match info

Compact, quiet metadata:

- current rule
- black player
- white player
- optional move count

The goal is orientation, not exhaustive detail.

#### C. Actions

Keep the action set short:

- `New Game`
- `Profile`
- `Home`
- `Replay` only after match end

Rules:

- no move list during live play
- no oversized nested cards competing with the board
- keep the side rail narrow and visually quiet
- player turn state should be obvious, but not louder than the main status line
- on narrow/tall-constrained mobile screens, allow the page to flow rather
  than letting controls overlap the board

#### Rules switching during a match

Rules stay accessible in the live HUD, but should not mutate an active game in
surprising ways.

- if the board is empty, switching rules applies immediately
- if a game is already underway, switching rules queues the new rules for the
  next game
- the UI should make that pending state explicit, e.g. `Next game: Renju`

This same interaction also updates the local preferred default.

### Match end state

The end of a game should feel like a compact payoff, not a modal interruption.

Use a stronger result block than the normal in-play HUD:

- winner/result line
- compact rule or stone-side context
- concise action row

Preferred actions:

- `Rematch` or `New Game`
- `Replay`
- `Home`

Arcade shortcut:

- clicking the finished board can still advance to the next round quickly
- shell buttons remain visible for discoverability and navigation

### Replay (`/replay/:matchId`)

Replay is the dedicated chronology surface, but it should still stay sparse.

Primary structure:

#### A. Result strip

Short, high-signal summary:

- winner/result
- move position, e.g. `Move 8 / 23`
- rule set

#### B. Transport controls

Transport is the main secondary UI on the page.

- jump to start
- previous
- auto play / pause
- next
- jump to end
- `Play From Here` once the replay is far enough in to form a meaningful branch

#### C. Timeline

The scrubber is allowed, but it should stay visually secondary to the transport
buttons and the board.

Rules:

- no move list in replay
- no extra side chronology surface
- board remains the hero
- metadata stays compact and quiet
- starting a replay slightly in is acceptable if the player can still scrub back
  to the beginning
- branching from replay should preserve the current board position and rule set,
  but it becomes a new local match with undo capped at the branch point

### Profile (`/profile`)

`Profile` is a local player-record screen first, not a settings admin page.

Primary elements:

- player name / local identity
- optional linked cloud identity
- cloud promotion/import status when signed in
- summary stats strip
- preferred rules control
- recent local matches
- clear replay entry points from history

Tone:

- think `player record`, not `settings`
- keep labels short
- keep historical rows ledger-like and scannable

Rules:

- no placeholder settings that do nothing
- summary stays fixed and visible while history can scroll independently
- recent matches should show only the metadata needed to open the right replay

### Bot controls

Bot controls are practice configuration, not profile administration. They should
not be buried inside the Profile record screen.

The first player-facing shape should have two layers:

- tested presets for normal players, selected with plain product copy
- an advanced Bot Lab layer for explicit bot configuration

The preset layer should answer "who am I practicing against?" The advanced
layer should answer "how does this bot think?" without turning the match HUD
into a debug panel.

Likely entry points:

- Home: passive current-opponent summary plus a compact configure action
- Local Match: next-game bot config change, similar to next-game rule changes
- Dedicated route or panel: full preset and advanced Bot Lab controls

Rules:

- `Play` still starts immediately with the current bot preset
- advanced config is opt-in and visibly experimental
- advanced custom configs are browser-safe, not raw lab access: D5 disables full
  width, and D7 only allows W8
- raw lab specs can be shown for transparency, but not as primary labels or
  persisted settings
- Local Match shows current-game timing only: player cards include each side's
  settled total, and the active side shows a `+current move` timer that folds
  into the total when a move lands
- Profile remains focused on player identity, cloud state, stats, and history
- replay analysis controls stay out of the first bot-controls slice

## Shared interaction rules

### Match chronology

Live play should not expose move-by-move notation by default.

- no live move list
- no replay move list
- prefer move count, transport, and board state over dense logs

### Defaults and quick overrides

There is one persistent local rules default.

- Home and Profile can both edit it
- Local Match can temporarily queue the next game rule while a game is in
  progress
- once that next game starts, the persisted default should match the selected
  rule

### Future cloud extension

When sign-in, sync, or shared replays arrive later, they should preserve the
same screen roles:

- Home stays a title/start surface
- Match stays a board-first HUD
- Replay stays transport-first
- Profile stays the player's record screen

Cloud features should add continuity and sharing, not force a new app shape.
