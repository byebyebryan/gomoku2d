# RenjuNet Advanced Extracted Source Boards

Source: https://www.renju.net/advanced/

Legend: `B` black, `W` white, `?` visually inferred probe/question cell, `.` empty. Coordinates use `A1` through `O15`; row 15 is shown at the top, matching tactical scenario docs.

## 1. forbiddens.jpg

- Black: E13 C12 E12 B11 E11 A10 E8 F8 H8 I8 J8 B4 K4 L4 B3 D3 J3 L3 D2

- White: E10

- Manual corrections: add_white: E10

- Probe/question cells: E14, G8, L5, C3

- Raw possible label/letter neighborhoods: E10

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . ? . . . . . . . . . .  14
13  . . . . B . . . . . . . . . .  13
12  . . B . B . . . . . . . . . .  12
11  . B . . B . . . . . . . . . .  11
10  B . . . W . . . . . . . . . .  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . B B ? B B B . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . . . . . . . . . . ? . . .  5
 4  . B . . . . . . . . B B . . .  4
 3  . B ? B . . . . . B . B . . .  3
 2  . . . B . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 2. problem1.jpg

- Black: M13 G10 H10 I9 H8

- White: F6

- Probe/question cells: J10

- Raw possible label/letter neighborhoods: F6, F6

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . B . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . B B . ? . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  . . . . . . . B . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . W . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 3. problem1a.jpg

- Black: M13 G10 H10 J10 I9 H8

- White: F6

- Probe/question cells: -

- Raw possible label/letter neighborhoods: F6, F6

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . B . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . B B . B . . . . .  10
 9  . . . . . . . . B . . . . . .  9
 8  . . . . . . . B . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . W . . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 4. problem2.jpg

- Black: O14 E13 A12 E12 F12 I12 C11 L11 N11 K10 N10 J9 N9 C5 L5 C4 F4 E3 K3 M3 L2

- White: I8 N8 K5 M5 K4 M4 J2 K2 M2 N2 G1 K1 M1

- Manual corrections: add_white: K5 K1 N2

- Probe/question cells: D12, N13, D4, L3

- Raw possible label/letter neighborhoods: K5, K5, K4, J2, M1, K1, G1, K1, M1

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . B  14
13  . . . . B . . . . . . . . ? .  13
12  B . . ? B B . . B . . . . . .  12
11  . . B . . . . . . . . B . B .  11
10  . . . . . . . . . . B . . B .  10
 9  . . . . . . . . . B . . . B .  9
 8  . . . . . . . . W . . . . W .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . . . .  6
 5  . . B . . . . . . . W B W . .  5
 4  . . B ? . B . . . . W . W . .  4
 3  . . . . B . . . . . B ? B . .  3
 2  . . . . . . . . . W W B W W .  2
 1  . . . . . . W . . . W . W . .  1
    A B C D E F G H I J K L M N O
