# Tactical Scenario Corpus

Purpose: index the focused one-move scenarios used by
`gomoku-eval tactical-scenarios`.

This corpus is for tactical contract checks and diagnostics. It is not a
tournament replacement and should not be used alone for bot-strength claims.

## Runner

```sh
cd gomoku-bot-lab
cargo run --release -p gomoku-eval -- tactical-scenarios
```

Source of truth:

- boards and shared fixtures: `gomoku-bot-lab/gomoku-lab-support/src/scenarios.rs`
- case definitions: `gomoku-bot-lab/gomoku-eval/src/scenario.rs`
- vocabulary: [`../lab/tactical_shapes.md`](../lab/tactical_shapes.md)

## Roles

| Role | Meaning |
|---|---|
| `hard_safety_gate` | Must pass; protects obvious tactical safety. |
| `diagnostic` | Records behavior; useful for tuning but not a promotion gate. |

Regression cases should be merged into these roles when possible. Avoid keeping
long replay-specific fixtures when a smaller shape scenario protects the same
behavior.

## Layers

| Layer | Purpose |
|---|---|
| `local_*` | One isolated local shape fact: complete, create, react, or prevent. |
| `priority_*` | Two tactical ideas compete; ordering should choose the higher-value idea. |
| `combo_*` | One move creates or resolves multiple connected threats. |

Lethal state classification is separate. Use:

```sh
cargo run --release -p gomoku-eval -- lethal-scenarios
```

## Active Case Index

| Case | Role | Layer | Intent | Expected |
|---|---|---|---|---|
| `local_complete_open_four` | hard | local | complete open four | one terminal endpoint |
| `local_react_closed_four` | hard | local | block closed four | only completion |
| `priority_complete_open_four_over_react_closed_four` | hard | priority | win before blocking | open-four endpoint |
| `priority_prevent_open_four_over_extend_three` | hard | priority | prevent stronger threat | open-three defense |
| `priority_create_open_four_over_prevent_open_three` | diagnostic | priority | counter with stronger threat | open-four creation |
| `local_create_open_four` | diagnostic | local | create open four | open-four gain |
| `local_create_closed_four` | diagnostic | local | create closed four | closed-four gain |
| `local_create_broken_four` | diagnostic | local | create broken four | broken-four gain |
| `local_react_broken_four` | diagnostic | local | block broken four | gap/completion |
| `local_create_open_three` | diagnostic | local | create open three | open-three gain |
| `local_prevent_open_four_from_open_three` | diagnostic | local | prevent upgrade | open-three defense |
| `local_create_closed_three` | diagnostic | local | create closed three | closed-three gain |
| `local_prevent_closed_four_from_closed_three` | diagnostic | local | prevent upgrade | closed-three defense |
| `local_create_broken_three` | diagnostic | local | create broken three | valid broken-three gain |
| `local_prevent_broken_four_from_broken_three` | diagnostic | local | prevent upgrade | broken-three defense |
| `combo_create_double_threat` | diagnostic | combo | create double threat | combo gain |

The CLI owns exact board rendering and expected coordinate lists. This doc owns
the purpose and structure of the corpus.
