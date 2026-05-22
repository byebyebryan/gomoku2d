# v0.4.7 Lethal Threat Plan

Purpose: add a precise lethal-threat layer on top of the existing immediate
and imminent threat model, then use it to improve replay analysis before trying
search integration.

This is an ad-hoc working plan. Canonical tactical terminology lives in
`docs/reference/lab/tactical_shapes.md`, `docs/reference/lab/corridor_search.md`, and
`docs/reference/lab/game_analysis.md`.

## Context

The current model has two useful non-lethal threat tiers:

- immediate threats: fours that must be answered now;
- imminent threats: forcing threes that create a narrow reply corridor.

Those tiers explain many forced corridors, but they do not name the moment when
the defender is already tactically dead before the final five appears. That is
the missing "lethal threat" layer.

A lethal threat is not a new local shape kind. It is a position-level
reply-coverage result: the attacker has legal terminal continuations, or legal
continuations into already-known lethal states, and the defender has no legal
reply that neutralizes them all.

## Design Guardrails

- Keep `LocalThreatKind` as local shape facts only.
- Put lethal detection in shared Rust logic under `gomoku-bot`, not in report
  or web rendering code.
- Treat Renju legality as part of the proof, not as a hardcoded shortcut.
- Prefer conservative false negatives over false positives.
- Validate in reports before wiring lethal as a search terminal.
- Do not mix classifier work, analyzer behavior, and bot-strength tuning in one
  commit.

## Slice 1: Terminal Coverage

Add a reusable lethal classifier for terminal threats only.

Target API shape:

```text
terminal_lethal_threat(board, attacker) -> Option<LethalThreat>
```

The classifier should return lethal only when:

- the board is ongoing and the defender is to move;
- the attacker has one or more legal immediate winning moves;
- the defender does not have an immediate winning move of their own;
- no legal defender reply covers every attacker terminal target.

This should cover:

- freestyle open four;
- multiple independent immediate winning targets;
- Renju cases where White's only natural block is forbidden for Black;
- legal Black open-four terminal coverage, while excluding illegal Black
  terminal targets through core legality.

This should not cover:

- single closed four or broken four with one legal block;
- 4+3;
- 3+3;
- any shape whose lethality depends on a next attacker move after a defender
  reply.

The output should include evidence, not just a boolean:

- attacker;
- terminal targets;
- legal covering replies, if any;
- optional defender immediate wins that prevented the lethal classification.

Implementation checkpoint:

- `gomoku_bot::tactical::terminal_lethal_threat` returns the proven lethal
  case.
- `gomoku_bot::tactical::terminal_lethal_threat_analysis` returns the same
  terminal targets plus defender immediate wins and covering replies, so tests
  and future reports can explain why a position was not lethal.
- Focused tests cover open-four terminal coverage, a single blockable four,
  defender immediate-win override, and a Renju-forbidden direct block.

## Slice 2: One-Step Lethal Coverage

Extend the classifier from terminal coverage to one-step lethal coverage.

This answers: if the defender replies to the current threat, can the attacker
still play a legal move that creates a terminal lethal threat, and does every
legal defender reply fail in that way?

Candidate scope:

- use existing immediate and imminent defender-reply generation as the reply
  surface;
- after each legal defender reply, inspect attacker moves that enter a forcing
  local threat;
- classify only moves that create terminal lethal coverage from Slice 1;
- if any legal defender reply avoids terminal coverage, treat the position as
  not proven lethal;
- if the defender has an immediate win or unhandled counter-threat, treat it as
  not proven lethal unless the attacker can answer it and still prove terminal
  lethal.

This is the first slice that can cover:

- 4+3;
- 3+3;
- "block one threat, attacker makes open four anyway" positions.

Evidence should distinguish:

- terminal targets available now;
- lethal entries available after a defender reply;
- defender replies that fail;
- defender replies that cover or escape;
- forbidden replies excluded by Renju legality.

Implementation checkpoint:

- `gomoku_bot::tactical::lethal_threat` checks terminal coverage first, then
  one-step coverage.
- `gomoku_bot::tactical::one_step_lethal_threat_analysis` records each defender
  reply, the attacker entries that create terminal coverage after that reply,
  defender immediate wins, and escaping replies.
- The lethal scenario harness now validates crossed `4+3`, crossed `3+3`, a
  non-lethal crossed broken-three pair with a shared block, and a non-lethal
  single open three alongside the terminal-coverage cases.

## Why Slice 2 Exists

Slice 1 is necessary but incomplete. It can only recognize positions where the
attacker already has terminal winning squares and the defender has no legal
coverage. That catches freestyle open fours, immediate multi-four positions,
and Renju single-threat cases where the only block is forbidden. Open four is
the common local lethal case in freestyle, but Renju keeps the rule generalized:
shape names only propose candidates; legal terminal targets and legal defender
coverage decide whether the position is lethal. The more common interesting
lethal cases still come from non-local combinations.

The classic lethal patterns are not always terminal yet:

- in 4+3, the defender may block the immediate four, but the attacker's three
  can still become an open four;
- in 3+3, there may be no immediate five at all, but every defender answer to
  one three still lets the attacker turn the other into a lethal open four;
- in Renju, legality can turn what looks like a normal reply into no reply, but
  this still has to be checked after each defender option.

Without Slice 2, analysis would identify the last one or two plies as
"lethal" only after the position has already become an open four. That is true
but not very useful: it explains the final conversion, not the tactical point
where the defender was already lost.

Slice 2 is therefore the bridge from "open four is lethal" to "this position
became lethal because all replies lead to open four or terminal coverage." It
keeps the model still bounded and inspectable without jumping directly into
general corridor proof.

## Slice 3: Analyzer Lethal Onset

Wire the classifier into replay analysis after Slice 1 and Slice 2 have focused
tests.

Replay analysis should expose three boundaries:

- terminal move: the actual five;
- lethal onset: the earliest frame in the final suffix where the loser has no
  legal reply avoiding terminal or known-lethal continuation;
- cause boundary: the earlier last escape or forced-corridor entry.

Implementation shape:

- scan replay prefix boards backward from the terminal move;
- when the loser is to move, call the shared lethal classifier for the winner;
- collect lethal evidence alongside existing proof evidence;
- show the lethal onset first in reports, then decide how much to surface in
  the in-game replay UI;
- leave search behavior unchanged.

Status: landed the first data/report slice. `GameAnalysis` now records
`lethal_onset`, analysis batch entries preserve it, and report cards surface the
prefix ply plus terminal/one-step kind. The in-game replay UI still uses the
existing corridor annotations; lethal-specific replay overlays remain a later
polish pass.

This keeps the first product value in analysis, where false positives are easy
to inspect and correct. Search integration can come later once report evidence
shows the classifier is reliable.

## Later Search Experiment

Search integration remains an experiment, not the first goal.

Possible integration:

- at leaves, treat proven lethal states as terminal-like scores;
- optionally compress open-four or 4+3 endings by one or two plies;
- record how often lethal classification changes root move choice and how much
  node/time cost it saves.

Expected benefit is modest for normal search because tactical ordering already
makes these endings narrow. The more important value is shared vocabulary and
analysis precision.
