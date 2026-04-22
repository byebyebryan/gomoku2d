# Gomoku2D Visual Style Brief

## Goal

Define a stable visual language for Gomoku2D while keeping layout and UX flexible for future feature work.

This brief is intentionally focused on styling and component language, not final screen structure.

---

## Creative direction

Gomoku2D should feel like:

- a minimalist retro board game
- a dark, quiet arcade/PC game shell
- chunky and tactile, not sleek
- sparse and atmospheric, not dashboard-like
- board-first, with UI acting as a HUD or cabinet shell

Keywords:

- retro
- restrained
- tactile
- pixel
- quiet
- high-contrast
- board-centric

Avoid:

- SaaS/dashboard vibes
- settings-app vibes
- dense admin-style panels
- too many cards and nested boxes
- verbose, modern product copy

---

## Visual identity pillars

### 1. Board is sacred

On game and replay screens, the board must remain the dominant visual element.

### 2. Fewer boxes

Prefer spacing, dividers, and typography over nested bordered panels.

### 3. Chunky controls

Buttons and toggles should feel physical and game-like.

### 4. Sparse chrome

UI should feel intentionally under-filled rather than fully packed.

### 5. Strong semantic colors

Each accent color should have a narrow and consistent role.

---

## Palette

### Base colors

- `--color-bg: #1E1E1E;`
- `--color-panel: #2A2A2A;`
- `--color-accent: #FCCB57;`
- `--color-text: #F5F5F5;`
- `--color-action: #30A860;`
- `--color-danger: #E04747;`
- `--color-secondary: #5F8B82;`

### Usage rules

- background: `--color-bg`
- large framed surfaces / side rails / panel fills: `--color-panel`
- titles / emphasis / result / active status: `--color-accent`
- normal text: `--color-text`
- primary action / selected positive state: `--color-action`
- destructive / reset: `--color-danger`
- secondary nav / replay / profile actions: `--color-secondary`

### Restrictions

- do not use secondary teal for primary CTAs
- do not overuse accent yellow on large surfaces
- do not introduce extra accent colors unless necessary
- keep the palette feeling limited and iconic

---

## Typography

## Font direction

- pixel / bitmap-style display font
- strong uppercase usage
- minimal prose
- labels should feel like game labels, not app form labels

## Text hierarchy

Keep to 4 tiers max:

- display / page title
- status / section title
- body / label
- meta / helper

## Rules

- use uppercase for buttons, labels, status, and navigation
- keep copy short and signal-heavy
- reduce explanatory sentences
- technical identifiers should be visually quiet
- timestamps and metadata should be lower emphasis than results/status

---

## Spacing system

Use a small consistent spacing scale, something like:

- `4px`
- `8px`
- `12px`
- `16px`
- `24px`
- `32px`

Rules:

- repeated tight UI uses small steps
- major layout separation uses larger steps
- prefer generous outer spacing and tighter inner spacing
- do not solve hierarchy problems by adding more boxes

---

## Borders and depth

## Borders

- use crisp 1px or 2px pixel-feel borders
- reserve stronger borders for top-level panels and buttons
- inner grouping should often use spacing or thin dividers instead of full boxes

## Depth

Use simple pixel-style shadow steps, not soft modern shadows.

Direction:

- hard offset shadow
- no blur or very minimal blur
- buttons should feel more tactile than panels
- pressed states should visibly reduce shadow/depth

---

## Buttons

## General

Buttons should feel chunky, arcade-like, and highly legible.

### Primary button

- green fill
- white text
- strong depth
- obvious hover/pressed state

### Secondary button

- teal fill
- white text
- used for navigation / record / replay

### Neutral button

- dark panel fill or muted gray
- white text
- used for inactive toggles / tertiary actions

### Danger button

- red fill
- white text
- used only for reset / destructive actions

## Rules

- keep labels short
- avoid overly wide sentence-style button text
- buttons should stand out more than surrounding panels
- button state change should be obvious at a glance

---

## Toggles and segmented controls

Use chunky segmented controls for:

- rules mode
- small mutually-exclusive choices
- filter-like game options

Rules:

- active state should be immediately obvious by color and depth
- inactive state should recede into panel/neutral styling
- keep toggles short and binary where possible

---

## Panels and containers

## Panel philosophy

Panels should frame major regions, not every piece of content.

Rules:

- use fewer, larger panels
- avoid nesting framed panel inside framed panel inside framed panel
- within a panel, prefer text groups + dividers over additional boxes
- let some elements sit directly on the page background

## Good panel usage

- board frame
- sidebar rail
- title card
- large modal / result card

## Bad panel usage

- every stat row boxed individually
- every subsection in its own thick rectangle
- card-inside-card structure for simple text groups

---

## HUD styling

For in-game HUD elements:

- sparse
- compact
- high signal
- board-adjacent, not form-like

Rules:

- current turn / result should be the loudest HUD element
- supporting details should be quieter
- do not overload the live match screen with record-keeping UI
- keep live match UI focused on immediate play state

---

## Lists and record views

For replay lists, match history, stat lines:

- use cleaner ledger-like rows
- lighter separators
- less card repetition
- tighter, more tabular rhythm

Rules:

- rows should feel like game records, not inbox cards
- reserve strong highlight treatment for selected/current item only
- avoid giving every row equal visual weight

---

## Motion / interaction feel

If adding animation or transitions:

- keep them short
- keep them pixel-like and snappy
- avoid floaty/mobile-app easing

Good candidates:

- button press depth shift
- blinking cursor / active marker
- subtle state flash for turn/result
- tiny board-adjacent emphasis animations

Avoid:

- smooth card sliding everywhere
- soft fades on all elements
- polished product-style motion systems

---

## Texture and polish

Optional, subtle:

- faint background grain
- faint scanline texture
- tiny shadow stepping
- slight controlled roughness

Rules:

- keep effects extremely restrained
- board readability always comes first
- do not turn the UI into a CRT gimmick

---

## Screen-level invariants

These should remain true even if layouts change:

- Home feels like a title screen
- Match feels like an arcade HUD
- Replay feels like a transport / review screen
- Profile feels like a save record, not account settings
- Board is dominant wherever gameplay is present
- Short labels beat explanatory paragraphs
- Fewer containers beat more containers

---

## Anti-patterns

Avoid these:

- dashboard density
- too many section headers
- large empty side panels with heavy framing
- multiple equal-priority boxes competing on screen
- form-like settings presentation
- blue/teal becoming the dominant action color
- overly polished modern UI effects
- long paragraph copy in the main game shell

---

## Implementation priority

### Phase 1

- establish CSS variables / tokens
- restyle buttons, toggles, panels, typography
- reduce nested borders and inner boxes

### Phase 2

- restyle in-game HUD around sparse board-first principles
- simplify record/replay rows into cleaner ledger styling

### Phase 3

- add small retro polish touches
- refine hover/pressed states
- optional subtle background texture
