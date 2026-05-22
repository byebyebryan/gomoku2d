# Rules And Renju

Gomoku is a five-in-a-row game. Players alternate placing stones on a grid; the
first player to make five connected stones horizontally, vertically, or
diagonally wins.

Gomoku2D currently supports two rule sets.

## Freestyle

Freestyle is the simpler version:

- Black moves first.
- Either side wins by making five or more in a row.
- There are no forbidden moves.

This is the easiest way to learn the board.

## Renju

Renju keeps the game more balanced by restricting Black. White can win normally,
but Black has forbidden moves:

- overline: more than five in a row;
- double-four: one move creates multiple four threats;
- double-three: one move creates multiple real three threats.

The important detail is "real." Gomoku2D does not rely only on rough pattern
counting for Renju. The rules model checks whether a forbidden-looking shape is
actually live under the rules, then uses that same legality model for play,
hints, bots, and replay analysis.

For implementation details and validation corpus notes, see
[`Renju Rules`](../reference/lab/renju_rules.md).
