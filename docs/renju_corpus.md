# Renju Rule Corpus

Purpose: define the golden Renju legality cases that Gomoku2D tests against.
This corpus combines a small handwritten core set with extracted RenjuNet
advanced tutorial cases.

Legend: `B` black, `W` white, `?` probe/question point, `.` empty. Boards use
full 15x15 tactical scenario coordinates (`A1` bottom-left, `O15` top-right).

The executable runner is:

```sh
cargo run -p gomoku-eval -- renju-rules \
  --report-json outputs/renju-rule-fixtures.json
```

Current corpus size: 29 cases.

## Handwritten Core Cases

These six cases cover the basic local contract before the extracted advanced
examples:

| Case | Candidate | Expected | Purpose |
| --- | --- | --- | --- |
| `black_exact_five_legal` | Black `I8` | legal | Black exact five wins before forbidden checks matter. |
| `black_overline_forbidden` | Black `E1` | forbidden | Black six-in-row without exact five is forbidden. |
| `black_double_four_forbidden` | Black `H8` | forbidden | Two real fours are forbidden. |
| `black_double_three_forbidden` | Black `H8` | forbidden | Two real threes are forbidden. |
| `black_four_plus_three_legal` | Black `H8` | legal | `4+3` is legal for Black. |
| `white_double_three_unrestricted` | White `H8` | legal | White is not restricted by forbidden-shape rules. |

### black_exact_five_legal

- Source: `rif`
- Candidate: Black `I8`
- Expected: legal

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . B B B B ? . . . . . .  8
 7  W . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  W . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### black_overline_forbidden

- Source: `rif`
- Candidate: Black `E1`
- Expected: forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . W . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . W . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . W . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . W . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . W . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  B B B B ? B . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### black_double_four_forbidden

- Source: `project_regression`
- Candidate: Black `H8`
- Expected: forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  W . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  W . . . . . . . . . . . . . .  9
 8  . . . B B B . ? . . . . . . .  8
 7  W . . . . . . B . . . . . . .  7
 6  . . . . . . . B . . . . . . .  6
 5  W . . . . . . B . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  W . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### black_double_three_forbidden

- Source: `project_regression`
- Candidate: Black `H8`
- Expected: forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . B B ? . . . . . . .  8
 7  W . . . . . . B . . . . . . .  7
 6  . . . . . . . B . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  W . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### black_four_plus_three_legal

- Source: `project_regression`
- Candidate: Black `H8`
- Expected: legal

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  W . . . . . . . . . . . . . .  9
 8  . . . . B B B ? . . . . . . .  8
 7  W . . . . . . B . . . . . . .  7
 6  . . . . . . . B . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  W . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  W . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### white_double_three_unrestricted

- Source: `rif`
- Candidate: White `H8`
- Expected: legal

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . W W ? . . . . . . .  8
 7  B . . . . . . W . . . . . . .  7
 6  . . . . . . . W . . . . . . .  6
 5  B . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  B . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  B . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## RenjuNet Advanced Cases

Source: <https://www.renju.net/advanced/>

The following 23 cases are extracted from RenjuNet's advanced forbidden-move
tutorial. The source extraction and external validation wrapper live in
[`../gomoku-bot-lab/external/renjunet-advanced-examples/`](../gomoku-bot-lab/external/renjunet-advanced-examples/).

Removed proof frames: `problem1a.jpg`, `problem4.jpg`, `problem4b.jpg`, `problem5a.jpg`. Synthetic frames are called out in their expected text.

### 01a_forbiddens_4x4

- Source: `forbiddens.jpg`
- Focus window: `A-E,10-15`
- Probe: `E14`
- Expected: 4x4 forbidden
- Count filler white: O8 O11 O5 O2 O15

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . W  15
14  . . . . ? . . . . . . . . . .  14
13  . . . . B . . . . . . . . . .  13
12  . . B . B . . . . . . . . . .  12
11  . B . . B . . . . . . . . . W  11
10  B . . . W . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . W  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . W  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . W  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 01b_forbiddens_overline_6

- Source: `forbiddens.jpg`
- Focus window: `E-J,8-8`
- Probe: `G8`
- Expected: 6 forbidden
- Count filler white: H1 H15 E1 E15 K1

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . B B ? B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . W . . W . . W . . . .  1
    A B C D E F G H I J K L M N O
