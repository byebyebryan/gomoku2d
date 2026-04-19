# Gomoku2D UI Implementation Backlog
**Date:** 2026-04-18  
**Purpose:** execution-oriented UI backlog for coding agents  
**Scope:** concrete milestones, component priorities, and implementation sequencing

---

## 1. Goal

This document translates the higher-level UI/UX design direction into an implementation backlog.

The purpose is to help coding agents:
- make progress in the right order
- avoid reactive UI sprawl
- build reusable primitives before one-off screens
- support upcoming online, replay, and profile features cleanly

This backlog assumes the project direction is:

- board-first match UI
- app shell with top-level screens
- reusable pixel UI library
- scalable support for online, replay, and profile features

---

## 2. Priority order

Build in this order:

1. **UI shell structure**
2. **Core reusable primitives**
3. **Match shell**
4. **Match menu**
5. **Home screen**
6. **Review/replay surfaces**
7. **Online screen**
8. **Profile screen**
9. **Polish and consistency pass**

Do not start with the full Online screen or Profile screen before the basic shell and primitives exist.

---

## 3. Milestone 0: audit and prep

### Objective
Understand current scene/state structure and identify where UI is currently ad hoc.

### Tasks
- inventory all current screens, overlays, and permanent buttons
- inventory current button variants and ad hoc styles
- inventory board overlay behaviors
- identify where state is local to scene vs global/app-level
- identify current reusable code vs duplicated UI logic

### Deliverables
- short implementation note or issue list
- current UI map
- list of immediate refactor targets

### Acceptance criteria
- current match UI is documented
- major ad hoc UI areas are identified
- app-level vs match-level state boundaries are clear enough to refactor

---

## 4. Milestone 1: app shell structure

### Objective
Introduce top-level app structure so future features stop invading match UI.

### Required top-level surfaces
- Home
- Match
- Online
- Replays
- Profile

### Tasks
- define top-level route/state model
- define navigation ownership
- decide how Phaser scenes map to app-level screens and overlays
- ensure Match is not the only root context
- add placeholders for screens not yet fully implemented

### Deliverables
- app-level navigation state
- navigation transitions between top-level screens
- placeholder stubs for Online / Replays / Profile if needed

### Acceptance criteria
- app has a notion of top-level screen/mode
- future features have a home outside Match
- Match can be entered/exited cleanly

---

## 5. Milestone 2: UI foundation primitives

### Objective
Create the minimum reusable UI library needed before building more screens.

### Required primitives

#### Buttons
- PrimaryButton
- SecondaryButton
- DestructiveButton
- ToggleButton
- IconButton

#### Layout containers
- Panel
- Card
- Divider
- SectionHeader

#### HUD/status
- StatusBadge
- FocusMarker
- TimerLabel
- MetaLabel

#### Selection
- ToggleGroup
- ChoiceRow

### Tasks
- unify button sizing and spacing
- define a small variant system
- define panel border/shadow treatment
- define shared text styles
- define status badge styles
- define focus/selection treatment

### Deliverables
- reusable components or equivalent prefabs
- style constants/tokens
- component usage examples

### Acceptance criteria
- no new screen should need to invent a new button style
- panel and card treatments are reusable
- status indicators look like part of the same family

---

## 6. Milestone 3: icon and visual token pass

### Objective
Create a restrained UI icon set and visual token system that matches current game assets.

### Tasks
- define semantic color tokens
- define spacing tokens
- define icon grid rules
- define initial icon set
- reduce overly illustrative/high-fidelity icon ideas
- align icons to stone/button/pointer material language

### Required initial icons
- play
- online
- replay
- profile
- human
- bot
- timer
- warning
- share
- lock
- connected
- disconnected
- trophy
- menu
- guest
- sign-in

### Deliverables
- icon sheet
- token definitions
- icon usage guidance

### Acceptance criteria
- icons feel like the same asset family as board/stones/pointer/buttons
- icons do not look like a separate app skin
- color usage is semantic and restrained

---

## 7. Milestone 4: match shell refactor

### Objective
Turn the current gameplay screen into a scalable shell.

### Desktop/landscape target
- board on left
- persistent sidebar on right

### Portrait/mobile target
- top status strip
- board center
- bottom action row

### Tasks
- formalize board area vs HUD area
- implement right sidebar structure for landscape
- create top status + bottom action pattern for portrait
- preserve board dominance
- move scattered actions into structured locations

