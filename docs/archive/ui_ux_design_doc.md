# Gomoku2D UI/UX Design Doc
**Date:** 2026-04-18  
**Purpose:** handoff document for coding agents and human developers  
**Scope:** game UI, app shell, visual language, scalable screen structure, and implementation priorities

---

## 1. Executive summary

Gomoku2D already has a strong visual identity:

- pixel-art presentation
- 16x16-ish sprite vocabulary
- board-first composition
- stone/button/pointer assets that feel cohesive
- lightweight animation that can add charm without needing high-fidelity art

The main UX risk is **not** weak style. The main risk is **reactive feature growth**:

- adding buttons and screens as features appear
- mixing match controls with app-level navigation
- letting online, profile, replay, and settings all accumulate inside the same shallow UI structure

### Main direction

The UI should evolve into:

- a **board-first match view**
- a **formal sidebar / HUD system**
- a **match-scoped menu panel**
- separate **app-level screens** for Home, Online, Replays, and Profile
- a reusable **pixel UI library** that matches the fidelity of the board/stones/pointer assets

The design goal is:

> Keep the interface feeling like it belongs to the game world,  
> while giving it enough structure to scale into a real app.

---

## 2. Core UX principles

1. **Board dominance first**  
   The board remains the visual and interaction center of the product.

2. **Match UI is not app UI**  
   Match-scoped controls stay in the match shell. Account, online, and library features get their own top-level places.

3. **Pixel cohesion over feature-specific decoration**  
   New UI pieces should feel like they are made from the same material family as the stones, pointer, and buttons.

4. **Simple hierarchy, strong readability**  
   The game should still feel quick and arcade-like, not like a heavy enterprise dashboard.

5. **Reusable primitives, not one-off solutions**  
   New features should be composed from panels, cards, buttons, badges, and list rows rather than inventing new styles per feature.

6. **Landscape and portrait share components, not layouts**  
   Reuse the same UI kit across device classes, but allow different structural layouts.

---

## 3. Product information architecture

Top-level app structure:

- **Home**
- **Match**
- **Online**
- **Replays**
- **Profile**

### Responsibilities

#### Home
Entry point and launcher for the main game modes.

#### Match
Live gameplay or match viewing.

#### Online
Room creation, room join, invites, match status, future matchmaking.

#### Replays
Saved replays, shared replays, replay browsing, review mode.

#### Profile
Auth, username, linked account, preferences, player identity.

### Important rule

Do **not** use the Match Menu as the place for:
- sign in
- account linking
- profile
- replay library
- room browser
- broader online navigation

Those belong to app-level screens.

---

## 4. Primary UX modes

Inside the app, explicitly distinguish these modes:

### A. Home mode
Main navigation and launch surface.

### B. Match mode
Live local/online/bot gameplay.

### C. Review mode
Replay browsing, move stepping, notation, analysis-oriented viewing.

### D. Menu overlay mode
Temporary match-scoped control panel.

### E. Online flow mode
Lobby, room create/join, invite handling, presence states.

This matters because current screens blur Match, Menu, and Review behaviors together.

---

## 5. Screen strategy

## 5.1 Home screen

### Purpose
A clean launcher that can scale into a richer product shell.

### Recommended content
Primary actions:
- Local Play
- Online Play
- Replays
- Profile

Supporting UI:
- top-right identity/auth card
- optional “continue last match”
- optional recent activity
- small footer links such as Rules / Controls / About

### Design recommendation
Home should be **launcher-first with light hub elements**:
- stronger than a plain menu
- lighter than a full dashboard

### Key behavior
The user should understand the product in one glance:
- play locally
- play online
- browse replays
- sign in / manage profile

---

## 5.2 Match screen

### Purpose
The main gameplay surface.

### Core layout
The board is the center. Match metadata and actions live in a dedicated HUD region.

### Desktop / landscape
Use a **2-column shell**:

- left/main = board
- right = persistent match sidebar

### Portrait / mobile
Use a **stacked shell**:

- top status strip
- board
- bottom action bar

### Match screen should contain
- board
- player cards
- ruleset label
- timer
- turn state
- primary match actions
- contextual panel slot

