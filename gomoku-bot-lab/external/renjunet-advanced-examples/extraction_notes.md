# RenjuNet Advanced Complex Case Review

Source: https://www.renju.net/advanced/

This is a review aid for extracting fixture candidates. The raw corrected
per-image board dump is in [`extracted_boards.md`](extracted_boards.md); the
split fixture candidates are promoted in
[`../../../docs/reference/corpora/renju_corpus.md`](../../../docs/reference/corpora/renju_corpus.md). Coordinate
overlays were temporary extraction artifacts and are not tracked.

Important caveat: the page uses hollow circles and small marks in ways that are
not always semantically distinguishable by the image pass. The fixture candidate
draft includes manual corrections from visual review.

The markdown board dumps mark probe/question cells with `?`.

The fixture candidate boards clear source stones outside the focus window, then
add count filler stones outside that window so each probe is a legal Black turn
position (`B == W` before the `?` move). Fillers are listed explicitly in each
candidate.

## Ordered Images

1. `forbiddens.jpg` - overview of common forbidden points, including one non-forbidden apparent 3x3.
2. `problem1.jpg` - asks whether point A is forbidden.
3. `problem1a.jpg` - proof frame for problem 1; removed from fixture candidates.
4. `problem2.jpg` - split into four independent questions E/F/G/H.
5. `problem3.jpg` - asks whether point I is legal/forbidden under secondary forbidden-point effects.
6. `game1.jpg` - real-game context involving point J.
7. `problem4.jpg` - prefix frame for the actual proof sequence; removed from fixture candidates.
8. `problem4a.jpg` - white executes the trap.
9. `problem4b.jpg` - proof frame; removed from fixture candidates.
10. `problem4c.jpg` - explanation position where point 17 is legal because one line is dead.
11. `heavyforks.jpg` - split into four independent complex fork questions N/O/P/Q.
12. `problem5.jpg` - white aims at forbidden 3x3 point R.
13. `problem5a.jpg` - proof frame; removed from fixture candidates.
14. `problem5b.jpg` - white converts the situation into an illegal 4x3x3 at V.
15. `problem6.jpg` - white attacks forbidden 4x4x3 point W.
16. `problem6a.jpg` - black uses a forcing line to make one line dead, then plays a legal 4x3.

## Split Cases To Validate First

### `problem2.jpg`

Source expectation:

- E: legal. Not a forbidden 3x3 because the horizontal three is not an open three.
- F: legal. Not a forbidden 4x4 because one diagonal four cannot become a winning five due to overline.
- G: forbidden. A real 3x3.
- H: forbidden. A real 3x3.

Extraction note:

- The image contains four separate target labels; the rough OCR/label neighborhoods are noisy.
- Probe cells are `D12`, `N12`, `D4`, and `L3`.
- Manual missing whites added at `K5`, `K1`, and `N2`.

### `heavyforks.jpg`

Source expectation:

- N: forbidden 3x3x3.
- O: forbidden 4x4x4.
- P: forbidden 4x4x3.
- Q: forbidden 4x3x3.

Extraction note:

- This is one image containing four independent fork examples.
- Probe cells are `D12`, `M12`, `D5`, and `M5`.
- Manual missing whites added at `D11`, `E10`, `N15`, and `F5`.

## Candidate Count

The current split draft contains 23 fixture candidates:

- 4 from `forbiddens.jpg`;
- 1 from `problem1.jpg`;
- 4 from `problem2.jpg`;
- 1 from `problem3.jpg`;
- 2 from `game1.jpg`, including one synthetic frame after placing black at `H11`;
- 1 from `problem4a.jpg`;
- 2 from `problem4c.jpg`, including one synthetic frame after removing black at `G8`;
- 4 from `heavyforks.jpg`;
- 1 from `problem5.jpg`;
- 1 from `problem5b.jpg`;
- 1 from `problem6.jpg`;
- 1 from `problem6a.jpg`, after removing black at `H10`.

## Other High-Value Cases

- `problem1.jpg` / `problem1a.jpg`: apparent 3x3 that is legal because one three cannot become an open four.
- `problem3.jpg`: apparent 3x3 where one line is dead because the needed endpoints are themselves forbidden.
- `problem4*.jpg`: trap and anti-trap sequence around a forbidden 3x3, including an artificial block/dead-three explanation.
- `problem5*.jpg`: dynamic transformation where a legal-looking 4x3 becomes illegal 4x3x3 after forced play.
- `problem6*.jpg`: black saves itself by first making a line dead, then playing a legal 4x3.
