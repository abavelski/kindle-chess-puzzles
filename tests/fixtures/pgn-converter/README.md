# Task 21 PGN converter fixtures

- `valid-book.pgn` covers recursive variations, NAGs, comments, explicit move references,
  unmarked SAN/UCI-looking prose, an alternative-role directive, black-to-move FEN,
  promotion, capture, and castling.
- `dangling-reference.pgn` contains an explicit UCI path that does not exist.
- `ambiguous-reference.pgn` contains duplicate sibling moves so an explicit path is ambiguous.
- `invalid-illegal.pgn` contains an illegal move and must fail without partial JSON output.
- `missing-id.pgn` verifies that generated IDs require explicit opt-in.
