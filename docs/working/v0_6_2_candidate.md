# v0.6.2 local candidate

Owner: Bryan. October 24, 2026. Local implementation and technical validation
are complete; this candidate has not been pushed, tagged or published.
Production remains v0.6.1. This is the single current handoff; the
[implementation archive](../archive/v0_6_implementation.md) retains earlier
design, review and mobile snapshots at their existing paths.

## What is ready

- Accepted two-column public player: simple identity/link card, optional dated
  record above compact games, stable pager, sparse/error states and preserved
  page/scroll/focus when returning from replay.
- Compact mobile Profile and records, short-landscape replay, responsive
  account controls and 200% text fixes.
- Guide has Tactics, Replays & sharing and Examples navigation, direct anchors,
  four concise action rows and optional privacy details. Home links to Examples.
- Two [static bot-game examples](../reference/app/replay_examples.md) reuse the
  standalone portable viewer. Opening neither imports nor saves the game.
  Share, explicit Save/deduplication and eligible Play From Here retain their
  existing behavior. Completed continuations join ordinary My games.
- Public profile enabling now uses Enable → Confirm enable / Cancel. No write
  occurs before confirmation; name/account changes invalidate consent.
  Public statistics remain separately off by default.
- Focused README sharing/profile media and updated replay media, with exact
  [capture provenance](../assets/README.md). Version and changelog are 0.6.2.

Application changes for the final completion slice are in `6d8e5a4`; capture
source is `bbc652f31fc4a211a85d1e60f61905617136f5a6` with clean application/version
inputs. Later documentation/evidence commits do not change those inputs.
No Rust, backend, rules, indexes, persistence schema, replay format, dependency
graph or catalog/library limit changed. No new cloud service/listener/polling.

## Automated evidence

[evidence.json](v0_6_2_candidate/evidence.json) records source, capture, build,
log hashes and preview identity. Raw artifacts remain ignored under
`gomoku-web/test-results/v062-completion/`.

| Gate | Result and scope |
|---|---|
| Web typecheck | Application, tooling and capture/encoder scripts pass |
| Web units | 508 tests / 51 files pass; application source matches the subsequent versioned candidate (version alone changed from 0.6.1) |
| Full ordinary browser lane | 231 pass, six intentional Firefox/WebKit WebGL qualification skips, zero failures; Chromium/Firefox/WebKit against the production build |
| Sharing/cloud lane | 33 pass, zero failures; C10–C19 plus M01 across three engines, isolated offset 1000 |
| Dependency audits | Production npm: zero vulnerabilities; Rust: zero vulnerabilities/warnings |
| Production/manual builds | Both pass, curated reports/direct-route entries included |
| Manual preview | All 147 current build files match served bytes; five preserved demo application-field hashes match recovery baseline |
| Media | Four reviewed outputs; 1280×800 PNGs, 960×600 GIFs at 5 fps (30 sharing / 27 analysis frames) |

The browser lane includes legal Wasm validation, guest discovery/save/share,
continuation/cancel and completed-game semantics, section focus/repeated anchors,
320px through desktop, portrait/landscape and 200% text. Existing Canvas
checks pass in all engines and WebGL checks pass in Chromium. The six skips
are explicit renderer qualification limits, not hidden failures.

Cloud C19 proves opening a standalone example makes zero Firestore requests,
including from a signed-in context; imported originals have no Publish action.
Enable/cancel/name-change checks verify no write before confirmation and default
statistics off. Existing reset/delete/revision/publication/return paths pass.
This is emulator evidence, not real OAuth or a capacity guarantee.

## Review on Starship

- [Examples](http://127.0.0.1:8013/guide/#examples): open both, inspect/share/save,
  try the eligible 26-ply position, and return to Guide.
- [Replays & sharing](http://127.0.0.1:8013/guide/#replays): check hierarchy, copy,
  section navigation and privacy disclosure.
- [Demo Player](http://127.0.0.1:8013/player/?id=v061-demo-player): optional record,
  all three logical pages and replay return. Existing fixture remains 18 games.
- [Profile](http://127.0.0.1:8013/profile/): enable/cancel confirmation and mobile
  controls. This preview uses local Auth/Firestore emulators.

The existing manual services/data were preserved; port 8001 was left untouched.
Older preview chunks remain available to already-open tabs. No user-browser
tabs were opened. Read-only served-byte and demo-field checks follow capture.
The prior emulator recovery regenerated Firestore creation/update metadata;
application fields are compared with that recovery baseline, not old metadata.

## Acceptance and publication boundary

Bryan previously approved the player-card direction and gave a broadly positive
mobile visual pass. Final new help/example/confirmation/media visual acceptance,
physical phone/touch/browser-keyboard, human screen-reader/zoom and novice checks
are not claimed. The inherited real signed-in sharing and two-device private
account/reset handoffs remain open or explicitly deferred as recorded in the
[v0.6.1 release](../archive/v0_6_1_release.md).

The version/changelog and local candidate are prepared. The normal release
script requires clean pushed main, so a local-only candidate cannot pass its
remote-parity gate. After separately authorized push, require hosted CI on the
exact release head, rerun `scripts/release.sh --check 0.6.2`, then follow the
[release workflow](../reference/ops/release.md) for tag/deployment and production
served-byte/journey verification. No extra backend/index migration is needed.
Do not record this local checkpoint as publication.

Next feature scope remains 0.6.3: thumbnail prototype plus reviewed private
metadata, version/old-client and conflict contracts. It is not part of this
candidate.
