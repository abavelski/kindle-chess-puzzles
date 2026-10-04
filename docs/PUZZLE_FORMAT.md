# Puzzle collection format

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
| `topic` | No | Optional string shown above the revealed description; Cyrillic and line breaks are supported. Absent/null/non-string values have no displayed topic. |
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

Topic is presentation metadata and never affects grading. It shares description visibility
and appears above the description, including when the description itself is absent.
The starting side to move appears in the center of the header, independent of Flip.

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

## Phase-two rich analysis extension

Phase two keeps root `version` at `1` and keeps the legacy `solution` array required.
Rich data is additive: a puzzle may add `description_content` and an `analysis` object with
`analysis.version = 1`. A version-1 reader that does not know these fields may ignore them
and continue grading from `solution`.

The canonical executable example is
`tests/fixtures/rich-analysis/valid-rich.json`. The existing
`tests/fixtures/puzzles.json` is the untouched legacy-v1 compatibility control.

### Analysis object

For `analysis.version = 1`:

- `version`, `root`, and `nodes` are required.
- `version` is integer `1`.
- `root` is the ID of exactly one node in the non-empty `nodes` array.
- Node IDs are non-empty strings, unique inside one puzzle, and are not durable progress keys.
- Node order and every `children` array are significant and deterministic. Producers preserve
  PGN/source order; consumers do not sort them.

The root node represents the puzzle starting position:

- its `id` equals `analysis.root`;
- its `fen` is required and is exactly equal to the puzzle `fen` string;
- it has no `parent`, `move`, `role`, or `nags`;
- it may have `comment`, `content`, and `children`.

Every non-root node:

- has exactly one `parent` naming an existing node;
- has exactly one `move` object with exactly `uci` and `san`;
- has a complete six-field post-move `fen`;
- appears exactly once in its parent's ordered `children` array.

Parent/child links must agree in both directions. Every named child exists, the graph is
connected from the root, and it is acyclic. A child ID may not appear twice in one
`children` array or under two parents.

`move.uci` uses the legacy coordinate syntax: four characters such as `e2e4`, or five for
promotion such as `h2h1q`; origin and destination differ and promotion is lowercase
`q/r/b/n`. `move.san` is a non-empty display string generated offline. The runtime validates
FEN/UCI syntax but does not recompute legality or derive a child FEN from its parent.

### Roles and the legacy main path

A non-root `role` is one of `main`, `alternative`, or `sideline`.

Starting at the root and repeatedly taking the single child whose role is `main` produces
the only main path. There may be at most one main child at each step, and the resulting UCI
sequence must equal the puzzle's legacy `solution` array exactly, including length and
promotion suffixes. A `main` node outside that projected path is invalid.

`alternative` and `sideline` nodes are browse-only in phase two. Their presence never makes
a move accepted by grading.

### Defaults and fallback text

Omitted fields have these semantic defaults:

| Field | Applies to | Omitted value |
| --- | --- | --- |
| `role` | non-root node | `sideline` |
| `comment` | any node | empty string |
| `content` | any node | one non-interactive `text` span containing `comment` when comment is non-empty; otherwise an empty span list |
| `nags` | non-root node | empty list |
| `children` | any node | empty list |

If `content` is present, even as `[]`, it is authoritative for phase-two rich rendering
instead of the comment fallback. If puzzle `description_content` is omitted, phase-two
rendering similarly falls back to one plain `text` span containing non-empty
`description`, or to no spans when the description is absent/empty.

When present, `comment` and `description` remain the plain-text compatibility projections
for older readers. Task 21's converter emits those projections from structured content, but
a phase-two parser does not reject hand-authored input merely because a plain projection is
worded differently.

`nags`, when present, is an array of non-negative integer PGN numeric annotation glyph
numbers. Producers should emit them deterministically.

The canonical valid fixture deliberately omits `role`, `comment`, `content`, `nags`,
and `children` on one sideline node so these defaults are executable rather than implied.

### Structured text spans

Puzzle `description_content` and node `content` use the same array-of-span contract.
An empty array is valid. In analysis version 1, each span object has one of exactly two shapes:

```json
{"type":"text","text":"Compare "}
{"type":"move_ref","node":"n2","label":"1...h1=N"}
```

- `text`: allowed keys are exactly `type` and `text`; `text` is a non-empty string.
  It is always non-interactive.
- `move_ref`: allowed keys are `type`, `node`, and optional `label`. `node` is a
  non-empty ID naming an existing **non-root** node in the same puzzle. If `label` is present
  it is a non-empty string; otherwise render the target node's `move.san`.

