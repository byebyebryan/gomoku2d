# Gomoku2D UI Language / Design System Doc
**Date:** 2026-04-18  
**Purpose:** define the visual language and reusable rules for the Gomoku2D UI  
**Audience:** coding agents, artists, and developers

---

## 1. Intent

This document defines the visual language for Gomoku2D.

It is not a full art bible.  
It is a practical UI design-system reference for building new screens and components coherently.

The main objective is:

> Make every UI element feel like it belongs to the same world as the board, stones, pointer, and existing pixel assets.

---

## 2. Core style identity

Gomoku2D UI should feel like:

- **pixel tactical board UI**
- **board-adjacent hardware**
- **simple, bold, readable**
- **playful but restrained**
- **low-fidelity and high-clarity**

It should **not** feel like:
- a separate glossy app skin
- colorful mobile app icons pasted over a board game
- a richer-fidelity menu system that visually outruns the game assets

---

## 3. Style pillars

### 3.1 Silhouette first
Every asset should read from its outer shape first.

### 3.2 Limited color
Use color as a role signal, not as decoration.

### 3.3 Board-material coherence
UI should feel made from the same material family as stones/buttons/pointer.

### 3.4 Small expressive accents
Charm comes from timing, shape, and tiny motion, not from richer rendering detail.

### 3.5 Readability at small size
Everything should remain legible and understandable at small pixel scales.

---

## 4. Fidelity target

The target fidelity is close to the current board assets:
- low-detail
- hard-edged
- small accent use
- simple highlight/shadow treatment
- 16x16-ish base vocabulary

### Important caution
Do not drift upward into a richer “app icon” fidelity tier unless the core game assets also evolve with it.

The UI library should be normalized to the game assets, not the other way around.

---

## 5. Grid and sizing

### Base unit
- **16x16** is the core icon and sprite unit

### Spacing rhythm
Use a simple step system:
- 4 px
- 8 px
- 16 px
- 24 px
- 32 px

### Usage guidance
- 4 px = micro spacing
- 8 px = internal spacing
- 16 px = component padding and gaps
- 24/32 px = section and layout spacing

---

## 6. Typography

### General rules
- bold pixel display font for titles and major labels
- simple pixel font for body/meta text
- strong contrast between title and body use
- avoid long dense paragraphs in core screens

### Roles
- **Title** = game logo, section headers
- **Meta** = ruleset, timer, small labels
- **Action label** = button text
- **Body/support** = helper text, recent activity, descriptions

### Guidelines
- use uppercase sparingly for emphasis and actions
- use mixed-case or calmer pixel body for supporting text if available
- keep text blocks short and scannable

---

## 7. Color system

Color is semantic.

### Base neutrals
- background dark
- panel dark
- panel elevated dark
- muted gray
- light gray
- white

### Semantic accents
- **green** = primary / confirm / ready / active / your turn
- **blue** = online / room / network / join
- **purple** = replay / history / review
- **yellow** = title / meta / focus / warning-lite
- **red** = destructive / danger / resign / reset

### Rules
- use one semantic accent per element wherever possible
- avoid multi-accent icons unless absolutely necessary
- reserve brighter saturation for important actions and states

---

## 8. Shadows and depth

### Intent
Depth should feel like old-school game UI hardware, not soft modern app elevation.

### Rules
- one primary shadow step
- one inset/pressed state
- sharp, readable edges
- avoid soft glows or blurred web-style shadows unless explicitly justified

### Pressed state
Buttons should feel like they physically depress:
- 1–2 px visual shift
- shadow collapse
- possible tiny highlight change

---

## 9. Button language

Buttons are one of the strongest existing style anchors.

### Core button principles
- keep them stone-adjacent
- retain simple highlight/shadow logic
- use clear rectangular silhouette
- keep label centered and readable

### Required button families
- Primary
- Secondary
- Destructive
- Toggle
- Icon
- List-row action

### Role meanings
- Primary = proceed / play / join / resume
- Secondary = browse / open / menu / review
- Destructive = reset / resign / delete / leave
- Toggle = option switching
- Icon = lightweight supporting actions

---

## 10. Panel language

Panels are the calmer structural layer around the expressive board/game pieces.

### Panel roles
- SidebarPanel
- ModalPanel
- CardPanel
- ListPanelRow

### Rules
- dark neutral base
- simple border
- simple shadow
- structured spacing
- restrained highlight use
- panels should frame content, not compete with it

### Sectioning
Use dividers and section headers rather than many different panel styles.

---

## 11. Icon language

Icons must remain coherent with the board assets.

### Rules
- 16x16 base grid
- strong silhouette first
- grayscale base + one accent preferred
- max 2–3 colors
- minimal interior detail
- same outline/highlight logic as stones/buttons/pointer
- should feel like small tactical markers, not full illustrations

