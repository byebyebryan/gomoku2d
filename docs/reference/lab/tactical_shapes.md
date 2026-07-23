# Tactical Shapes

Purpose: define local tactical facts shared by search, corridor proof, replay
analysis, scenarios, and hints.

This doc owns local shape vocabulary only. Position-level combo/lethal coverage
lives in [`lethal_threats.md`](lethal_threats.md). Corridor state transitions
live in [`corridor_search.md`](corridor_search.md). Replay attribution lives in
[`game_analysis.md`](game_analysis.md).

Source of truth in code: `gomoku-bot-lab/gomoku-bot/src/tactical/`.

## Shape Facts

A tactical shape fact is move-centric:

- `player`: side creating the shape;
- `kind`: shape class;
- `origin`: candidate move or existing run anchor;
- `defense_squares`: points the opponent should consider;
- `rest_squares`: attacker continuation points for weaker shapes.

Raw facts are policy-neutral. Consumers decide whether a fact is active,
forcing, legal under Renju, useful for ordering, or worth showing in UI.

## Local Vocabulary

| Shape | Pattern | Meaning | Forcing |
|---|---|---|---|
| `Five` | `XXXXX` | Immediate win. | Yes |
| `OpenFour` | `.XXXX.` | Two winning completions. Usually lethal in freestyle. | Yes |
| `ClosedFour` | `OXXXX.` / `.XXXXO` | One winning completion. | Yes |
| `BrokenFour` | `XX.XX`, `X.XXX`, `XXX.X` | One winning completion through a gap. | Yes |
| `OpenThree` | `..XXX..`, `O.XXX..`, `..XXX.O` | Three that can become a two-answer or open-four threat. | Yes |
| `ClosedThree` | `OXXX.` / `.XXXO` | One-ended three. | No |
| `BrokenThree` | `.XX.X.` / `.X.XX.` | Non-contiguous three with open outside endpoints and one internal rest square. | Yes |

`O` means opponent stone, board edge, or rule-forbidden blocker. `.` means an
empty legal point in the local line.

## Broken Three Rule

Broken threes are the main subtle local case.

Active broken threes:

- `.XX.X.`
- `.X.XX.`

Non-active broken material:

- fixed-window forms such as `X.X.X`, `XX..X`, and `X..XX`;
- one-side-blocked forms such as `OXX.X.`, `.XX.XO`, `OX.XX.`, and `.X.XXO`;
- boxed forms where the continuation can only become a blockable closed four.

For an active broken three, the internal gap is the attacker rest square.
Defender replies include the internal gap and both outside endpoints.

## Consumer Policy

Search and corridor proof share raw facts but interpret them differently:

- search policy uses facts for ordering and child retention under caps;
- corridor policy filters facts into active obligations and named replies;
- UI/report hints show selected facts after policy and replay context;
- Renju legality can remove or mark Black gain, completion, or defense squares.

Closed threes stay non-forcing diagnostics. Direct open/broken threes and
compound imminent obligations are both imminent-tier threats, but compound
obligations are position-level facts, not new local shapes.

## Validation

Focused local behavior is validated by the tactical scenario corpus:

```sh
cd gomoku-bot-lab
cargo run --release -p gomoku-eval -- tactical-scenarios
```

The corpus index lives in
[`../corpora/tactical_scenarios.md`](../corpora/tactical_scenarios.md).
