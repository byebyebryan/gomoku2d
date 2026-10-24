# v0.6.2 completion plan

Owner: Bryan. Drafted October 24, 2026 against local `54848dc`.
Status: implementation authorized by Bryan's October 24 goal-loop request.
Proceed with the recommended Guide section and inline enable confirmation;
freeze the two candidate games after current-core validation. Publication remains
separate from this local implementation/candidate checkpoint.

The [follow-up plan](v0_6_followup_plan.md) owns the broader research and
0.6.2–0.6.4 sequence. The public-player layout, replay return and mobile fixes
are implemented and technically checked. This plan finishes the remaining
0.6.2 help, demonstration, owner consistency and presentation work.

## Outcome and boundaries

A guest can find an example, open a real replay, understand Share/Save/Publish,
and deliberately save a study copy or try an eligible earlier position. An
owner can enable a public profile with explicit consent and understand which
games and statistics are public. Release documentation demonstrates that path.

Retain the accepted profile composition, compact lists, stable pager and
full-sidebar Share/Publish behavior. Reopen them only for a concrete defect.
Keep thumbnails, private titles/notes and saved opening moments in 0.6.3;
authored tactical lessons, prompts/reveals and retry flows remain in 0.6.4.
No new service, database family, schema, replay format, background listener,
public discovery, analytics or per-move request is needed.

## 1. Concise player help

**Selected placement:** a compact Replays & sharing section in the existing
Guide, with clearly labeled local navigation and the examples nearby. Keep
the current tactical sections and their identity.

Make the section reachable without scrolling through all tactical diagrams.
Use the existing fonts, icons and button styles for navigation; no onboarding
modal, long sidebar explanation or new icon design. Existing Board and replay
guide controls remain useful entry points. Direct section links must work on
first load as well as client navigation, including keyboard focus and the
Guide's actual scroll container.

Keep the central explanation to an action table plus short state notes:

| Action | Explanation to convey |
|---|---|
| Share replay | Creates a portable link to the game and selected opening move. Review names/profile links before copying. Anyone holding the link can view it; it does not require publication. |
| Save replay | Explicitly keeps a private study copy in Saved replays. It does not claim the viewer played the original game or add it to played-game totals. |
| Publish replay | Adds an eligible own-game listing to an enabled public profile. Sign-in and deliberate publication are required. |
| Play From Here | Starts an ordinary new game from an eligible position against the selected bot. It becomes a My game when completed; no separate Practice badge/category. |

Explain Local versus signed-in Cloud state without promising successful sync
while offline. My games, Saved replays and the public catalog have different
membership. Public statistics are a separately enabled, dated My games
snapshot, explicitly refreshed; they are not totals of the selected catalog.
Unpublishing/disabling removes listings, not existing portable copies.
Opening a shared/example replay does not automatically save, import or publish.

## 2. Two static example replays

Ship two examples initially. Each is one compact row with a short title,
one-line purpose, rules/move information and the established Open replay
action. No board thumbnails, carousel, examples account or seeded public page.

| Proposed example | Initial view | Candidate source |
|---|---|---|
| Review a finished game | Finished board; use the existing timeline/analysis controls | Published analysis entry `match_0095__search-d1__vs__search-d3_pattern-eval`, 28 moves, also used by existing README analysis capture |
| Try another line | A selected eligible earlier position; existing Play From Here stays inside the viewer | Published entry `match_1129__search-d3_pattern-eval__vs__search-d7_tactical-cap-8_pattern-eval_corridor-proof-c16-d8-w4`, 36 moves; ply 26 is a candidate, not a claim of a guaranteed escape |

Freeze the exact sources/opening plies only after current-core validation and
a visual content check. Both candidates come from `reports/lab/analysis-report.json`;
the source tournament/analysis provenance is already recorded there. They are
engine-generated games, not user submissions. Their participant specifications
map to the existing Easy/Normal and Normal/Hard presets respectively. Preserve
truthful participant kinds/configurations with neutral display labels and no
real profile UIDs; never turn bot records into fictitious human results to
simplify an adapter.

Bundle a small, versioned content manifest with stable content IDs, titles,
short descriptions, complete portable games, selected view plies and source
references. Reuse the portable codec, legality validator and existing shared
replay viewer. Do not ship the whole lab report or build another board/viewer.
Examples can use the existing `/replay/shared/#g2r1=...` transport; no dedicated
example route, short-link service or persistence representation is needed.
Documentation can point to the Guide section and ordinary portable links.

The viewer remains a standalone public route: no private-history controller
or Firestore operations are introduced by opening an example. Save/Share and
eligible continuation use their existing controls and guards. The existing
Guide action provides a way back to help/examples. A copied portable link
remains independently usable without example metadata.

