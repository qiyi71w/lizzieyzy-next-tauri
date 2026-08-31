# Winrate chart uses one series: Black or selected side to play

The Java winrate graph offered Black perspective as one curve and Both perspective as complementary Black and White curves. Next `ANA-11` keeps a user-visible Graph Perspective control, but the second value converts the **entire single series** to the selected node's side to play instead of drawing two curves.

Dual complementary series were rejected because a side-to-play flip stays readable as one encoding, matches score-lead sign, and avoids treating color changes as Blunder Bar swings. Per-move sawtooth side-to-play was rejected for the same Blunder Bar reason.
