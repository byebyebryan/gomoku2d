# Corridor Search

Purpose: define the bounded forced-sequence model used by replay analysis and
available to bot diagnostics.

Corridor search is not broad minimax. It only follows named tactical obligations:
immediate threats, imminent threats, counter-threats, immediate wins, and lethal
coverage. If play reaches a quiet position with no named continuation, the
corridor has ended for this model.

Source of truth in code:

- proof/search model: `gomoku-bot-lab/gomoku-bot/src/corridor.rs`
- tactical facts and policies: `gomoku-bot-lab/gomoku-bot/src/tactical/`
- replay consumer: `gomoku-bot-lab/gomoku-analysis/`

## Core Model

A threat corridor exists when one side creates an obligation that must be
answered. The branch space is bounded because perfect play does not ignore
active threats.

State transitions:

- enter when a side creates an immediate or imminent threat;
- stay while each move answers an active threat, wins immediately, creates a new
  immediate/imminent threat, or reaches lethal coverage;
- prove when a side wins or creates a lethal threat;
- escape when active threats are neutralized and no named forcing continuation
  remains;
- possible escape when a legal reply is visible but bounded proof cannot show it
  remains losing.

The model never starts quiet broad search to decide whether a neutral escaped
position is "really" winning later.

## Vocabulary

| Term | Meaning |
|---|---|
| Immediate threat | A four threat that wins next turn unless answered. |
| Imminent threat | A forcing three threat, including valid compound imminent obligations. |
| Lethal threat | Position-level coverage where no legal defender reply avoids terminal or known-lethal continuation. |
| Corridor reply | Named move that answers or creates an active corridor threat. |
| Forced reply | Defender reply that is legal but still leaves attacker a proven corridor win. |
| Escape | Defender reply that exits the detected corridor. |
| Possible escape | Defender reply not proven losing within corridor limits. |
| Full forced interval | Proof suffix from forced start through terminal conversion. |
| Setup corridor | Replay-facing span from forced start through lethal onset. |
| Lethal tail | Suffix after lethal onset through the terminal move. |

## Candidate Generation

Candidate generation is tiered:

1. Immediate wins and immediate threats.
2. Imminent threats and legal counter-threats.
3. Corridor entry denial when the latest escape is before the active threat.

Only the highest active tier is used for defender replies: immediate threats
suppress imminent replies. Within a tier, all surviving candidates are kept.

Then the model:

- marks actual replay moves as already-resolved in replay context;
- marks forbidden Black moves but does not probe them as legal alternatives;
- probes remaining legal non-actual alternatives;
- treats unresolved legal alternatives as possible escapes, not forced losses.

## Renju

Renju legality affects both sides of the proof:

- Black attack entries, completions, and defenses may be forbidden.
- White threats can become stronger when Black's required block is forbidden.
- Forbidden squares are evidence, not silent omissions.

The exact legality model lives in [`renju_rules.md`](renju_rules.md).

## Bot Integration Status

Corridor search was tested as a live bot shortcut/portal and did not pay for
itself under browser-scale compute budgets. The durable result is the shared
proof model and tactical vocabulary, not a default live-search extension.

Current useful roles:

- replay analysis and report generation;
- tactical diagnostics;
- future player education surfaces;
- possible targeted proof after normal search, only if measured again.

Retired portal/leaf-proof tuning history belongs in performance/archive notes,
not this model contract.