### Match screen should not contain
- account management
- broad online navigation
- profile editing
- replay library browsing

---

## 5.3 Match menu

### Purpose
A match-scoped control panel.

### Rename current “Settings” button
Use **MENU** instead of **SETTINGS**.

Reason:
the current screen already includes much more than settings:
- rules
- player setup
- resume
- new game

That is a match menu, not a generic settings page.

### Menu sections
- Rules
- Players
- Match actions

Possible actions:
- Resume
- New Game
- Reset
- Resign
- Return Home

### Do not put here
- sign in
- username
- profile settings
- room browser
- replay library

---

## 5.4 Online screen

### Purpose
A dedicated surface for online flows.

### Likely v1 content
- Create Room
- Join Room
- Enter room code
- paste invite link
- recent rooms
- incoming invites
- guest vs signed-in entry paths

### Later additions
- friend invites
- match history
- casual/ranked
- spectating
- recent opponents

---

## 5.5 Replays screen

### Purpose
A dedicated browsing surface for replay artifacts.

### Content
- local saved replays
- imported/shared replay URLs
- replay cards/list rows
- filters or sort later
- open into Review mode

### Important distinction
Replays as a top-level library are different from stepping through a replay inside Review mode.

---

## 5.6 Profile screen

### Purpose
Identity and account management.

### Content
- guest/signed-in state
- username
- linked account
- auth controls
- lightweight preferences
- maybe player stats later

### Keep separate from match UI
Profile is app shell, not gameplay.

---

## 6. Match layout specification

## 6.1 Desktop / landscape shell

Recommended structure:

1. board area
2. right sidebar HUD

### Proportions
- board area: approx. 70–78%
- sidebar: approx. 22–30%

### Sidebar sections
From top to bottom:

1. Match header
2. Game meta
3. Player 1 card
4. Player 2 card
5. Context panel slot
6. Action stack

This gives every new feature a home before the UI becomes cluttered.

---

## 6.2 Portrait / mobile shell

Recommended structure:

1. top status strip
2. board
3. bottom action row

### Overlays
Use:
- full-screen overlays
or
- bottom sheets

Do not attempt to preserve the desktop sidebar composition inside portrait mode.

---

## 7. Context panel slot

This is the most important scalability mechanism in the match shell.

Reserve a dedicated panel area that swaps content by mode.

### Possible contents

#### Live match
- Your turn
- Bot thinking
- Waiting for opponent
- Connected / disconnected state

#### Result
- winner
- win condition
- rematch / new game
- share replay

#### Online room
- room code
- invite link
- opponent connected status
- ready state

#### Review
- move index
- step controls
- notation summary
- export/share

#### Analysis later
- evaluation
- PV
- bot metadata

The point is to make complexity appear in one controlled place rather than spreading across the whole screen.

---

## 8. Board area rules

The board remains visually dominant.

### Allowed board overlays
- last move marker
- move numbers in review mode
- legal move hover
- subtle win-line highlight
- candidate marker / cursor
- lightweight tactical overlays

### Avoid inside board
- profile cards
- auth prompts
- room management UI
- broad settings
- unrelated app navigation

Board UI should answer:
- what is the position
- whose move it is
- what just happened

---

## 9. Action hierarchy

Buttons currently read too similarly. Formalize three action levels.

### Primary
Most important current action.
Examples:
- Play
- Resume
- Join Room
- Create Room
- New Game
- Rematch

### Secondary
Useful but non-destructive.
Examples:
- Menu
- Share Replay
- Copy Room Code
- Open Review
- Open Profile

### Destructive
Negative or risky actions.
Examples:
- Reset
- Resign
- Leave Room
- Delete Replay

Destructive actions should share a family but not overwhelm the interface.

---

## 10. Review mode

Review should become an explicit mode, not just a board with numbered stones.

### Review mode responsibilities
- move stepping
- move numbering
- move list / notation later
- export/share replay
- optional analysis overlays later

### Desktop
Put controls in the context panel slot.

### Mobile
Put step controls into a bottom sheet or compact control strip.