Unknown span types, extra span keys, missing required keys, wrong JSON types, empty required
strings, dangling targets, and root targets are invalid for analysis version 1. A
`move_ref` may target any non-root node in the same analysis tree; it need not be a child or
ancestor of the node whose prose contains the span.

A `move_ref` is the **only** way prose makes a move interactive. Plain text that happens to
look like SAN or UCI remains plain text. The runtime must never regex-detect move-looking
prose and turn it into a link.

The renderer gives every `move_ref` an explicit monochrome move-chip/button treatment and a
padded hit rectangle. Plain text never receives that treatment.

### PGN/comment authoring directive for inline move references

Task 21 converts one explicit directive syntax inside author-authored prose:

```text
[%move_ref <uci-path>]
[%move_ref <uci-path>|<label>]
```

`<uci-path>` is one or more lowercase UCI moves separated by `/`, always resolved from the
analysis root. For example:

```text
Compare [%move_ref h2h1q|1...h1=Q+] with [%move_ref h2h1n].
```

Resolution walks from the root one path segment at a time. For each segment, the converter
selects the current node's child whose `move.uci` exactly equals that segment. Zero matches
is a dangling reference; more than one match is ambiguous; either is a conversion error with
puzzle/comment context. At least one segment is required, so the root cannot be referenced.

If `|<label>` is present, everything after the first `|` through the directive's closing
`]` is the display label and must be non-empty after trimming. A label cannot contain a
newline or `]`. Without a label the target node's `move.san` is used. The directive itself
does not remain visible: structured output contains a `move_ref` span, while the plain
compatibility projection substitutes the chosen label/SAN.

This syntax is intentionally based on explicitly marked UCI paths, not SAN recognition.
Unmarked prose such as `Nd5`, `h1=Q+`, or `e2e4` is never linked. The separate
`[%role alternative]` first-move directive described in Phase 2 is not a prose link and is
stripped from the visible comment by the converter.

### Contract fixtures

`tests/fixtures/rich-analysis/` contains the Task-20 contract corpus:

- `valid-rich.json`: main, sideline default, alternative, comment, NAG, black-to-move,
  promotion, structured text, and labeled/unlabeled `move_ref` coverage.
- invalid fixtures for duplicate node ID, missing child, cycle, disconnected node, root-FEN
  mismatch, invalid UCI, main-path/legacy-solution mismatch, dangling/root `move_ref`, and
  malformed spans.
- `README.md` maps each invalid fixture to the invariant it is intended to violate.

Task 22 must accept the valid fixture, reject every invalid fixture, and preserve public
semantics of the untouched `tests/fixtures/puzzles.json` legacy control.

### Collection revision and update policy

Root `version` is a schema compatibility version, not an edit counter. Optional root
`revision` is the human/content revision; Git history is authoritative. Keep collection
filenames stable when updating a book so existing progress still refers to the same
collection key.

Puzzle IDs are durable identity. Reordering puzzles or enriching comments/analysis does not
change `id`. Do not reuse an ID for a different starting position or materially different
exercise. If an edit should intentionally stop old solved state carrying forward, use a new
puzzle ID until a future progress schema explicitly models per-puzzle revisions.

Generated files may carry ignored provenance metadata such as `generated_by`,
`generated_at`, or a source SHA-256; these fields do not change runtime semantics.

The operational regeneration/rollback cookbook is in `docs/COLLECTION_UPDATES.md`.
When `tools/pgn_converter.py` targets an existing output file, it compares durable IDs and
starting FENs before replacement, reports added/removed/duplicate/FEN-changed IDs, rejects
duplicates, and performs a same-directory atomic replacement. It never reads or writes the
separate progress store.

### File-size compatibility and phase-two cap

Phase-one builds reject collection files above **256 KiB (262,144 bytes)**. Task 21 must still
report encoded output size and warn whenever generated output exceeds that old-build threshold.

Task 20 uses a reproducible representative rich-book profile in
`tests/test_rich_analysis_contract.py`: 300 puzzles with 36 analysis nodes each (10,800 nodes
total), complete FEN/UCI/SAN fields, 96-character node comments, NAG coverage, and structured
move references. UTF-8 JSON measures:

- deterministic compact encoding: **2,845,573 bytes (2.71 MiB)**;
- two-space pretty encoding: **5,214,393 bytes (4.97 MiB)**.

The phase-two runtime collection cap is therefore frozen at **8 MiB (8,388,608 bytes)**,
measured on the raw UTF-8 file before parsing. This leaves material headroom above the
representative pretty-printed book while keeping a finite parser allocation boundary.
Task 22 implements this cap. Collections above it must be split into multiple stable files;
raising the cap later is an explicit format-policy change.
