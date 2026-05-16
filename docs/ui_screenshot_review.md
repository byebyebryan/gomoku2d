# UI Screenshot Review

This document is the screenshot-based companion to `ui_design.md`.

Use it when reviewing mocks, UI tweaks, or layout passes against the actual
evolution of the DOM shell. `ui_design.md` is the rulebook; this file is
the evidence and critique set.

## v0.1

### References

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_1_match_desktop.png" alt="v0.1 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_1_settings_desktop.png" alt="v0.1 settings screen on desktop" width="100%">
      <br>
      <sub>Settings / Desktop</sub>
    </td>
  </tr>
</table>

### What v0.1 got right

- strong retro tone immediately
- clear board dominance
- chunky, high-contrast controls
- UI feels game-like before it feels app-like

### Where v0.1 broke down

- too much of the experience lived inside one game scene
- gameplay, settings, and shell concerns blurred together
- UI language was expressive, but not scalable

The lesson is not "go back to v0.1." The lesson is to preserve its punch
without rebuilding the whole app as one canvas-driven surface.

## v0.2.1

### References

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_1_home_desktop.png" alt="v0.2.1 home screen on desktop" width="100%">
      <br>
      <sub>Home / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_1_match_desktop.png" alt="v0.2.1 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_1_replay_desktop.png" alt="v0.2.1 replay screen on desktop" width="100%">
      <br>
      <sub>Replay / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_1_profile_desktop.png" alt="v0.2.1 profile screen on desktop" width="100%">
      <br>
      <sub>Profile / Desktop</sub>
    </td>
  </tr>
</table>

### What v0.2.1 improved

- proper DOM-shell structure
- clearer screen separation between home, match, replay, and profile
- stronger foundation for local profile and replay features
- more scalable spacing, scrolling, and panel ownership

These references are useful because they show the shell under practical
density, not just idealized empty states.

### Where v0.2.1 still feels weak

- the shell can still feel too app-like or too muted
- some screens still carry more chrome than they need
- retro charm is less immediate than in v0.1
- hierarchy is better than before, but not fully confident under dense states

## v0.2.2

### References

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_2_home_desktop.png" alt="v0.2.2 home screen on desktop" width="100%">
      <br>
      <sub>Home / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_2_match_desktop.png" alt="v0.2.2 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_2_replay_desktop.png" alt="v0.2.2 replay screen on desktop" width="100%">
      <br>
      <sub>Replay / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_2_profile_desktop.png" alt="v0.2.2 profile screen on desktop" width="100%">
      <br>
      <sub>Profile / Desktop</sub>
    </td>
  </tr>
</table>

### What v0.2.2 improved

- stronger palette contrast and clearer button roles
- flatter shell with fewer unnecessary boxes and panel frames
- clearer live-match and replay HUD language
- stronger record-screen treatment on profile

These references are useful as the baseline for the next refinement pass, not
as the end of the visual pass.

### Where v0.2.2 still needs work

- portrait/mobile layouts still feel like desktop screens collapsing downward
- the shell is still heavily text-driven in places where compact controls would
  help
- some controls and metadata blocks still depend on long labels rather than a
  denser visual language
- the mobile version likely needs screen-specific layouts, not just more
  breakpoint tuning

## v0.2.3

### References

#### Desktop

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_3_home_desktop.png" alt="v0.2.3 home screen on desktop" width="100%">
      <br>
      <sub>Home / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_3_match_desktop.png" alt="v0.2.3 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_3_replay_desktop.png" alt="v0.2.3 replay screen on desktop" width="100%">
      <br>
      <sub>Replay / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_3_profile_desktop.png" alt="v0.2.3 profile screen on desktop" width="100%">
      <br>
      <sub>Profile / Desktop</sub>
    </td>
  </tr>
</table>

#### Mobile

