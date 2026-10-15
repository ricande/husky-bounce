# Game rules

Husky Bounce mostly plays itself: it starts with one ball and ends in chaos.

- Balls bounce inside rotating rings under gravity. Every bounce plays a
  note (up and down an eight-note scale, at most one note per 45 ms) and may
  spawn a new ball in the middle.
- Each ring has a gap. A ball that slips through moves out to the next ring;
  through the outermost ring it escapes, falls off the screen and scores one
  point ("Escaped").
- A level lasts 179 seconds (`LEVEL_SECONDS`), a little under 3:00, the
  longest YouTube Short, so the whole level fits in one. The clock at the top
  counts down.
- The chaos builds steadily: the field may hold at most
  1 + 399 × (time / 179 s)² balls, so about 40 after one minute, 180 after
  two and 400 at the end. The rings spin up to four times faster and their
  gaps widen, so the swarm pours out at the end.
- The field is never empty: when the last ball has escaped, a new one starts
  in the middle.
- Level 1 has one ring; each level adds one, up to four. Every level has its
  own fixed random seed, so it plays the same every time.
- Input is optional: Space or a click adds one ball (up to 400). Esc or P
  pauses.
- The score on the results screen is the number of escaped balls.