### Required sidebar sections
- match header
- game meta
- player 1 card
- player 2 card
- context panel slot
- action stack

### Deliverables
- new match shell layout
- responsive behavior rules
- cleaned action placement

### Acceptance criteria
- board remains dominant
- match UI feels structured, not accreted
- future match-related features can land in sidebar/context slot cleanly

---

## 8. Milestone 5: player card component

### Objective
Create one reusable player card that can support local, bot, and online states.

### Required fields
- name
- role/type
- side/color
- wins
- timer
- active/turn state
- connection/thinking/ready state
- optional rating/avatar later

### Tasks
- design compact player card
- support local human state
- support bot state
- support online state placeholders
- define status line handling

### Example statuses
- Your turn
- Thinking...
- Ready
- Connected
- Disconnected
- Reconnecting
- Won +1

### Deliverables
- PlayerCard component
- supported state list
- test render variants

### Acceptance criteria
- same card works for local/bot/online without redesign
- status states are visually clear
- card integrates into sidebar and portrait layouts

---

## 9. Milestone 6: context panel slot

### Objective
Create the swap-in panel region that will absorb feature growth.

### Tasks
- define ContextPanel container
- define mode switching for content
- support at least:
  - live match
  - result
  - review
- leave room for online room status later

### Initial panel variants

#### LiveMatchPanel
- your turn / thinking / waiting
- short helper text
- maybe tiny status icon

#### ResultPanel
- winner
- win condition
- rematch/new game
- share replay later

#### ReviewPanel
- move index
- previous/next
- first/last
- exit review

### Deliverables
- context panel system
- 3 initial content variants

### Acceptance criteria
- result/review/live content does not require new screen layouts
- panel is easy to swap by mode
- complexity is contained in one place

---

## 10. Milestone 7: match menu overlay

### Objective
Replace ad hoc settings with a proper match-scoped menu.

### Rename
- `Settings` -> `Menu`

### Menu sections
- Rules
- Players
- Match Actions

### Tasks
- build overlay shell
- implement menu panel content
- support rules toggle
- support player-type toggle
- support resume/new game
- add resign/reset/home only if needed and clearly separated

### Deliverables
- reusable overlay shell
- MatchMenuPanel
- landscape and portrait behavior

### Acceptance criteria
- menu feels match-scoped
- menu does not become account/profile/online dumping ground
- current settings behavior is preserved or improved

---

## 11. Milestone 8: home screen

### Objective
Build a clean launcher-first home screen with light hub features.

### Core content
- Local Play
- Online Play
- Replays
- Profile

### Support content
- top-right auth/identity card
- continue last match
- optional recent activity
- footer links for Rules / Controls / About

### Tasks
- define home layout
- implement action hierarchy
- implement identity/auth card
- implement lightweight recent/continue card if available

### Deliverables
- Home screen
- navigation into Match / Online / Replays / Profile

### Acceptance criteria
- new users understand main app structure immediately
- home does not feel cluttered
- future online/replay/profile growth is supported

---

## 12. Milestone 9: review mode and replay surfaces

### Objective
Separate Review from Match and build replay-facing UI.

### Tasks
- define explicit Review mode
- add move stepping controls
- support move numbers
- support entering Review from replay entry points
- support leaving Review back to Match/Home as appropriate

### Replay library tasks
- build replay row/card
- build replay list screen
- support loading a replay into Review

### Deliverables
- Review mode
- Replay browser/list
- replay list items

### Acceptance criteria
- replay functionality is no longer just a special-case board overlay
- review actions live in their own mode and controls
- replay browsing has its own top-level place

---

## 13. Milestone 10: online screen v1

### Objective
Create the first dedicated online surface.

### v1 content
- Create Room
- Join Room
- Paste invite link / enter room code
- recent rooms
- invite rows
- guest vs signed-in entry points

### Tasks
- build Online screen layout
- build room row/list item
- build invite row/list item
- wire to backend placeholders or actual flows as available
- add auth-aware empty states

### Deliverables
- Online screen
- room/invite list row components
- create/join interaction entry points

### Acceptance criteria
- online flows do not require entering Match Menu first
- screen feels purpose-built, not borrowed from settings
- online state has a clear future home

---

## 14. Milestone 11: profile screen v1

### Objective
Create a separate profile/account surface.

### v1 content
- guest/signed-in state
- sign in / sign out
- username
- linked account or provider
- small preferences placeholder