```

## 5. problem3.jpg

- Black: I9 K9 H8 J8 L8 H7 J7 L7 H6 L6

- White: G12 H11 I10 I8 K8 G7 I7 K7 M7 H5 L5

- Manual corrections: add_white: I10 I7 K7

- Probe/question cells: J9

- Raw possible label/letter neighborhoods: G12, K8, H5, L5

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . W . . . . . . . .  12
11  . . . . . . . W . . . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . . . . . B ? B . . . .  9
 8  . . . . . . . B W B W B . . .  8
 7  . . . . . . W B W B W B W . .  7
 6  . . . . . . . B . . . B . . .  6
 5  . . . . . . . W . . . W . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 6. game1.jpg

- Black: I13 F12 I12 I11 G10 J10 G9 J9 H8 J8 K8 I7 K6

- White: H13 H12 G11 F10 H10 H9 I9 K9 L9 G8 I8 J7 I6

- Probe/question cells: H11

- Raw possible label/letter neighborhoods: G11, F10, F10, H10, I10, K9

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

## 7. problem4.jpg

- Black: E8 H8 E7 E6 F6 G6

- White: E9 H9 I9

- Probe/question cells: G8

- Raw possible label/letter neighborhoods: I10

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . W . . W W . . . . . .  9
 8  . . . . B . ? B . . . . . . .  8
 7  . . . . B . . . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 8. problem4a.jpg

- Black: G9 E8 H8 E7 E6 F6 G6

- White: J11 I10 E9 F9 H9 I9 F7

- Manual corrections: add_white: F7 I10

- Probe/question cells: G8

- Raw possible label/letter neighborhoods: J11, I10, F9, M6

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

## 9. problem4b.jpg

- Black: E8 H8 E7 E6 F6 G6 E5

- White: E9 H9 I9 E4

- Probe/question cells: -

- Raw possible label/letter neighborhoods: I10

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . . . . . . . .  10
 9  . . . . W . . W W . . . . . .  9
 8  . . . . B . . B . . . . . . .  8
 7  . . . . B . . . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . B . . . . . . . . . .  5
 4  . . . . W . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 10. problem4c.jpg

- Black: G9 E8 G8 H8 E7 E6 F6 G6 E5

- White: J11 I10 E9 F9 H9 I9 F7 E4

- Manual corrections: add_white: F7 I10

- Probe/question cells: G7

- Raw possible label/letter neighborhoods: J11, I10, F9, F7

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . W . . . . .  11
10  . . . . . . . . W . . . . . .  10
 9  . . . . W W B W W . . . . . .  9
 8  . . . . B . B B . . . . . . .  8
 7  . . . . B W ? . . . . . . . .  7
 6  . . . . B B B . . . . . . . .  6
 5  . . . . B . . . . . . . . . .  5
 4  . . . . W . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 11. heavyforks.jpg

- Black: M15 B14 K14 M14 O14 L13 M13 N13 C12 E12 C11 E11 L11 B10 O10 M6 B5 C5 E5 K5 N5 D4 E4 L4 M4 D3 F3 K3 D2 J2

- White: L15 N15 L14 N14 K13 O13 D11 C10 E10 F5 C4 F4 K4 N4 O4 C3 E3 L3 D1 I1

- Manual corrections: add_white: D11 E10 N15 F5

- Probe/question cells: D12, M12, D5, M5

- Raw possible label/letter neighborhoods: N15, L14, K13, O13, C10, E10, F5, F5, F4, K4, O4, D1

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . W B W .  15
14  . B . . . . . . . . B W B W B  14
13  . . . . . . . . . . W B B B W  13
12  . . B ? B . . . . . . . ? . .  12
11  . . B W B . . . . . . B . . .  11
10  . B W . W . . . . . . . . . B  10
 9  . . . . . . . . . . . . . . .  9
 8  . . . . . . . . . . . . . . .  8
 7  . . . . . . . . . . . . . . .  7
 6  . . . . . . . . . . . . B . .  6
 5  . B B ? B W . . . . B . ? B .  5
 4  . . W B B W . . . . W B B W W  4
 3  . . W B W B . . . . B W . . .  3
 2  . . . B . . . . . B . . . . .  2
 1  . . . W . . . . W . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 12. problem5.jpg

- Black: I10 J9 K9 H8 H7 J7 K7

- White: H9 I9 I8 J8 L8 I7 H6

- Probe/question cells: K8

- Raw possible label/letter neighborhoods: J9, J8, H6

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

## 13. problem5a.jpg

- Black: I10 K10 J9 K9 H8 H7 J7 K7

- White: H9 I9 I8 J8 L8 I7 H6

- Probe/question cells: -

- Raw possible label/letter neighborhoods: J9, J8, H6

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . . . . . . . .  13
12  . . . . . . . . . . . . . . .  12
11  . . . . . . . . . . . . . . .  11
10  . . . . . . . . B . B . . . .  10
 9  . . . . . . . W W B B . . . .  9
 8  . . . . . . . B W W . W . . .  8
 7  . . . . . . . B W B B . . . .  7
 6  . . . . . . . W . . . . . . .  6
 5  . . . . . . . . . . . . . . .  5
 4  . . . . . . . . . . . . . . .  4
 3  . . . . . . . . . . . . . . .  3
 2  . . . . . . . . . . . . . . .  2
 1  . . . . . . . . . . . . . . .  1
    A B C D E F G H I J K L M N O
```

## 14. problem5b.jpg

- Black: I10 K10 J9 K9 H8 H7 J7 K7 I6

- White: H9 I9 I8 J8 L8 M8 I7 H6 I5

- Probe/question cells: K8

- Raw possible label/letter neighborhoods: J9, J8, H6

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

## 15. problem6.jpg

- Black: G11 F10 G10 E9 H9 I9 H8 J8 H7

- White: I11 G9 D8 F8 G8 I8 G7 K7 H6

- Manual corrections: add_white: K7 I11

- Probe/question cells: H10

- Raw possible label/letter neighborhoods: F8, H6

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

## 16. problem6a.jpg

- Black: H12 G11 F10 G10 H10 E9 H9 I9 H8 J8 H7

- White: I13 I11 G9 D8 F8 G8 I8 G7 K7 H6

- Manual corrections: add_white: K7 I11

- Probe/question cells: -

- Raw possible label/letter neighborhoods: F8, H6

```text
    A B C D E F G H I J K L M N O
15  . . . . . . . . . . . . . . .  15
14  . . . . . . . . . . . . . . .  14
13  . . . . . . . . W . . . . . .  13
12  . . . . . . . B . . . . . . .  12
11  . . . . . . B . W . . . . . .  11
10  . . . . . B B B . . . . . . .  10
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
