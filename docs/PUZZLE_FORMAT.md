# Puzzle collection format: version 1

The first Kindle milestone intentionally accepts the same version-1 puzzle JSON used by the reference Kobo application.

Collection filenames use the same convention:

- `puzzles.json` for the default collection;
- `puzzles-<name>.json` for additional collections.

The platform adapter decides the directory. Do not put a Kindle or Kobo absolute path in the parser.

For compatibility, version-1 files are initially limited to **256 KiB**, matching the reference parser.

## Example

```json
{
  "version": 1,
  "title": "Lichess sample puzzles",
  "source": "https://database.lichess.org/#puzzles",
  "license": "CC0-1.0",
  "puzzles": [
    {
      "id": "lichess-001cr",
      "fen": "8/3B2pp/p5k1/6P1/1ppp1K2/8/1P6/8 w - - 0 39",
      "description": "Solution begins with d7e8.\nFollow the line one move at a time.",
      "difficulty": 4,
      "solution": ["d7e8"],
      "source": "https://lichess.org/training/001cr"
    }
  ]
}
```

## Root fields

| Field | Required | Meaning |
| --- | --- | --- |
| `version` | Yes | Must be integer `1`. |
| `puzzles` | Yes | Non-empty ordered array. |
| `title` | No | Picker display title after trimming; blank behaves as missing. |
| other metadata | No | May be present; ignored by the current runtime unless explicitly modeled later. |

## Puzzle fields

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | Yes | Non-empty and unique within the collection. |
| `fen` | Yes | Complete six-field FEN. |
| `description` | No | String, empty, or null. Revealed on solve or by the description toggle. |
| `difficulty` | No | JSON string or number; displayed as text in the header. |
| `solution` | Yes | Non-empty array of UCI coordinate moves. |
| `source` | No | Optional provenance; ignored by the runtime. |

## FEN

The parser validates a standard six-field FEN.

The second field is the side to move:

- `w` -> White solves and normally faces the user;
- `b` -> Black solves and normally faces the user.

The app stores orientation separately from square identity. Manual Flip never changes FEN/UCI semantics.

The parser validates FEN metadata but the application remains a physical-board puzzle tool, not a legal chess engine.

## UCI move syntax

Accepted moves are:

- four characters: `e2e4`;
- five characters for promotion: `a7a8q`, `a7a8r`, `a7a8b`, `a7a8n`.

Rules:

- files are `a` through `h`;
- ranks are `1` through `8`;
- origin and destination must differ;
- promotion suffix is lowercase `q/r/b/n`.

Standard castling, if present in a stored line, uses the king coordinate move such as `e1g1`. The app compares stored coordinate strings; it does not implement chess-rule validation.

## Solution semantics

The solution array alternates plies:

- index 0: solver move;
- index 1: automatic opponent reply;
- index 2: solver move;
- index 3: automatic opponent reply;
- etc.

The user enters only solver plies.

On a wrong solver move:

- restore the exact board before the attempt;
- keep the solution cursor unchanged;
- show wrong feedback.

On a correct solver move:

- keep the move;
- advance the solution;
- apply the next stored opponent reply automatically when present;
- wait for the next solver move.

When no solution plies remain, the puzzle is complete and may be marked solved.

Promotion choice becomes part of UCI before comparison. Stored opponent promotions are applied directly without a chooser.

## Descriptions and difficulty

Difficulty is presentation metadata and never affects correctness.

Description visibility is transient UI state:

- solving reveals it automatically;
- the note/description control can reveal or hide it at any time;
- revealing it early does not solve the puzzle or advance the solution;
- reset/navigation hides it again.

## Multiple collections

Only names matching `puzzles.json` or `puzzles-<nonempty>.json` are collections.

Sort filenames lexicographically for deterministic presentation.

Use a non-empty trimmed collection `title` in the picker; otherwise show the filename.

A failed collection load must leave the current valid collection and board active.

If no collection exists, the platform may install/load bundled examples as `puzzles.json`. If user collections exist but are invalid, do not overwrite them.

## Progress format

Progress is separate from puzzle files and uses the version-1 logical shape:

```json
{
  "version": 1,
  "active_file": "puzzles-endgames.json",
  "files": {
    "puzzles.json": {
      "current_puzzle_id": "lichess-001cr",
      "solved_ids": ["lichess-001cr"]
    }
  }
}
```

Semantics:

- file identity is the collection filename;
- puzzle identity is `id`, never array index;
- solving is idempotent;
- Reset and Free Board do not clear or add solved state;
- malformed or future progress is preserved and not silently overwritten;
- a failed save keeps the newest progress in memory and surfaces a warning.

The Kindle storage path may differ from Kobo's old Cobalt key, but serialization semantics stay compatible.

## Deliberately deferred formats

Version 2 / branching alternative solution lines are not part of initial Kindle parity. Do not accept an invented version-2 schema until a dedicated task specifies it.