### Good icon traits
- obvious role
- readable at tiny size
- category signal through shape + one accent
- slight personality without added clutter

### Bad icon traits
- too many colors
- soft or decorative detail
- rounded “app sticker” look
- richer pixel clustering than the board pieces

---

## 12. Recommended icon families

### Action
- play
- next
- back
- share
- open
- copy

### Identity
- human
- bot
- guest
- profile
- sign in

### State
- ready
- thinking
- connected
- disconnected
- warning
- locked
- success
- danger

### Navigation / product
- online
- room
- replay
- trophy
- menu
- timer

---

## 13. Status language

Statuses should be compact, clear, and reusable.

### Preferred forms
- badges
- chips
- small labels
- compact card sublines

### Common statuses
- Your Turn
- Thinking...
- Ready
- Connected
- Disconnected
- Offline
- Replay
- Guest
- Won +1

### Rule
Status UI should not require whole new component styles.

---

## 14. Player card language

Player cards are key bridge components between game and app shell.

### Should support
- human / bot / online
- timer
- wins
- turn state
- ready/connection/thinking states
- optional avatar/rating later

### Visual behavior
- clear identity block
- one or two lines of metadata
- optional status line
- strong but calm card treatment
- should still feel board-adjacent

---

## 15. Motion language

UI motion should feel tactile and low-frame.

### Good motion patterns
- blink
- pulse
- 1–2 px nudge
- focus bracket flicker
- tiny pop-in
- warning flash
- status beacon

### Avoid
- long easing-heavy transitions
- exaggerated bounce
- mobile-app-like fluidity
- too many simultaneous animations

### Rule of thumb
- 2-frame or 4-frame loops preferred
- 1–2 px movement max for most UI
- motion clarifies state, not decoration

---

## 16. Board overlay language

Board overlays need to remain restrained and game-relevant.

### Allowed
- last move marker
- move numbers in review
- hover target
- legal move cues
- subtle win line
- tiny candidate markers

### Avoid
- UI clutter on the board
- profile/auth/room UI inside board space
- unrelated decorative overlays

The board should remain readable first.

---

## 17. Home / app-shell style

The app shell can be slightly calmer than the board/game pieces.

### Good balance
- board/game interactions stay very game-like
- app-level panels become more structured and quiet
- identity cards and lists use restrained panel language
- semantic color helps category navigation

### Avoid
- making Home/Online/Profile feel like a separate design language
- over-coloring category surfaces
- icons becoming richer than game pieces

---

## 18. Visual hierarchy rules

### Highest emphasis
- board
- current turn / active player
- primary action
- result state

### Medium emphasis
- player cards
- room/replay list items
- top-level navigation items

### Low emphasis
- support text
- footer links
- helper copy
- version/build info

The hierarchy should remain obvious even without color.

---

## 19. Component families to enforce

### Primitives
- PixelButton
- PixelIcon
- Panel
- Badge
- Divider
- ToggleGroup
- FocusMarker

### Composite
- PlayerCard
- MenuSection
- RoomRow
- ReplayRow
- ProfileChip
- StatusBlock

### Screen-level structures
- MatchSidebar
- MatchMenuPanel
- OnlineListPanel
- ReplayBrowser
- ProfilePanel
- HomeLauncher

Every new UI element should ideally derive from these families.

---

## 20. Coherence checklist

Before adding a new UI element, ask:

1. Does it feel like it belongs to the same material family as the board assets?
2. Is the silhouette readable first?
3. Is color used semantically, not decoratively?
4. Is it richer in detail than the existing game pieces?
5. Could this be composed from existing primitives instead of invented fresh?
6. Is the motion low-frame and tactile enough?
7. Does it belong to Match, Menu, Online, Replays, or Profile?
8. Is it trying to solve a layout problem with art instead of structure?

If answers drift the wrong way, simplify.

---

## 21. Rules for coding agents

1. Do not upscale fidelity casually.
2. Do not add colorful icons that look like a different game.
3. Reuse button/panel/icon families before creating one-offs.
4. Match semantic colors to meaning consistently.
5. Preserve board dominance.
6. Prefer stronger silhouette over richer pixel detail.
7. Use animation only when it improves clarity or delight without style drift.
8. If uncertain, simplify rather than embellish.

---

## 22. Final recommendation

The best visual path for Gomoku2D is:

- keep the current pixel tactical identity
- build the UI library at the same asset fidelity as the game pieces
- allow the app shell to be calmer, but not stylistically separate
- use semantic color and tiny motion to scale the interface
- let coherence come from material language, not from making everything more elaborate