```

### 01c_forbiddens_3x3

- Source: `forbiddens.jpg`
- Focus window: `J-L,3-5`
- Probe: `L5`
- Expected: 3x3 forbidden
- Count filler white: H15 E15 K15 N15

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . W . . W .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . ? . . .  5
 4  . . . . . . . . . . B B . . .  4
 3  . . . . . . . . . B . B . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 01d_forbiddens_false_3x3_dead_diag

- Source: `forbiddens.jpg`
- Focus window: `B-D,2-4`
- Probe: `C3`
- Expected: not forbidden; diagonal three is not open
- Count filler white: H15 O8 E15 K15

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . W . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . W  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . B . . . . . . . . . . . . .  4
 3  . B ? B . . . . . . . . . . .  3
 2  . . . B . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 02_problem1_false_3x3_dead_diag

- Source: `problem1.jpg`
- Focus window: `F-M,6-13`
- Probe: `J10`
- Expected: not forbidden; diagonal three is not real
- Count filler white: A8 H1 A11 A5

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . B . .  13
12  . . . . . . . . . . . . . . .  12
11  W . . . . . . . . . . . . . .  11
10  . . . . . . B B . ? . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  W . . . . . . B . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . W . . . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . W . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 04a_problem2_false_3x3_dead_horizontal

- Source: `problem2.jpg`
- Focus window: `A-I,11-13`
- Probe: `D12`
- Expected: not forbidden; horizontal three is not real
- Count filler white: H1 E1 K1 B1 N1 H4

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . B . . . . . . . . . .  13
12  B . . ? B B . . B . . . . . .  12
11  . . B . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . W . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . W . . W . . W . . W . . W .  1
    A B C D E F G H I J K L M N O
```

### 04b_problem2_false_4x4_dead_diag

- Source: `problem2.jpg`
- Focus window: `I-O,8-14`
- Probe: `N13`
- Expected: not forbidden; diagonal four is not real
- Synthetic remove black: I12
- Count filler white: A8 A11 A5 A14 A2

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  W . . . . . . . . . . . . . B  14
13  . . . . . . . . . . . . . ? .  13
12  . . . . . . . . . . . . . . .  12
11  W . . . . . . . . . . B . B .  11
10  . . . . . . . . . . B . . B .  10
 9  . . . . . . . . . B . . . B .  9
 8  W . . . . . . . W . . . . W .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  W . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 04c_problem2_3x3

- Source: `problem2.jpg`
- Focus window: `C-G,1-5`
- Probe: `D4`
- Expected: 3x3 forbidden
- Count filler white: H15 E15 K15

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . W . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . B . . . . . . . . . . . .  5
 4  . . B ? . B . . . . . . . . .  4
 3  . . . . B . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . W . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 04d_problem2_3x3

- Source: `problem2.jpg`
- Focus window: `J-N,1-5`
- Probe: `L3`
- Expected: 3x3 forbidden
- Count filler black: H15 E15 K15 B15 N15 A8

```text
    A B C D E F G H I J K L M N O
15  . B . . B . . B . . B . . B .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  B . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . W B W . .  5
 4  . . . . . . . . . . W . W . .  4
 3  . . . . . . . . . . B ? B . .  3
 2  . . . . . . . . . W W B W W .  2
 1  . . . . . . . . . . W . W . .  1
    A B C D E F G H I J K L M N O
```

### 05_problem3_false_3x3

- Source: `problem3.jpg`
- Focus window: `G-M,5-12`
- Probe: `J9`
- Expected: not forbidden
- Count filler black: A8

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . W . . . . . . . .  12
11  . . . . . . . W . . . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . . . . . B ? B . . . .  9
 8  B . . . . . . B W B W B . . .  8
 7  . . . . . . W B W B W B W . .  7
 6  . . . . . . . B . . . B . . .  6
 5  . . . . . . . W . . . W . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 06a_game1_false_3x3

- Source: `game1.jpg`
- Focus window: `F-L,6-13`
- Probe: `H11`
- Expected: not forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . W B . . . . . .  13
12  . . . . . B . W B . . . . . .  12
11  . . . . . . W ? B . . . . . .  11
10  . . . . . W B W . B . . . . .  10
 9  . . . . . . B W W B W W . . .  9
 8  . . . . . . W B W B B . . . .  8
 7  . . . . . . . . B W . . . . .  7
 6  . . . . . . . . W . B . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 06b_game1_after_h11_4x4

- Source: `game1.jpg`
- Focus window: `F-L,6-13`
- Probe: `I10`
- Expected: 4x4 forbidden after placing black at H11
- Synthetic add black: H11
- Count filler white: A8

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . W B . . . . . .  13
12  . . . . . B . W B . . . . . .  12
11  . . . . . . W B B . . . . . .  11
10  . . . . . W B W ? B . . . . .  10
 9  . . . . . . B W W B W W . . .  9
 8  W . . . . . W B W B B . . . .  8
 7  . . . . . . . . B W . . . . .  7
 6  . . . . . . . . W . B . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 08_problem4a_3x3

- Source: `problem4a.jpg`
- Focus window: `E-J,6-11`
- Probe: `G8`
- Expected: 3x3 forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . W . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . W W B W W . . . . . .  9
 8  . . . . B . ? B . . . . . . .  8
 7  . . . . B W . . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 10a_problem4c_4x4

