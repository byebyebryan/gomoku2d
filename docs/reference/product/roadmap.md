# Roadmap

This roadmap owns release sequencing and product scope. Current formats and
behavior live in the reference contracts; completed work lives in
[`release_history.md`](../../archive/release_history.md).

## Current State: `v0.6.3` Published; Acceptance Follow-up Open

`v0.6.3` was published on October 27, 2026 after `v0.6.2`. It establishes the
static developer site with Visual design, UI design and Bot lab, source-based
asset inspectors, qualified UI history and prepared design reviews. Narrow
Analysis layout and profile Auth-readiness fixes are included. Native code,
backend schemas/rules, portable replay format and recorded Lab data are unchanged.

The real signed-in profile/statistics/game-publication round trip remains
pending. Physical-device/touch, human accessibility, two-device private-account
and novice-feedback checks retain their recorded open or deferred status.
The [release record](../../archive/v0_6_3_release.md) owns these boundaries
and the exact-source CI, deployment and production evidence.

## `0.6` Sequence

Bryan revised the sequence on October 24: finish 0.6.2, establish the developer
site and practical UI review tools in 0.6.3, then replay collections in 0.6.4.
Curated learning follows later, unnumbered for now.
`0.6.2` and `0.6.3` are published; remaining human checks stay explicit.
`0.6.4` remains planned. No future release dates are committed.

| Release | Focus | Player outcome |
|---|---|---|
| `0.6.2` | Public-profile design, sharing usability, documentation and presentation | Understand a player's page, open a game, and understand saving, sharing and publishing; demonstrate the app without signing in |
| `0.6.3` | Developer site, design/assets and UI review tools | Inspect the current design system, compare assets/screens at matched scale and follow real review findings |
| `0.6.4` | Replay collection and study tools | Recognize saved games visually, give them private context, and reopen a deliberately chosen moment |
| Later | Curated learning through replays | Read a short tactical prompt, inspect the example, and try another line against the existing bot |

The [follow-up plan](../../working/v0_6_followup_plan.md) records the review,
first-party research, implementation implications, decision gates and release
acceptance. The [release record](../../archive/v0_6_2_release.md) owns
0.6.2 publication; the [candidate checkpoint](../../working/v0_6_2_candidate.md)
retains earlier preparation evidence and preview context.
Guide help, two validated bot-game examples (28/36 moves, opening at final/26),
inline owner consent and focused README/profile/replay media are implemented.
Earlier checkpoints are retained in the
[implementation archive](../../archive/v0_6_implementation.md).

### `0.6.2`: Design, Usability and Presentation

- Refine public-profile hierarchy: identity, optional statistics, then games.
  Keep sparse profiles intentional and game lists easy to scan.
- Bring owner controls, My games and Saved replays into a consistent visual
  language. Preserve the accepted board-first replay and full-sidebar sharing
  patterns; this is a focused pass rather than a replacement app shell.
- Clarify Share replay, Save replay and Publish replay through placement,
  concise copy and feedback. Preserve public-list position when returning from
  a replay and keep Previous/Next controls in stable positions.
- Review loading, signed-out, empty, unavailable, offline and error states,
  keyboard/focus behavior, enlarged text and actual touch use.
- Add concise player help and two static, account-free example
  replays. Keep tactical teaching in Guide and make the new help easy to find.
- Refresh release-facing media and reconcile current documentation with the
  shipped product. Archive completed planning material with links preserved.

Compare compact thumbnail layouts during design, but ship the reusable
thumbnail implementation in `0.6.4`. `0.6.2` does not require a sharing-data or
portable-format change. Human visual approval precedes implementation of the
selected layout; refreshed media follows the accepted UI.

### `0.6.3`: Developer Site and UI Review

- Establish prepared human presentation alongside detailed agent/maintainer
  Markdown. Reconcile current facts, duplication and reference/archive status.
- Expand the design/asset reference with current tokens/components/states,
  sprites and a full icon inspector with native sizes and enlarged pixel grids.
- Organize existing UI comparison reports into a reusable review gallery with
  source/capture conditions, findings and acceptance status.
- Show the useful parts of the process through tools already used for the work:
  source inspectors, recorded experiments, screen references, matched-scale UI
  comparisons and the maintained screenshot checklist. Record findings and
  follow-up alongside evidence; retrospective journey stories remain later work.
- Derive current references from canonical sources where practical and retain
  historical reviews as dated snapshots. Reuse static hosting without a backend.

