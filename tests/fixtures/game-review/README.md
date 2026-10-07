# Game-review contract fixtures

These fixtures freeze Task 41. The authoritative prose contract is
`docs/GAME_REVIEW_FORMAT.md`; this directory is the executable corpus used by
`tests/test_game_review_contract.py` and later review parser/converter tasks.

Valid fixtures:

- `valid-standard.json`: standard initial FEN, Unicode metadata, comments, NAGs,
  castling, structured move references, and a nested variation.
- `valid-custom-fen.json`: custom PGN `SetUp`/FEN semantics, black to move,
  promotion, and an alternative underpromotion.

Invalid fixtures intentionally violate one contract rule each:

| Fixture | Intended rejection |
| --- | --- |
| `invalid-duplicate-game-id.json` | duplicate game ID within one collection |
| `invalid-missing-fen.json` | missing game starting FEN |
| `invalid-root-fen-mismatch.json` | analysis root FEN differs from game FEN |
| `invalid-duplicate-node-id.json` | duplicate analysis node ID |
| `invalid-missing-child.json` | child ID does not exist |
| `invalid-cycle.json` | cycle in the analysis graph |
| `invalid-disconnected.json` | node is unreachable from the root |
| `invalid-uci.json` | malformed node UCI |
| `invalid-node-fen.json` | malformed node FEN |
| `invalid-empty-main-line.json` | no `main` move is reachable from the root |
| `invalid-move-ref-dangling.json` | structured move reference targets a missing node |
| `invalid-solution-field.json` | review game contains forbidden puzzle `solution` data |

Error-message wording is not frozen. The invariant/rejection category is.
