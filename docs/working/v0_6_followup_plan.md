# v0.6.2–v0.6.4 follow-up plan

Owner: Bryan. Reviewed October 23, 2026 against local `main` at `84f1c1f`.
The published `v0.6.1` source is `b1a67d3`; the intervening commit changes
documentation only. Bryan selected UI/presentation for `0.6.2`, replay
collection improvements for `0.6.3`, and assigned the subsequent learning
extension to `0.6.4`. The [roadmap](../reference/product/roadmap.md) owns that
sequence. This plan records the research, proposed interaction details and
decisions required before implementation; it does not change current contracts.

## Review scope and evidence

This was a source/documentation review and inspection of recorded candidate
captures, with targeted first-party product and web-platform research. It was
not a new live-account, phone, performance or human-accessibility acceptance
run. No runtime, production data, dependency or infrastructure was changed.

| Local evidence | Finding and consequence |
|---|---|
| [Public visitor](../../gomoku-web/src/routes/PublicPlayerRoute.tsx), [recorded mobile profile](v0_6_1_evidence/sharing-chromium-public-profile.png) and [maximum-label capture](v0_6_1_evidence/sharing-chromium-maximum-visitor.png) | Identity, five statistics tiles, disclosure and game rows form one long stack. Review hierarchy and sparse/full composition before adding more fields. |
| [Owner controls](../../gomoku-web/src/routes/PublicProfileManager.tsx) and [recorded desktop manager](v0_6_1_evidence/sharing-chromium-profile-manager-desktop.png) | General Settings-style rows are established. Refine density and action feedback within that language. Statistics use Off/On buttons; the separate enable confirmation still uses a checkbox and needs a consistency review while preserving deliberate consent. |
| [Replay viewer](../../gomoku-web/src/routes/ReplayRoute.tsx) and [public visitor](../../gomoku-web/src/routes/PublicPlayerRoute.tsx) | Replay's Profile action leads to the owner's record screen. Public pagination lives in route component state. Define the return to the originating public page and its list position, separately from My profile. |
| [Saved replay panel](../../gomoku-web/src/routes/ReplayLibraryPanel.tsx) | Entries show player names, moves, rules and save time, but have no private title, note or thumbnail. New study tools belong in `0.6.3`; `0.6.2` can improve existing presentation. |
| [Local library](../../gomoku-web/src/replay/replay_library_store.ts), [stored viewer](../../gomoku-web/src/routes/StoredReplayRoute.tsx) and [sync](../../gomoku-web/src/replay/replay_library_sync.ts) | Entries have exact keys and canonical payloads discard `viewPly`. Saved opening moments and metadata need a storage/compatibility design. The current merge policy is not a per-field note-conflict policy. |
| [Board component](../../gomoku-web/src/components/Board/Board.tsx) | Each mounted Board creates a Phaser game. List thumbnails need a lightweight static projection instead of one game instance per row. |
| [Portable v1](../reference/app/portable_replay.md) and [sharing data](../reference/backend/sharing_data.md) | Links contain complete legal 15×15 games; view ply is excluded from identity. Catalogs hold 32 games and private libraries 64. Reuse already fetched moves for previews. |
| [Guide](../../gomoku-web/src/routes/GuideRoute.tsx) | It explains tactics and analysis, but not the save/share/publish workflow. Many diagrams are partial 7×7 teaching positions, not portable full games; encoding their cells is insufficient for Play From Here. |
| [README](../../README.md), [media provenance](../assets/README.md), [app design](../reference/app/app_design.md) and [working index](README.md) | Media still documents `0.5.5`; app route documentation and the active-plan index lag the shipped sharing surfaces. Reconcile references and archive completed plans during `0.6.2`, preserving evidence and incoming links. |

## External research and adopted implications

Sources were checked on October 23, 2026. These are reference patterns, not
evidence that a particular layout works for Gomoku2D or a mandate to reproduce
another product's feature set.