---

## 11. Visual language direction

## 11.1 What is already working

The current style is strong because it is:
- low-color
- silhouette-first
- high-contrast
- tactical rather than decorative
- coherent across stones, pointer, and buttons

This should be preserved.

## 11.2 Style direction

The UI should feel like a **pixel tactical board UI**, not a separate colorful app skin.

The design language should remain close to:
- board/stone material logic
- small accent usage
- bold shapes
- simple highlights and shadows
- readable at small size

### Important caution
Do not let new icons or panels drift into a higher-fidelity “cute app icon” style that feels disconnected from the board/stones/pointer assets.

The UI library should be built at the **same asset fidelity tier** as the core game pieces.

---

## 12. UI library strategy

Think of the system as a **pixel HUD kit with app-shell extensions**, not generic web UI first.

### Layers

#### 1. Core visual tokens
- pixel grid
- render scale
- spacing rhythm
- color roles
- font sizes
- shadow rules
- animation timings

#### 2. Primitive UI pieces
- button
- panel
- icon
- badge
- divider
- toggle
- focus marker
- chip

#### 3. Composite components
- player card
- menu row
- room row
- replay row
- profile card
- result card
- toast/status item

#### 4. Screen patterns
- home launcher
- match sidebar
- match menu
- online room list
- replay browser
- profile screen

---

## 13. Visual tokens

## 13.1 Icon/grid system
- base icon size: **16x16**
- preserve strong silhouette readability
- use same outline/highlight/shadow logic as stones/buttons/pointer
- avoid over-detailed interior pixel clusters

## 13.2 Spacing
Use a simple rhythm:
- 4 px
- 8 px
- 16 px
- 24/32 px where needed for layout grouping

## 13.3 Shadows and depth
Keep depth simple and crisp:
- one primary shadow step
- one inset/raised treatment
- hardware-like motion, not soft web-style elevation

---

## 14. Semantic color roles

Define color by meaning, not by screen.

Suggested roles:

- `bg/base`
- `bg/panel`
- `bg/elevated`
- `text/primary`
- `text/muted`
- `accent/meta`
- `success`
- `warning`
- `danger`
- `focus`
- `online`
- `replay`

### Recommended semantic mapping
- **green** = confirm / ready / your turn / primary action
- **blue** = online / room / network
- **purple** = replay / history / review
- **yellow** = meta / title / focus / warning-lite
- **red** = destructive / danger / reset / resign
- **white/gray/black** = neutral identity / board-adjacent materials

Use accent colors sparingly.

---

## 15. Icon language

Icons should feel like miniature board-adjacent pieces, not colorful app stickers.

### Icon rules
- 16x16 base grid
- max 2–3 colors per icon
- strong silhouette first
- grayscale base + one accent preferred
- no extra decorative shading unless consistent with game assets
- maintain the same “material family” as stones/buttons/pointer

### Suggested icon set
- play
- online / link / room
- replay / history
- profile
- bot
- human
- warning
- timer
- share
- lock
- connected
- disconnected
- trophy
- settings / menu
- guest
- sign-in

---

## 16. Button system

Buttons should remain stone-adjacent in feel, but branch into clear variants.

### Required button families
- Primary button
- Secondary button
- Destructive button
- Toggle button
- Icon button
- List-row action button

### Shared characteristics
- crisp pixel silhouette
- simple highlight/shadow
- readable label
- strong pressed state
- consistent sizing rhythm

---

## 17. Panel system

You will need panel families soon.

### Required panel families
- Sidebar panel
- Modal panel
- Card panel
- List row panel

Each should define:
- border style
- shadow style
- padding rules
- header treatment
- optional section divider

Panels should feel structured and calm so the game elements stay expressive.

---

## 18. Status / HUD language

The uploaded assets suggest the start of a symbolic HUD language. Build on that.

### Useful status elements
- success badge
- warning badge
- danger badge
- ready
- offline
- thinking
- your turn
- connected
- replay
- guest

These should appear as chips, badges, or compact HUD blocks rather than full new panel styles.

---

## 19. Animation language

