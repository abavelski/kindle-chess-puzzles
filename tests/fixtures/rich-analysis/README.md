# Rich-analysis contract fixtures

These fixtures freeze Task 20. The authoritative prose contract is
`docs/PUZZLE_FORMAT.md`; this directory is the executable corpus consumed by Task 22 and
later tooling tests.

`valid-rich.json` is the canonical valid collection. It covers a black-to-move position,
promotion, one-node legacy main line, an explicit alternative, a sideline whose optional
fields are omitted to exercise defaults, comments, NAGs, structured puzzle/node text, and
labeled plus unlabeled `move_ref` spans.

The untouched legacy compatibility control is `../puzzles.json`, not a copy in this
directory.

| Fixture | Intended rejection |
| --- | --- |
| `invalid-duplicate-id.json` | duplicate analysis node ID |
| `invalid-missing-child.json` | child ID does not exist |
| `invalid-cycle.json` | cycle in child graph |
| `invalid-disconnected.json` | node unreachable from root |
| `invalid-root-fen-mismatch.json` | root FEN differs from puzzle FEN |
| `invalid-uci.json` | malformed node UCI |
| `invalid-main-solution-mismatch.json` | main-role path does not equal legacy `solution` |
| `invalid-move-ref-dangling.json` | `move_ref` target does not exist |
| `invalid-move-ref-root.json` | `move_ref` targets the root |
| `invalid-span-missing-text.json` | `text` span is missing required text |
| `invalid-span-unknown-type.json` | unknown span type/shape |
| `invalid-span-bad-label.json` | `move_ref.label` has the wrong JSON type |

Error-message wording is not frozen by these files. The invariant/rejection category is.