| First-party source | Observation | Implication for this line |
|---|---|---|
| [Lichess: Study chess, the lichess way](https://lichess.org/@/lichess/blog/study-chess-the-lichess-way/V0KrLSkA) | Studies combine games/positions, comments and chapters, with explicit public/private access. The article also describes hiding moves for lessons. This introductory reference dates from 2016. | Give saved replays useful context and a deliberate opening moment. Keep private metadata separate from public publication. Collaboration, chapters and a lesson editor exceed this line. |
| [Chess.com: Game Collections](https://support.chess.com/en/articles/13557248-how-do-i-use-game-collections) | Current help describes named collections within Analysis, explicit visibility, editing, searching and sharing actions. | Keep collection work near replay/analysis and make visibility explicit. A folder hierarchy, community catalog or viewer/editor permission model is unnecessary for the bounded library. |
| [Lichess Practice](https://lichess.org/practice) | The public entry organizes learning around tactical topics and presents sign-up as a way to save progress. | Make `0.6.4` a small, topic-led set of guest-accessible examples. Progress infrastructure is not required to demonstrate learning. |
| [W3C: consistent navigation](https://www.w3.org/WAI/WCAG22/Understanding/consistent-navigation), [reflow](https://www.w3.org/WAI/WCAG22/Understanding/reflow.html) and [target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html) | Repeated navigation retains relative order. Non-excepted content reflows at 320 CSS pixels; small targets need sufficient size or spacing under the criterion's conditions. | Preserve stable Previous/Next positions and visible focus. Review narrow/enlarged layouts without shrinking usable targets. These checks do not constitute a blanket WCAG conformance claim. |
| [MDN: URI fragment](https://developer.mozilla.org/en-US/docs/Web/URI/Reference/Fragment) | The fragment is processed by the client and is not sent in the HTTP request. | Ordinary server-side unfurl requests cannot select a per-game image from fragment data. Keep portable transport; dynamic social cards, a URL shortener and an image service are outside the plan. Static project/example artwork remains possible. |

The sequencing judgment is ours: make the shipped experience coherent, add
persistent study tools, then use the foundations for authored learning.
Research does not establish usage, hosting capacity or measured usability gains.

## `0.6.2`: public-profile design and a clear sharing journey

The [public-profile design study](v0_6_2_design/README.md) explores three
visitor compositions and a shared owner companion against the existing visual
language. Bryan's preferred refinement combines A's record treatment with B's
columns: simple identity on the left, statistics above games on the right.
Bryan accepted the subsequent public player card with shared-game count, link
and record context. The [implementation checkpoint](v0_6_2_implementation.md)
tracks the real-data UI, technical acceptance and preview; broader 0.6.2 help,
examples and media remain separately sequenced.

### Planned minimum

1. Review visitor and owner screens together. Establish identity → optional
   record → published games, with balanced desktop composition and compact
   mobile rows. Statistics off and no games must look intentional. Keep the
   dated snapshot and casual/unverified status understandable without a
   dominant block of explanatory prose.
2. Apply existing fonts, tokens, icons, buttons and settings patterns across
   Public profile, My games and Saved replays. Preserve accepted Share/Publish
   full-sidebar behavior. Review the enable-confirmation checkbox against
   established confirmation patterns; retain explicit public-sharing consent.
3. Make action destinations clear: Share creates a portable link; Save keeps
   a private study copy; Publish adds an own-game listing to the public page.
   Play From Here remains an ordinary continuation. Put essential scope/status
   information beside the action instead of adding paragraphs.
4. Provide a return from replay to its originating public profile, preserving
   logical page and scroll position. Keep Previous/Next in stable positions and
   handle changed/disabled profiles honestly. Scope transient navigation state
   by public UID; never reuse stale owner state across accounts or navigation.
5. Give empty/loading/error/offline states useful actions, consistent retry and
   focus behavior. Visitor empty states must not expose owner-only controls;
   owner empty states can lead to My games and publishing.
6. Add short player help for local/private/public state and replay actions.
   Keep Guide's tactical identity; choose a compact section or separate linked
   surface during design. Avoid mandatory onboarding or instructions repeated
   throughout the sidebars.
7. Ship two or three static replay examples with generic identities and no real
   user UIDs. Use the existing viewer, analysis and eligible Play From Here
   controls, an obvious help/Guide entry and direct documentation links. These
   are examples, not a discovery service or production-seeded public profiles.
8. Refresh README/profile/sharing media after visual approval. Keep the root
   README focused on the live app, concise capabilities, setup and repository
   references. Update app routes/screen contracts and player help; archive
   completed plans while preserving links and release receipts. Leave unrelated
   bot reports and unaffected screenshots alone.

### Decisions and acceptance before shipping

Compare compact rows with a thumbnail variant using the same empty/sparse/full
data. Select composition, navigation hierarchy, statistics density, disclosure
placement, owner preview actions and originating-profile return behavior with
Bryan before implementation. The reusable thumbnail renderer ships in `0.6.3`
and does not block `0.6.2`. Select help placement and examples; validate full-game
legality, safe labels and eligible opening plies.

Review the outstanding real-account sharing handoff early where available;
record it as open until completed or explicitly deferred. Inspect statistics
on/off, empty/one-game/full eight-row pages, long Unicode labels, all pagination
positions and adverse states. Cover desktop, portrait mobile, 320px reflow,
enlarged text and keyboard/focus. Include actual phone/touch review when
available; distinguish unperformed human checks from automated results.

A guest can discover an example, inspect it, deliberately save it or start an
eligible continuation, and find the next relevant action without sign-in.
Opening examples/shared replays creates no private history or sharing database
work. Public profiles retain bounded root/page reads; list return introduces
no polling. Help/media match accepted UI, current behavior and recorded capture
source. No persistence schema change is required for this release.

## `0.6.3`: useful replay collections

### Planned minimum

- Add non-animated board thumbnails to saved/public replay lists. Use the
  deliberately chosen opening move when set, otherwise the finished position.
  Keep adjacent game information and a usable Replay action. Do not present
  computed tactical overlays as stored evidence.
- Give saved replays a private title, one bounded plain-text note and one
  deliberately saved opening move. Provide explicit Save and Clear/reset
  actions; viewing or scrubbing must not rewrite the library. This is one
  opening point per replay, not a list of per-ply annotations.
- Keep metadata outside portable payloads and fingerprints. Editing it does
  not duplicate a game, change player names, or imply the owner played an
  imported record. No saved opening point means the ordinary replay default.
- Allow explicit edits of an existing public title/opening move through the
  owner flow. Public and private openings may differ. Private titles/notes
  never copy automatically into exports or public data; publication remains
  an independent review action.
- Retain 32 publications, four physical public pages, 64 private entries and a
  bounded snapshot. Thumbnails add no cloud operations beyond existing loads.
  Explicit metadata saves may use bounded existing sync; they are not cost-free
  and do not run on each keystroke or replay move.

### Technical decisions before implementation

Current exact-key entry parsers accept only fingerprint, payload, kind and
save time; the local store strips viewing ply. Cloud sync replaces a complete
revisioned snapshot. A note input alone would not make metadata durable or safe.

Set exact fields, character/byte limits and total snapshot budgets first.
Review a versioned local/cloud representation, legacy reads, safe old-tab
behavior and update-required states. Prove an old writer cannot erase new
metadata. Define whole-entry or field-level conflict review before changing
the existing merge action; never silently prefer one device's note. Preserve
guest import, account switching, offline pending changes, reset/delete and
revision barriers. Do not silently evict entries to make metadata fit.

Use a bounded static projection of validated moves and accepted palette/assets
for thumbnails. Do not mount one Phaser game or run analysis per row. Select
the renderer after a small visual/performance prototype; render the visible
list with a neutral fallback for unavailable records. Do not introduce a
second legality engine. Keep portable v1 decoding and identity unchanged.

### Acceptance

Metadata survives reload and supported explicit guest/account sync. Reopening
uses the selected move; clearing restores the ordinary default. Re-saving does
not duplicate the fingerprint. Private edits leave public data and portable
links unchanged. Test new/legacy reads, old-client writes, metadata conflicts,
offline edits, account changes, guest import, storage denial and lifecycle
cleanup against the selected contract.

Inspect thumbnail stone colours and move prefixes on desktop/mobile, including
Canvas-only environments. Measure populated lists to confirm no per-row Phaser
games, analysis jobs or added cloud operations. Human visual acceptance remains
separate from these technical checks.

Folders, multiple per-move annotations, public comments, arbitrary catalog
reordering and featured games are outside the planned minimum.

## `0.6.4`: curated learning through replays

### Planned minimum

Ship three to five authored scenarios organized by tactical idea, using
complete legal games plus selected eligible nonterminal plies. Reuse or extend
the `0.6.2` corpus. Initial topics can include answering a threat, recognizing
a forcing sequence and exploring an alternative before the ending. Select
final examples from rules/analysis evidence, not attractive diagrams alone.

Each scenario provides a short prompt, source inspection, optional explanation/
analysis reveal, Play From Here against the selected existing bot, and a clear
return/retry path to the original position. Choose Guide entry and reveal
behavior in a reviewed interaction prototype. Preserve active-game replacement
warnings; returning to a lesson must not silently discard an ongoing game.

Content is static, versioned and carries source provenance. Opening or
resetting an example does not save a match, import a study copy, update cloud
progress or award statistics. A completed continuation remains an ordinary
My game with source origin and current counting rules. Do not reintroduce a
Practice badge or separate played-game category.

### Decisions and correctness boundaries

- Current Guide diagrams may be cropped or synthetic. Keep them as diagrams
  unless a reviewed full legal source supports the lesson. Portable v1 still
  starts on an empty 15×15 board and ends in a legal win/draw; select a ply of
  at least four within the currently eligible continuation range.
- A main line is an example; another legal move is not automatically wrong.
  Avoid "only correct move" or guaranteed-win claims without appropriate proof.
- Distinguish authored guidance, recomputed bounded analysis, confirmed versus
  possible escapes and the outcome against a particular bot configuration.
- Review bot/configuration and reproduction expectations. The authored source
  and explanation provide a stable demonstration; the bot need not reproduce
  one exact continuation for the lesson to work.
- Exclude automatic puzzle extraction, arbitrary-position sharing, correctness
  grading, public lesson authoring, a branching study editor, achievements and
  account-based progress.

### Acceptance

A guest can enter from Guide, understand a prompt, inspect/reveal an
explanation, start a continuation and return to retry. Each scenario has a
legal full-game source, reviewed prompt/explanation, chosen ply and usable
mobile/keyboard path. Verify no history/cloud-progress effects from opening
or resetting, while actual finished games retain normal history and origin.
Saving/publishing still require explicit actions. Handle analysis unavailable
and different valid moves without inventing a score or failed-lesson state.

## Shared constraints and next checkpoint

Keep static hosting and browser rules/bots/analysis; add no service, thumbnail
storage, background sharing listener, polling or per-move request. No schema or
infrastructure migration is authorized by this planning commit. Future schema,
rules or index changes require their own reviewed compatibility and release
sequence. Public statistics stay opt-in and manually refreshed; richer
statistics need a specific useful question and supporting records.

Avatars/bios, social discovery, collaboration, public notes, featured games,
automatic puzzles, broad skins and multiplayer remain outside the planned
minimum. The [P2P research](../reference/backend/p2p_multiplayer.md) retains the
later multiplayer direction and its deferral.

The immediate checkpoint is `0.6.2` composition/interaction review, then a
bounded implementation plan for the accepted design. Validate changed areas
instead of reopening unrelated completed audits. Commit, push, backend
deployment, tagging, served-byte verification and human acceptance remain
separate gates. Preserve manual preview/emulators and their data; use isolated
test ports and keep routine captures out of the user's browser tabs.
