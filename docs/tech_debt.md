# Tech Debt & Deferred Refactors

Items flagged during code review but deferred as too invasive for a cleanup pass.
Pick these up when touching the relevant code for other reasons.

---

## `gomoku-web`

### IdleCycle unification — `game.ts`

`scheduleNextPointerAnim` and `scheduleNextStoneAnim` are structurally identical:
pick a random item from a list, play it on a sprite, wait for `ANIMATION_COMPLETE`,
optionally reset to a static frame, recurse via `delayedCall`. Only the sprite ref,
anim list, delay range, and post-complete frame reset differ.

**Fix:** extract a generic `IdleCycle` class:

```typescript
class IdleCycle {
  start(sprite, anims, delayRange, onComplete?)
  stop()
}
```

This would collapse `pointerIdleTimer`, `stoneIdleTimer`, `lastStoneSprite`, and all
four schedule/cancel/start/stop methods into two `IdleCycle` instances.

---

### `NameEditor` extraction — `ui.ts` `SettingsPanel`

Six fields (`editingPlayer`, `inputBuffer`, `cursorOn`, `cursorTimer`,
`keydownHandler`, `pointerupHandler`) plus `startEditing` / `stopEditing` /
`updateEditLabel` / `onKeydown` form a self-contained inline state machine that
has grown to ~60 lines inside `SettingsPanel`.

**Fix:** extract a `NameEditor` class with a simple `start(initial, onDone)` /
`stop(confirm)` API. Cursor blink, keyboard listener, and pointer-away listener
become private implementation details, impossible to leak.

---

### `ToggleGroup` parameter order — `ui.ts`

`vertical: boolean = false` sits between `width` and `onSelectedClick?`, so any
caller that only needs `onSelectedClick` must pass an explicit `false`:

```typescript
new ToggleGroup(scene, 0, 0, options, selected, scale, width, false, callback)
//                                                               ^^^^^ noise
```

**Fix:** move `vertical` after `onSelectedClick`, or swap the two optionals so
the callback comes first. Low risk change but touches all three `ToggleGroup`
call sites in `SettingsPanel`.

---

### Per-second guard in `update()` — `game.ts`

`formatTime` is called 3–4 times per frame, and `showPendingTimer` reads
`BitmapText.width` (triggers bounds recalculation) every frame to reposition
the delta text — even though the displayed strings only change once per second.

**Fix:** add a `lastDisplaySec` field; gate all timer text updates behind
`Math.floor(Date.now() / 1000) !== this.lastDisplaySec`. Saves ~4 string
allocations and 1–2 BitmapText bounds recalculations per frame while a game
is running.

---

### `stopEditing` listener cleanup duplication — `ui.ts` `SettingsPanel`

The null-check + `.off()` + null-assign pattern is repeated for `keydownHandler`
and `pointerupHandler` in `stopEditing`:

```typescript
if (this.keydownHandler)  { scene.input.keyboard!.off(...); this.keydownHandler = null; }
if (this.pointerupHandler){ scene.input.off(...);           this.pointerupHandler = null; }
```

A third handler would require a third copy. **Fix:** a small inline helper
`disposeHandler(target, event, field)`, or absorb into the `NameEditor` class
above (which makes this go away entirely).