### Tasks
- build Profile screen
- build ProfileCard or profile header
- support guest state
- support signed-in state
- support auth entry actions

### Deliverables
- Profile screen
- profile identity component

### Acceptance criteria
- account/auth does not live in Match Menu
- profile state is visible and understandable
- future preferences can land here cleanly

---

## 15. Milestone 12: responsive pass

### Objective
Make sure the system works coherently in portrait and landscape.

### Tasks
- audit all screens in desktop/landscape
- audit all screens in portrait/mobile
- adapt layout composition without forking style language
- ensure overlays become full-screen or bottom-sheet on small screens
- ensure player cards collapse cleanly

### Deliverables
- responsive rules
- per-screen layout adjustments

### Acceptance criteria
- portrait is not just a squeezed desktop layout
- same components work across device classes
- information density stays readable

---

## 16. Milestone 13: animation and feedback pass

### Objective
Add tactile motion without breaking the pixel identity.

### Tasks
- button press state
- focus/selection animation
- your-turn indicator
- bot-thinking indicator
- warning flash
- connected/disconnected pulse
- result pop or state emphasis

### Rules
- 2-frame or 4-frame max preferred
- 1–2 px movement max
- tactical/hardware feel, not juicy mobile-app feel

### Deliverables
- small animation system
- per-component motion guidelines

### Acceptance criteria
- UI feels more alive
- motion remains coherent with low-fidelity pixel style
- animations improve clarity, not distract from it

---

## 17. Milestone 14: consistency audit

### Objective
Clean up style drift and one-off UI decisions.

### Audit checklist
- does every button use an approved family?
- do icons match the game asset fidelity?
- do panels share common border/shadow rules?
- are semantic colors used consistently?
- are destructive actions visually distinct but not overused?
- are board overlays still restrained?
- are match-level and app-level actions clearly separated?

### Deliverables
- cleanup pass
- issue list for remaining inconsistencies

### Acceptance criteria
- the product feels like one coherent system
- fewer one-off styles remain
- future additions have a clear visual home

---

## 18. Suggested component backlog

### Highest priority
- Panel
- PixelButton variants
- PlayerCard
- StatusBadge
- MatchSidebar
- MatchMenuPanel
- OverlayShell

### Medium priority
- RoomRow
- ReplayRow
- ProfileChip
- TopStatusBar
- BottomActionBar
- ContextPanel variants

### Lower priority / later
- Toast
- Tooltip/help hint
- richer replay controls
- profile stats card
- leaderboard rows

---

## 19. Suggested file/module organization

This is conceptual; exact names may vary.

- `ui/tokens/*`
- `ui/icons/*`
- `ui/primitives/*`
- `ui/components/*`
- `ui/layouts/*`
- `screens/Home/*`
- `screens/Match/*`
- `screens/Online/*`
- `screens/Replays/*`
- `screens/Profile/*`

At minimum, keep:
- tokens
- primitives
- composite components
- screen implementations
separate enough that reuse is obvious.

---

## 20. Coding-agent constraints

1. Do not solve a screen problem by inventing a brand new visual style.
2. Reuse or extend primitives before creating screen-local custom widgets.
3. If a new feature is not board-related, decide whether it belongs in:
   - Home
   - Online
   - Replays
   - Profile
   - Match context panel
   - Match menu
4. Prefer semantic color usage over feature-specific random color choices.
5. Keep icon detail low and silhouette readability high.
6. Preserve board dominance in Match.
7. Avoid moving account/profile logic into Match just because it is convenient.
8. Favor incremental system-building over wholesale redesigns.

---

## 21. Definition of done for the UI foundation

The UI foundation can be considered “established” when:

- Home / Match / Online / Replays / Profile all exist as real app surfaces
- Match has a formal shell with sidebar/context panel
- Match Menu replaces ad hoc settings
- PlayerCard exists and supports local/bot/online states
- a reusable button/panel/badge/icon system exists
- Review mode is distinct from Match mode
- major one-off UI styling has been reduced
- online and profile features have a scalable place to live

---

## 22. Final recommendation

The best path is:

- build structure first
- then build primitives
- then refactor Match
- then expand into Home / Replays / Online / Profile
- then polish motion and visual consistency

The goal is not just nicer screens.

The goal is a UI system that can survive:
- online play
- auth/profile
- replay sharing
- match history
- future bot/review tooling
without collapsing into feature sprawl.