- Source: `problem4c.jpg`
- Focus window: `E-J,4-11`
- Probe: `G7`
- Expected: 4x4 forbidden
- Count filler white: O8

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . W . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . W W B W W . . . . . .  9
 8  . . . . B . B B . . . . . . W  8
 7  . . . . B W ? . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . B . . . . . . . . . .  5
 4  . . . . W . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 10b_problem4c_without_g8_false_3x3

- Source: `problem4c.jpg`
- Focus window: `E-J,4-11`
- Probe: `G8`
- Expected: not forbidden after removing black at G8
- Synthetic remove black: G8

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . W . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . W W B W W . . . . . .  9
 8  . . . . B . ? B . . . . . . .  8
 7  . . . . B W . . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . B . . . . . . . . . .  5
 4  . . . . W . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 11a_heavyforks_3x3x3

- Source: `heavyforks.jpg`
- Focus window: `B-E,10-14`
- Probe: `D12`
- Expected: 3x3x3 forbidden
- Count filler white: O8 O11 O5

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . B . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . B ? B . . . . . . . . . .  12
11  . . B W B . . . . . . . . . W  11
10  . B W . W . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . W  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . . . . W  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 11b_heavyforks_4x4x4

- Source: `heavyforks.jpg`
- Focus window: `K-O,10-15`
- Probe: `M12`
- Expected: 4x4x4 forbidden
- Count filler white: A8 A11 A5

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . W B W .  15
14  . . . . . . . . . . B W B W B  14
13  . . . . . . . . . . W B B B W  13
12  . . . . . . . . . . . . ? . .  12
11  W . . . . . . . . . . B . . .  11
10  . . . . . . . . . . . . . . B  10
 9  . . . . . . . . . . . . . . .  9
 8  W . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  W . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 11c_heavyforks_4x4x3

- Source: `heavyforks.jpg`
- Focus window: `B-F,1-5`
- Probe: `D5`
- Expected: 4x4x3 forbidden
- Count filler white: H15 E15

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . B B ? B W . . . . . . . . .  5
 4  . . W B B W . . . . . . . . .  4
 3  . . W B W B . . . . . . . . .  3
 2  . . . B . . . . . . . . . . .  2
 1  . . . W . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 11d_heavyforks_4x3x3

- Source: `heavyforks.jpg`
- Focus window: `I-O,1-6`
- Probe: `M5`
- Expected: 4x3x3 forbidden
- Count filler white: H15 E15

```text
    A B C D E F G H I J K L M N O
15  . . . . W . . W . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . B . .  6
 5  . . . . . . . . . . B . ? B .  5
 4  . . . . . . . . . . W B B W W  4
 3  . . . . . . . . . . B W . . .  3
 2  . . . . . . . . . B . . . . .  2
 1  . . . . . . . . W . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 12_problem5_3x3

- Source: `problem5.jpg`
- Focus window: `H-M,6-10`
- Probe: `K8`
- Expected: 3x3 forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . B . . . . . .  10
 9  . . . . . . . W W B B . . . .  9
 8  . . . . . . . B W W ? W . . .  8
 7  . . . . . . . B W B B . . . .  7
 6  . . . . . . . W . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 14_problem5b_4x3x3

- Source: `problem5b.jpg`
- Focus window: `H-M,5-10`
- Probe: `K8`
- Expected: 4x3x3 forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . B . B . . . .  10
 9  . . . . . . . W W B B . . . .  9
 8  . . . . . . . B W W ? W W . .  8
 7  . . . . . . . B W B B . . . .  7
 6  . . . . . . . W B . . . . . .  6
 5  . . . . . . . . W . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 15_problem6_4x4x3

- Source: `problem6.jpg`
- Focus window: `D-K,6-11`
- Probe: `H10`
- Expected: 4x4x3 forbidden

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . B . W . . . . . .  11
10  . . . . . B B ? . . . . . . .  10
 9  . . . . B . W B B . . . . . .  9
 8  . . . W . W W B W B . . . . .  8
 7  . . . . . . W B . . W . . . .  7
 6  . . . . . . . W . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

### 16_problem6a_without_h10_false

- Source: `problem6a.jpg`
- Focus window: `D-K,6-13`
- Probe: `H10`
- Expected: not forbidden after removing black at H10
- Synthetic remove black: H10

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . W . . . . . .  13
12  . . . . . . . B . . . . . . .  12
11  . . . . . . B . W . . . . . .  11
10  . . . . . B B ? . . . . . . .  10
 9  . . . . B . W B B . . . . . .  9
 8  . . . W . W W B W B . . . . .  8
 7  . . . . . . W B . . W . . . .  7
 6  . . . . . . . W . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```