The [developer-site plan](../../working/v0_6_3_developer_site_plan.md) owns the
accepted context and proposed page/source/implementation gates. Bryan selected
expansion of the existing site/domain/repository and refinement of its established
look, using React/Vite. The shared developer layout, public/private boundary,
UI history and review tools are implemented and reviewed. The
[release record](../../archive/v0_6_3_release.md) owns the completed publication
gate; the [release plan](../../working/v0_6_3_release_plan.md) retains its
preparation snapshot.

### `0.6.4`: Replay Collection and Study Tools

- Add lightweight, static board thumbnails to public and private replay lists,
  derived from the existing replay payload and chosen opening move.
- Add a private title, one short private note and one explicitly saved opening
  move per saved replay. Keep these separate from game identity and player
  attribution. Scrubbing alone does not save or sync anything.
- Let owners explicitly edit an existing publication's title and opening move
  without removing and republishing it. Public titles remain independent of
  private notes and titles.
- Preserve guest/account ownership, explicit saves, offline recovery and
  reviewable conflicts. Keep the existing 32-publication and 64-saved-replay
  bounds unless a separately reviewed need changes them.

Before writing metadata, settle bounded fields, local/cloud schema evolution,
older-client behavior, conflicts and reset/delete semantics. Preserve portable
v1 links and fingerprints; no silent metadata loss during an old-client write
or merge. Folders, multiple per-move annotations, public comments, arbitrary
catalog reordering and featured games are outside the planned minimum.

### Later: Curated Learning Through Replays

- Build three to five short, authored scenarios around legal, complete games
  and selected nonterminal positions. Reuse the `0.6.2` example corpus where it
  fits; teach threats, forcing sequences and alternatives through Guide.
- Offer a clear prompt, inspection/analysis, Play From Here and a return/retry
  path to the original example. Use the existing bot and replay controls.
- Keep scenarios and explanations as static, versioned content. Opening an
  example does not import it or publish progress. Completed continuations join
  ordinary My games and statistics under the existing rules.
- Distinguish authored explanation, bounded analysis and the actual outcome
  against the selected bot. A different legal line is not automatically wrong.

Review the corpus and lesson interaction before implementation. This release
does not introduce automatic puzzle generation, a correctness-grading engine,
arbitrary starting-board sharing, a lesson editor or cloud progress tracking.

## Constraints Across the Line

- Preserve local-first guest play, the existing pixel fonts/icons/controls and
  board-first Match/Replay layouts. Public statistics stay independently opt-in
  and explicitly refreshed; no ratings or server-verified results are implied.
- Reuse static hosting, browser computation and bounded explicit cloud actions.
  Add no service, image storage, per-move request, polling or persistent sharing
  listener. Thumbnails and examples require no additional Firestore documents.
- Private notes and collection metadata never enter public profiles or portable
  links automatically. Opening a replay remains separate from saving it.
- Run checks appropriate to each change, with desktop/mobile and adverse-state
  visual review. Keep local validation, manual acceptance and publication as
  separate gates; SDK operation counts are not a free-tier capacity guarantee.

## Deferred Multiplayer

Multiplayer remains outside the planned `0.6.2`–`0.6.4` work because of its UX
and implementation effort, uncertain use and need for a second demo player.
The [P2P research](../backend/p2p_multiplayer.md) preserves the preferred casual
PoC: direct WebRTC moves, validation in both Rust/Wasm clients and a small
backend role for joining peers and signaling. Nothing is implemented.

Reopen it when there is a concrete two-player use case and demo/acceptance path.
Design connection failure, reconnect, leaving and rematch before implementation;
TURN, relaying, matchmaking discovery and trusted rankings remain separate
decisions. No connectivity success rate or player capacity is claimed.

## Parking Lot and Non-goals

Automatic puzzle generation and personalized "save this game" challenges,
richer statistics, avatars/bios, bot personalities, server-side bots and broader
themes remain parked. The later curated lessons do not adopt these projects.

Native apps, chat/social feeds, global player/game discovery, SSR, monetization
and ranked/esports features are outside this line. Cloud or sign-in must not
become a prerequisite for the core play and learning loop.

## Completed Context

The [release history](../../archive/release_history.md) and
[October closeout](../../archive/reconciliation_2026_10/README.md) retain the
`0.5.x` work and visual decisions. The
[`0.6.1` release record](../../archive/v0_6_1_release.md),
[original implementation plan](../../working/v0_6_implementation_plan.md) and
[execution record](../../working/v0_6_execution.md) retain the sharing rollout.
Completed reviews and rejected visual studies are not an additional backlog.
