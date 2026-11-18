# 0.6.4 replay collection design review

Prepared November 18, 2026 against `f94dde6bf209af29122a5fcfbbbdc8774534b50c`.
Bryan approved the revised UI direction on November 18. The storage and sync
contract remains proposed.

Open the [revised interactive review](http://starship:8787/reports/7442fffba177c793585cd6411433a5bab87b5ccbc9189b3d74bb93b7d5394f2c/) in a desktop or phone browser.
It uses fixtures and retains edits until reload. The
[proposed contract](v0_6_4_replay_collection_contract.md) records the field,
storage, migration and sync behavior for implementation.

The first review used a stand-in for Replay Analysis and gave the impression
that saved replays would have a separate viewer. The revision uses the app's
existing Replay Analysis component, board renderer and analysis worker.
**Edit details** sits beside Share replay and uses the same sidebar swap as
Share and Publish. Public editing enters the same viewer with its edit
sidebar open. The reviewer can use the existing timeline and step controls
inside either editor to choose the opening.

1. Open **Threat before the finish**, edit its details, scrub to a move, select
   **Use move N** and save. Return to the list and open it again. Try move 0
   and **Use default** to see both opening behaviors.
2. In **Public settings**, edit a publication's title and opening. Check the
   visitor row and starting board in **Public profile**. Choose the changed
   or removed publication fixture to see how a failed edit is reported.
3. In **Saved replays**, choose **Account conflict**. Compare both versions
   and choose details for each replay. Review the merge before confirming.
4. Check the row density at phone width, including the long title and the two
   sizes of thumbnail. Use the 64-entry fixture to inspect incremental reveal.

## Design decisions

| Area | Direction |
|---|---|
| List | Full-board thumbnails at 90 CSS px desktop and 75 CSS px narrow, using existing stone assets and a lighter grid. Keep compact rows and explicit Replay. |
| Editors | Keep the existing Replay Analysis layout and replace its sidebar while editing. Use the existing timeline and step controls inside the editor and stage the opening with **Use move N**. |
| Conflict | Choose one coherent title/note/opening set per replay. Save only after all choices and the merge summary have been reviewed. |
| Storage | 80-code-point title, 280-code-point note. Use a new authoritative local key, and one-way v2 upgrades at the existing Firestore path. |
| Publishing | Mirror the chosen public opening in both existing `view_ply` and portable `at`. This preserves behavior for older visitors who open the payload directly. |

The first implementation slice should prove the codecs, migration, old-tab
write guard and reset/delete path. The design audit found that replacing a v1
cache in place relies on a raw-value check followed by a separate write. The
new-key proposal makes protections independent of the timing of that check.
The audit also found that v1 can update memory after a failed durable write,
and merge can write before validating incoming replay fingerprints. Carry
both findings into v2 acceptance.

## Evidence

- **Revised viewer.** Eight follow-up runs passed across Chromium, Firefox
  and WebKit on desktop and portrait, plus Chromium at 320px and 200% root
  text. Checked the normal viewer, sidebar swap, board width on desktop,
  focus restoration, unchanged staged opening while scrubbing, save/reopen,
  discard, public edit and the visitor return. The review-only build injects
  the editor and fixture adapter at build time. Sources and captures are under
  `gomoku-web/test-results/v064-design/replay-preview/` and
  `gomoku-web/test-results/v064-design/revised-evidence/`.
- **Revised delivery.** All 33 files matched their served bytes. Chromium
  passed the replay and editing flows at the delivered URL on desktop and
  portrait, with zero third-party requests or page errors.
- **Prototype.** Twelve browser runs passed across Chromium, Firefox and WebKit
  at 1280×900, 390×844, 320×760 and 640×900 with 200% root text. Covered
  private save and reopen, default and move 0, staged changes, discard, public
  edit and stale/removed states, pagination and return focus, explicit conflict
  choices and expansion to 64. No horizontal overflow, page errors or third
  party requests in these runs. Axe reported zero violations on the two
  Chromium conflict captures tested. Desktop and portrait captures were
  visually inspected. Portrait is a browser viewport reference.
- **Thumbnails.** Exact Black, White and omitted move pixels matched across all
  configurations for the move-18 prefix of the 28-move example. Three existing
  game fixtures supply the moves, including a 225-move draw. Across tested
  configurations, median Canvas drawing time was 0–1 ms for eight rows and
  3–9 ms for 64. Drawing 64 full boards with 225 moves each on a reused canvas
  took 3.5–19.5 ms. Low results include browser timer quantization. These are
  host CPU draw timings over prepared move arrays, separate from decoding,
  layout, painting and physical-device performance.
- **Storage.** The byte probe puts a full 64-entry cloud library at or below
  250,433 ASCII bytes for the current canonical portable format and proposed
  Unicode limits. It also documents overflow for 64 synthetic tokens at the
  transport ceiling and for three simultaneous full local owner buckets.
- **Compatibility.** Three browser engines passed a standalone storage probe.
  Late v1 writes and removes touched only the retired key, and concurrent
  v2 writers using an origin lock and the same expected revision produced one
  save and one conflict. Nineteen isolated Firestore emulator cases exercised
  proposed rules, including legacy writes before migration, downgrade denial
  after migration or clear, private read isolation and profile deletion.
  Fifty-two existing replay/store/sync/action/codec tests passed on the baseline.
- **Initial report delivery.** All 22 published files returned HTTP 200 with identical bytes.
  A smoke check on the published URL covered the prototype, contract reflow
  and the unavailable-sprite fallback with Replay still usable. The report
  uses bundled fonts, icons and stone art with their licenses.

Local evidence and reproduction scripts are under
`gomoku-web/test-results/v064-design/`. The initial immutable snapshot is
`900939af1389e03f62a4a6137857578e09a9d59bf5207c96bf95b63e756f7e10`.
The local and emulator probes use synthetic data and exercise the proposed
mechanisms. The actual app migration, full deletion/reset paths, account
switching, offline recovery and 32-entry public cap remain implementation
checks. Physical phone, real-account and human accessibility review follow the
usual manual acceptance path.