Animation should add fun without changing fidelity tier.

### Animation style
Use **tiny, tactile, low-frame motion**.

Good examples:
- button depress (1–2 px shift)
- focus bracket flicker
- subtle pulse
- small blink
- warning flash
- online connection beacon
- bot thinking tick
- last-move pop

### Avoid
- soft modern easing-heavy app motion
- over-bouncy transitions
- overly smooth juice that conflicts with the pixel hardware feel

### Rule of thumb
- 2-frame or 4-frame max for most UI motion
- 1–2 px movement max
- blink/pulse over squash/stretch

---

## 20. Core reusable components

These are the first components the implementation should formalize.

### Atoms
- `PixelButton`
- `PixelIcon`
- `StatusBadge`
- `Divider`
- `ToggleGroup`
- `FocusMarker`
- `Panel`

### Molecules
- `PlayerCard`
- `ActionRow`
- `RoomRow`
- `ReplayRow`
- `ProfileChip`
- `MenuSection`

### Organisms
- `MatchSidebar`
- `MatchMenuPanel`
- `OnlineListPanel`
- `ReplayBrowser`
- `ProfilePanel`

---

## 21. Home screen design guidance

### Preferred direction
Launcher-first with light hub elements.

### Strong candidate structure
Center:
- Local Play
- Online Play
- Replays
- Profile

Support:
- top-right auth/identity card
- optional continue-last-match
- bottom secondary links

### Avoid
- making Home a dense dashboard too early
- pushing all online/replay activity directly into the center if that hurts readability

---

## 22. Responsive behavior

## Desktop / tablet landscape
- persistent right sidebar
- board dominant
- context panel in sidebar
- player info always visible

## Portrait mobile
- compact top status
- board centered
- bottom actions
- full-screen overlays for menu/profile/online
- player cards collapse into compact cards or chips

### Rule
Reuse the same components, but allow different layout structures.

---

## 23. Implementation phases

## Phase 1: structure
- define top-level routes or equivalent state:
  - Home
  - Match
  - Online
  - Replays
  - Profile
- rename Settings to Menu in Match flow
- separate Review mode from Match mode conceptually

## Phase 2: match shell
- formalize desktop sidebar
- define context panel slot
- formalize player cards and action stack

## Phase 3: UI primitives
- button variants
- panel variants
- status badges
- icon family
- toggle groups
- list rows

## Phase 4: overlays and app shell
- reusable overlay shell
- Match Menu panel
- Profile panel
- Online room screen
- Replay browser

## Phase 5: polish
- icon refinement
- animation passes
- status states
- visual consistency audit
- mobile-specific layout tuning

---

## 24. Coding-agent rules

1. Preserve board dominance.
2. Do not add random new persistent buttons directly onto the match screen.
3. All new non-board UI must map to one of:
   - sidebar section
   - context panel
   - overlay
   - top-level screen
4. Online/account features do not belong in Match Menu unless directly match-scoped.
5. Build reusable primitives before one-off panels.
6. Keep icon fidelity close to the board/stones/pointer assets.
7. Use semantic colors, not ad hoc per-feature colors.
8. Prefer tactical, hardware-like motion over modern soft app motion.
9. Match and Review should diverge more cleanly over time, not blur together more.
10. Support portrait and landscape through shared components with different layout compositions.

---

## 25. Non-goals

These are not immediate goals for the first UI system pass:

- fully polished production dashboard
- overly detailed icon illustration
- dense profile/social features
- maximal animation
- pixel-art flourishes that reduce clarity
- giant Figma-perfect component taxonomy before the real primitives exist

Focus first on:
- structure
- consistency
- scalability
- coherence with the existing game assets

---

## 26. Final recommendation

The right next move is **not** to abandon the current visual direction.

It is to formalize it.

### In short:
- keep the board/stone/pointer/button language
- build a reusable pixel HUD/app-shell library from that language
- formalize Home / Match / Online / Replays / Profile
- turn Settings into a true Match Menu
- make the right rail a deliberate sidebar system
- keep icon fidelity low and cohesive
- let the game feel like one product, not a game plus a separate app skin