<table>
  <tr>
    <td width="25%">
      <img src="assets/screenshot_v0_2_3_home_mobile.png" alt="v0.2.3 home screen on mobile" width="100%">
      <br>
      <sub>Home / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_3_match_mobile.png" alt="v0.2.3 match screen on mobile" width="100%">
      <br>
      <sub>Match / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_3_replay_mobile.png" alt="v0.2.3 replay screen on mobile" width="100%">
      <br>
      <sub>Replay / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_3_profile_mobile.png" alt="v0.2.3 profile screen on mobile" width="100%">
      <br>
      <sub>Profile / Mobile</sub>
    </td>
  </tr>
</table>

### What v0.2.3 improved

- match, replay, and profile now have intentional portrait layouts instead of
  reading like desktop screens collapsing downward
- the shell has a usable icon language for desktop actions and replay transport
- replay controls are denser and clearer without taking focus away from the
  board
- local match on portrait has a dedicated touch-placement flow instead of
  relying on direct tap-to-place

These references are useful because they show the shell as a paired
desktop/mobile system rather than a desktop-first shell with a narrow fallback.

### Where v0.2.3 still feels uneven

- profile is still the busiest screen, especially around the settings block and
  reset action
- some secondary controls still pull a little more visual weight than they need
- the icon language is helpful now, but it still needs restraint so it does not
  become a separate app skin

### v0.2.4 polish direction

These notes became the `v0.2.4` polish pass. The work stayed deliberately
narrow and on top of the same shell:

- labels are quieter and secondary metadata competes less with values
- desktop Match and Replay rails are tighter and read more like HUD/transport
  surfaces than utility sidebars
- desktop Profile puts more emphasis on the record summary and less on the
  identity/settings rail
- replay transport removes one more redundant subtitle (`Replay timeline`) to
  keep the playback zone compact
- the icon system stays monochrome and narrow in scope, but now intentionally
  renders at a uniform `24px` because the authored pack reads best at its
  native scale

The important constraint stayed the same: `v0.2.4` polished the current shell,
not the mobile layout or control model.

## v0.2.4

### References

#### Desktop

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_4_home_desktop.png" alt="v0.2.4 home screen on desktop" width="100%">
      <br>
      <sub>Home / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_4_match_desktop.png" alt="v0.2.4 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_2_4_replay_desktop.png" alt="v0.2.4 replay screen on desktop" width="100%">
      <br>
      <sub>Replay / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_2_4_profile_desktop.png" alt="v0.2.4 profile screen on desktop" width="100%">
      <br>
      <sub>Profile / Desktop</sub>
    </td>
  </tr>
</table>

#### Mobile

<table>
  <tr>
    <td width="25%">
      <img src="assets/screenshot_v0_2_4_home_mobile.png" alt="v0.2.4 home screen on mobile" width="100%">
      <br>
      <sub>Home / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_4_match_mobile.png" alt="v0.2.4 match screen on mobile" width="100%">
      <br>
      <sub>Match / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_4_replay_mobile.png" alt="v0.2.4 replay screen on mobile" width="100%">
      <br>
      <sub>Replay / Mobile</sub>
    </td>
    <td width="25%">
      <img src="assets/screenshot_v0_2_4_profile_mobile.png" alt="v0.2.4 profile screen on mobile" width="100%">
      <br>
      <sub>Profile / Mobile</sub>
    </td>
  </tr>
</table>

### What v0.2.4 improved

- the app now exposes the release version on Home without adding another
  persistent metadata block
- the final icon scale is more legible and consistent across the shell
- Profile is denser and reads more like a local record page than a settings
  form
- the canvas warning language and sequence-number rendering are cleaner after
  the animation and font passes
- the README hero and social image now reflect the current `v0.2.4` look

### Remaining watch points

- mobile Profile is still the densest screen, so future additions should be
  conservative
- small pixel labels can still get hard to read at mobile scale; avoid adding
  more low-priority microcopy there
- the shell should stay effectively frozen for the rest of `0.2.x` unless a
  real bug or release-blocking polish issue appears

## v0.4.5

