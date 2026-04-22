# Gomoku2D Revised UI Brief

## Product direction

Simplify the flow and restore retro charm by removing:

- dedicated pre-match setup screen
- in-match move history panel

Core principle:

- less friction before play
- less chrome during play
- detailed chronology belongs in replay, not in active match UI

---

## New flow

- Home
- Local Match
- Result state
- Replay or Home

Meta flow:

- Home
- Profile / Record
- Replay browser

---

## Key changes

### Remove pre-match screen

Do not require users to confirm match options before every game.

Instead:

- `PLAY` starts immediately
- use persistent defaults for rule set / opponent
- expose only a lightweight rules toggle on Home or Profile

### Remove in-match move history

Do not show move list during active match.

Reason:

- it adds visual noise
- it weakens board dominance
- it makes the game screen feel like a tool/dashboard

Move chronology belongs in Replay.

---

## Home screen

### Goal

Minimal title screen with immediate play.

### Contents

- title
- tagline
- primary `PLAY` button
- secondary `RECORD` button
- lightweight rule toggle:
  - `FREESTYLE | RENJU`

Optional tiny status line:

- `VS: CLASSIC BOT`

### Rules

- do not turn Home into a full setup screen
- only expose 1–2 quick-start options
- keep lots of negative space

---

## Match screen

### Goal

Board-first arcade HUD.

### Layout

Two regions only:

- large board
- slim right HUD

### HUD contents

#### 1. Status

Examples:

- `BRYAN TO MOVE`
- `CLASSIC BOT THINKING`
- `BLACK WINS`

#### 2. Compact match info

- `RULE: RENJU`
- `BLACK: BRYAN`
- `WHITE: CLASSIC BOT`

#### 3. Actions

Keep action count low:

- `HOME`
- `RESET`
- `REPLAY` only after match end

### Remove

- move history panel
- oversized player cards
- repeated section boxes
- verbose labels

### Styling

- board must be dominant
- right rail should be narrow and quiet
- use spacing + dividers, not many nested borders
- status/result should be the loudest element after the board

---

## Match end state

### Goal

Compact but stronger emotional payoff.

### Replace normal HUD status with result block

Example:

- `CLASSIC BOT WINS`
- `BLACK · RENJU`

### Actions

- `REMATCH`
- `REPLAY`
- `HOME`

---

## Replay screen

### Goal

Single dedicated place for chronology.

### Contents

- large board
- result/status strip
- transport controls
- move list / timeline

### Controls

Use a compact transport row:

- `|<`
- `<`
- `AUTO`
- `>`
- `>|`

### Rules

- replay can contain move history
- keep it cleaner than current v2
- board still remains the main visual focus

---

## Profile / Record screen

### Goal

Save-record screen, not settings admin.

### Contents

- player name / local player identity
- stats
- default rule set toggle
- recent matches
- replay access

### Tone

Use labels like:

- `PLAYER RECORD`
- `DEFAULT RULE`
- `RECENT MATCHES`

Avoid:

- settings-page feel
- long technical IDs as primary content

---

## Visual rules

### Board dominance

- increase board share of screen on match and replay
- slim down side panels
- remove low-value info from active match screen

### Boxes

- fewer nested borders
- strong borders only for top-level frames and buttons
- inside panels, prefer spacing and dividers

### Copy

- short, game-like wording
- fewer labels
- less metadata during active play

### Buttons

- chunky, tactile, clear pressed states
- green = primary action
- red = destructive/reset
- teal = secondary navigation
- yellow = titles / result / emphasis

---

## Palette

- BG: `#1E1E1E`
- Panel: `#2A2A2A`
- Accent: `#FCCB57`
- Text: `#F5F5F5`
- Action: `#30A860`
- Danger: `#E04747`
- Secondary: `#5F8B82`

---

## Screen tone targets

- Home = title screen
- Match = arcade HUD
- Replay = VCR transport
- Profile = save record
