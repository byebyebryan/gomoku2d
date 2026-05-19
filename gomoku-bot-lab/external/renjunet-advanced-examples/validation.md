# RenjuNet Advanced Reference Check

Fixture cases are generated from `fixtures.json`. Expected labels are the current manual labels.

Checkers:
- `piskvork`: C++ Renju foul checker cloned from `https://github.com/wind23/piskvork_renju.git` at `f76a43afb67861883c86f8bd22b1a4957c27f068`.

| Fixture | Probe | Expected | Piskvork | Ref verdict |
| --- | --- | --- | --- | --- |
| `01a_forbiddens_4x4` | `E14` | forbidden | forbidden (double-four) | ref matches expected |
| `01b_forbiddens_overline_6` | `G8` | forbidden | forbidden (overline) | ref matches expected |
| `01c_forbiddens_3x3` | `L5` | forbidden | forbidden (double-three) | ref matches expected |
| `01d_forbiddens_false_3x3_dead_diag` | `C3` | legal | legal (legal) | ref matches expected |
| `02_problem1_false_3x3_dead_diag` | `J10` | legal | legal (legal) | ref matches expected |
| `04a_problem2_false_3x3_dead_horizontal` | `D12` | legal | legal (legal) | ref matches expected |
| `04b_problem2_false_4x4_dead_diag` | `N13` | legal | legal (legal) | ref matches expected |
| `04c_problem2_3x3` | `D4` | forbidden | forbidden (double-three) | ref matches expected |
| `04d_problem2_3x3` | `L3` | forbidden | forbidden (double-three) | ref matches expected |
| `05_problem3_false_3x3` | `J9` | legal | legal (legal) | ref matches expected |
| `06a_game1_false_3x3` | `H11` | legal | legal (legal) | ref matches expected |
| `06b_game1_after_h11_4x4` | `I10` | forbidden | forbidden (double-four) | ref matches expected |
| `08_problem4a_3x3` | `G8` | forbidden | forbidden (double-three) | ref matches expected |
| `10a_problem4c_4x4` | `G7` | forbidden | forbidden (double-four) | ref matches expected |
| `10b_problem4c_without_g8_false_3x3` | `G8` | legal | legal (legal) | ref matches expected |
| `11a_heavyforks_3x3x3` | `D12` | forbidden | forbidden (double-three) | ref matches expected |
| `11b_heavyforks_4x4x4` | `M12` | forbidden | forbidden (double-four) | ref matches expected |
| `11c_heavyforks_4x4x3` | `D5` | forbidden | forbidden (double-four) | ref matches expected |
| `11d_heavyforks_4x3x3` | `M5` | forbidden | forbidden (double-three) | ref matches expected |
| `12_problem5_3x3` | `K8` | forbidden | forbidden (double-three) | ref matches expected |
| `14_problem5b_4x3x3` | `K8` | forbidden | forbidden (double-three) | ref matches expected |
| `15_problem6_4x4x3` | `H10` | forbidden | forbidden (double-four) | ref matches expected |
| `16_problem6a_without_h10_false` | `H10` | legal | legal (legal) | ref matches expected |

## Summary

- Fixtures: 23
- Piskvork matches expected: 23/23
- External reference matches expected: 23/23
- External reference disagrees with expected: 0/23

Piskvork is GPL-licensed external executable evidence. The wrapper compiles it outside the repo and does not copy reference code into Gomoku2D.