Source games must start from an empty 15×15 board, be legal through their
terminal result, and have no moves after the result. Selected continuation
plies must satisfy current eligibility, including the minimum inherited ply.
Do not encode the Guide's cropped/synthetic 7×7 diagrams as full games.
Recomputed analysis is bounded; neither example is a graded puzzle or a claim
that only one continuation is valid.

## 3. Bounded owner/action consistency pass

Keep the existing General Settings-style owner rows. Review the signed-out,
off, enabled-with-no-games and enabled-with-publications states alongside the
new help, without adding more descriptive blocks to sidebars.

**Selected enable flow:** replace the native enable checkbox with an inline
confirmation matching the existing disable/reset language. The first Enable
action opens a short confirmation showing the selected public name and scope;
Confirm enable performs the write, Cancel performs none. Name/account changes
invalidate the confirmation. Preserve explicit consent, default-off statistics,
pending/error feedback, revision guards and the existing cleanup barriers.

Check that guest examples and imported study copies cannot publish the
original as a played game. A completed eligible continuation retains normal
My games/publication behavior. No statistics or attribution policy changes.

## 4. Documentation and focused media

After the UI/content review, add a public-profile capture and a short
example → replay/share demonstration to the README where they explain the new
capability. Preserve its prominent live-app link, setup and repository map.
Update the affected replay/profile media; retain unrelated lab/visual assets
with their existing dated provenance. Do not regenerate reports or redesign
OG artwork without a concrete need.

Reconcile app/reference routes, player-help references, docs status and the
working-note index; keep their status aligned with the actual implementation.
At completion, archive/index completed 0.6
planning material while preserving incoming links, published snapshots and
release evidence. Avoid leaving several competing active implementation plans.

Prepare and review candidate compositions after UI approval. Capture final
release-facing assets after the version is set during release preparation,
with accurate source/version provenance. Use an owned free preview port and
explicit base URL; never take unrelated port 8001.

## Implementation checkpoints

| Checkpoint | Work | Exit condition |
|---|---|---|
| A — content and presentation | Resolve help placement; finalize the short copy, two source games and opening plies; prepare the compact section/row composition | Current validator accepts both full games; provenance/participant mapping is truthful; entry points, copy and visual scope are settled |
| B — working guest and owner journey | Integrate static help/examples with the existing viewer; implement the owner confirmation; add focused contract coverage | Guest open/save/share/continue and owner enable/cancel/error work; opening examples has no import, history or cloud-write side effects |
| C — presentation closeout | Review desktop/mobile/enlarged text; prepare README media/copy and current docs; reconcile active notes | Accepted visible UI/media composition and accurate, linked documentation; retained human acceptance limits stated honestly |
| D — release preparation | Version/changelog, final media capture/provenance, proportional final tests/audits and release preflight | Concrete release candidate and validation record, ready for separately authorized push/publication |

Local implementation/docs commits should follow cohesive checkpoints. Keep
application work separate from tag/deployment actions. The goal loop authorizes
local implementation and candidate preparation, not push or publication.

## Acceptance and release gates

- Validate the two complete games, chosen plies, canonical portable identity,
  safe labels/no UIDs and manifest bounds. Scrubbing does not persist anything.
- Browser journey: Home → discover help/examples → open each → step/scrub,
  Share/clipboard fallback, explicit Save/deduplication, eligible continuation.
  Opening leaves My games/statistics untouched; canceling active-game
  replacement preserves that game. Completed continuations count normally.
- Owner journey: enable/cancel/name-change/account-change, off-by-default
  statistics, failed-write feedback and publication eligibility. Preserve
  reset/delete/revision behavior; reuse existing scenario coverage.
- Inspect 320px, portrait, short landscape, 200% text, keyboard/focus and direct
  Guide-section links. Use Chromium, Firefox and WebKit; physical phone,
  screen-reader and real-account evidence remain separate.
- Run web typecheck/units/build and the changed browser/cloud scenarios.
  Cloud harnesses always use `GOMOKU_CLOUD_PORT_OFFSET=1000`; preserve the manual
  preview/emulator snapshot and demo. At release, perform the repository's
  mandatory production dependency audits and relevant final gates once.
- Verify final manual-preview bytes and current restored demo fields. The
  October 24 recovery regenerated Firestore document metadata; use its recovery
  baseline instead of mistaking changed creation timestamps for changed data.
- Complete the existing real-account sharing check where available, or carry
  it forward explicitly alongside retained device/accessibility deferrals.
  Do not record emulator checks as real Google sign-in acceptance.
- Follow [Release Workflow](../reference/ops/release.md) for exact-head hosted
  CI, tagging, deployment and served-byte checks after authorization. No extra
  backend/index migration is implied by this UI/content release.
