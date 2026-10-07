# Game review collection format

Task 41 freezes version 1 of the game-review JSON contract. It is intentionally parallel to the puzzle collection format while remaining a separate document type: review games are browsed, not graded.

The Kindle runtime will consume this precomputed JSON. PGN parsing, move legality, and position generation remain host-side responsibilities.

## File names and size limit

Review collections use the filename convention:

- `games.json` for the default review collection;
- `games-<name>.json` for additional collections, where `<name>` is non-empty and contains no `/` or `\`.

A version-1 review file is limited to **8 MiB (8,388,608 bytes)** measured on the raw UTF-8 file before JSON parsing. Larger libraries must be split across multiple stable collection files.

## Root document

Example:

```json
{
  "version": 1,
  "title": "Annotated classics",
  "source": "optional provenance",
  "games": []
}
```

| Field | Required | Meaning |
| --- | --- | --- |
| `version` | Yes | Integer `1`. |
| `games` | Yes | Non-empty ordered array of review games. |
| `title` | No | Collection display metadata. Must be a string when present. |
| `source` | No | Collection provenance. Must be a string when present. |
| other metadata | No | May be present and is ignored by version-1 consumers unless explicitly modeled later. |

Game array order is significant and is preserved by producers and consumers. Game IDs are the durable identity within a collection file; array indexes are not identities.

## Game object

Each game requires:

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | Yes | Stable non-empty string, unique within the file. |
| `fen` | Yes | Complete six-field starting FEN. |
| `white` | Yes | Non-empty display string. Unknown PGN values may use `?`. |
| `black` | Yes | Non-empty display string. Unknown PGN values may use `?`. |
| `result` | Yes | One of `1-0`, `0-1`, `1/2-1/2`, or `*`. |
| `event` | Yes | Non-empty display string. |
| `site` | Yes | Non-empty display string. |
| `date` | Yes | Non-empty display string; the PGN-style unknown form `????.??.??` is valid. |
| `round` | Yes | Non-empty display string; `?` is valid. |
| `source` | No | Optional provenance string. |
| `analysis` | Yes | An `analysis.version = 1` tree described below. |

Version-1 review games **must not contain `solution`**. A graded solution belongs only to puzzle data.

Unknown additional game metadata may be preserved by producers and ignored by version-1 consumers. This keeps room for future PGN headers without changing the review schema version.

## Starting FEN and PGN SetUp

`fen` is always authoritative in review JSON, including for ordinary games from the standard initial position.

When a source PGN uses `SetUp "1"` with a `FEN` header, the host converter writes that FEN into the game `fen` field. Review JSON does not need a separate `SetUp` field.

FEN syntax is the same complete six-field syntax already used by puzzle data:

1. eight ranks describing exactly eight squares each;
2. active color `w` or `b`;
3. valid castling rights or `-`;
4. a valid en-passant target for the active color or `-`;
5. a non-negative halfmove clock;
6. a positive fullmove number.

The analysis root `fen` must exactly equal the game `fen` string.

## Analysis tree

A review game reuses the existing phase-two `analysis.version = 1` node contract from `docs/PUZZLE_FORMAT.md`.

The shared invariants remain unchanged:

- `version`, `root`, and non-empty `nodes` are required;
- node IDs are non-empty and unique within the game;
- the root has the game starting FEN and no `parent`, `move`, `role`, or `nags`;
- every non-root node has exactly one declared parent and one `move` object containing exactly `uci` and `san`;
- every node has a complete six-field FEN;
- UCI uses four-coordinate characters plus an optional lowercase promotion suffix `q/r/b/n`;
- SAN is an authored non-empty display string and is not recomputed on Kindle;
- `role` is `main`, `alternative`, or `sideline`, with omitted role defaulting to `sideline`;
- omitted `comment`, `content`, `nags`, and `children` use the same phase-two defaults;
- parent/child links agree in both directions;
- child arrays are ordered, contain no duplicates, and name existing nodes;
- the graph is connected from the root and acyclic;
- structured `text` and `move_ref` spans use the same strict shapes;
- a `move_ref` may target any existing non-root node in the same analysis tree and is the only way prose becomes interactive.

Node `children` order remains authored order. Consumers must not sort it.

## Review main line

Review mode has no legacy puzzle `solution` to project against. Instead, the main line is the unique chain obtained by starting at the root and repeatedly taking the child whose normalized role is `main`.

The following rules are frozen for version 1:

- there must be **at least one `main` move** reachable from the root;
- each node on that chain may have at most one child with role `main`;
- every node marked `main` must belong to that one projected root main line;
- omitted roles are `sideline` and therefore never enter the main line;
- PREV/NEXT review navigation will follow this role-defined main line, not arbitrary child order.

A review producer should normally emit the authored PGN main continuation first in each `children` array, but main-line identity is defined by `role`, not by array position.

## Comments, NAGs, and structured references

Comments, NAGs, and structured text use the exact phase-two analysis semantics.

For example:

```json
{
  "comment": "Compare the sideline.",
  "content": [
    {"type": "text", "text": "Compare "},
    {"type": "move_ref", "node": "v1", "label": "1...c5"},
    {"type": "text", "text": "."}
  ],
  "nags": [5]
}
```

Plain SAN- or UCI-looking prose is inert. Only explicit `move_ref` spans are interactive.

## Contract fixtures

`tests/fixtures/game-review/` is the executable Task-41 corpus.

Valid fixtures:

- `valid-standard.json` — standard initial position, Unicode player/event metadata, comments, NAGs, castling, a nested variation, structured text, and move references.
- `valid-custom-fen.json` — custom `SetUp`/FEN semantics, black to move, promotion, and an alternative underpromotion.

Invalid fixtures freeze rejection categories for:

- duplicate game IDs;
- missing starting FEN;
- root-FEN mismatch;
- duplicate analysis node IDs;
- missing children;
- cyclic and disconnected graphs;
- malformed UCI;
- malformed node FEN;
- empty review main line;
- dangling `move_ref`;
- forbidden puzzle `solution` data.

`tests/test_game_review_contract.py` validates these semantics without Kindle/device code. Task 43 will later implement the production Rust parser against this already-frozen corpus.

## Puzzle compatibility

This document does not change puzzle JSON.

`docs/PUZZLE_FORMAT.md`, `tests/fixtures/puzzles.json`, and the existing rich-analysis fixtures remain authoritative for puzzle collections. Puzzle analysis keeps its existing invariant that the role-defined main path must exactly equal the puzzle `solution`; review analysis replaces only that puzzle-specific projection rule with the non-empty review main-line rule above.