### References

These captures use the `v0.4.5` web package version and a seeded local replay.
Mobile references are viewport captures at `390 x 844`, not full-page scroll
captures, so they show the same review frame across screens.

#### Desktop

<table>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_4_5_home_desktop.png" alt="v0.4.5 home screen on desktop" width="100%">
      <br>
      <sub>Home / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_4_5_match_desktop.png" alt="v0.4.5 match screen on desktop" width="100%">
      <br>
      <sub>Match / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <img src="assets/screenshot_v0_4_5_replay_desktop.png" alt="v0.4.5 replay screen on desktop" width="100%">
      <br>
      <sub>Replay / Desktop</sub>
    </td>
    <td width="50%">
      <img src="assets/screenshot_v0_4_5_profile_desktop.png" alt="v0.4.5 profile screen on desktop" width="100%">
      <br>
      <sub>Profile / Desktop</sub>
    </td>
  </tr>
  <tr>
    <td colspan="2">
      <img src="assets/screenshot_v0_4_5_settings_desktop.png" alt="v0.4.5 settings screen on desktop" width="100%">
      <br>
      <sub>Settings / Desktop</sub>
    </td>
  </tr>
</table>

#### Mobile

<table>
  <tr>
    <td width="20%">
      <img src="assets/screenshot_v0_4_5_home_mobile.png" alt="v0.4.5 home screen on mobile" width="100%">
      <br>
      <sub>Home / Mobile</sub>
    </td>
    <td width="20%">
      <img src="assets/screenshot_v0_4_5_match_mobile.png" alt="v0.4.5 match screen on mobile" width="100%">
      <br>
      <sub>Match / Mobile</sub>
    </td>
    <td width="20%">
      <img src="assets/screenshot_v0_4_5_settings_mobile.png" alt="v0.4.5 settings screen on mobile" width="100%">
      <br>
      <sub>Settings / Mobile</sub>
    </td>
    <td width="20%">
      <img src="assets/screenshot_v0_4_5_replay_mobile.png" alt="v0.4.5 replay screen on mobile" width="100%">
      <br>
      <sub>Replay / Mobile</sub>
    </td>
    <td width="20%">
      <img src="assets/screenshot_v0_4_5_profile_mobile.png" alt="v0.4.5 profile screen on mobile" width="100%">
      <br>
      <sub>Profile / Mobile</sub>
    </td>
  </tr>
</table>

### What v0.4.5 improved

- Settings is now a first-class route instead of Profile carrying rule and bot
  configuration.
- The app has an explicit bot-control surface: tested presets, advanced depth /
  width / scoring / proof controls, and compact generated config summaries.
- Human tactical hints are now configurable in compact immediate/imminent rows
  without making Renju forbidden-move legality optional.
- Home keeps the release version and static links compact while adding routes
  to the lab artifacts.
- Match and replay now share the newer navigation order and retain board-first
  weight after the settings split.

### Remaining watch points

- Mobile Settings is intentionally scroll-heavy. The first viewport reads
  cleanly, but later advanced controls should stay compact and avoid adding
  explanatory copy inside the form.
- The Custom bot path is visible and useful, but it is also the densest part of
  the product UI. Keep future bot-lab knobs behind the same controlled layer
  rather than spreading them into Match or Profile.
- Profile is cleaner after losing rule/bot settings, but local/cloud identity,
  reset, and history still make it the most sensitive screen for future
  additions.

## Design takeaway

- keep v0.1's retro punch and board-first confidence
- keep v0.2.1's structure, separation, and scalability
- keep v0.2.2's flatter shell and clearer button-role language
- keep v0.2.3's intentional mobile layouts and tighter transport language
- keep v0.2.4's tighter icon scale, profile density, and documented asset set
- keep v0.4.5's dedicated settings route and controlled bot-lab layer
- avoid reintroducing v0.1's scene-bound UI
- avoid rebuilding dense sidebars or over-explained controls as the shell grows
