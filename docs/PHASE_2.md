# Phase 2 — Rich solution browsing

## Goal

Support book-style solutions containing main lines, side lines, alternative lines, annotations,
and explanations. Tapping a rendered move with a finger or stylus must show the exact board
position after that move.

## Architectural decision

**PGN is an offline authoring/import format, not a Kindle runtime format.** A host-side
converter uses a real chess library to parse PGN, walk variations, compute SAN/UCI, validate
moves, and emit deterministic JSON. Every analysis node stores its complete post-move FEN.
The Kindle therefore needs neither a PGN parser nor a chess engine to jump to a variation.

Use the additive version-1 extension in `PUZZLE_FORMAT.md`: retain the current `solution` main
line and add `analysis.version = 1` with a flat tree. Root version 2 is reserved for a future
incompatible change.

## Runtime state

Normalize rich JSON into a core `AnalysisTree` containing node IDs/indexes, parent/children,
display SAN, machine UCI, comments, roles, NAGs, and precomputed FEN.

Keep three concepts separate:

- **live solve board** — existing graded puzzle state;
- **analysis selection** — browser open/closed, selected node, page/path;
- **display board** — selected analysis FEN while previewing, live board otherwise.

Selecting a node never advances `solution_ply`, marks solved, emits `ProgressChanged`, or
mutates the live board. Closing the browser restores the exact live solve position.

## UI direction

Keep the chessboard visible and use the lower/status area for a paginated rich-solution panel.
Prefer deterministic pages over smooth scrolling/animation. Move tokens get explicit hit
rectangles; variations are indented/labeled; selection is visible without relying on color.
Tapping a visible move selects its node and refreshes the board.

Finger and stylus resolve to the same logical action. Stylus decoding belongs only in the
Kindle input adapter; the shared app must not know Linux pen event codes.

## PGN authoring recommendation

Use one PGN game per puzzle, `FEN`/`SetUp` where needed, and a custom `PuzzleId` tag for stable
identity. Preserve recursive annotation variations, comments, and numeric annotation glyphs.
PGN variations do not say whether a branch is another correct solution or explanatory analysis,
so non-main branches default to `sideline`. An explicit first-move comment directive
`[%role alternative]` marks an alternative branch; the converter strips that directive from
the visible comment.

Converter output must be deterministic: identical PGN plus options produces byte-stable JSON
apart from explicitly enabled timestamp metadata.

## JSON update/versioning policy

- Root `version`: schema major; remain `1` in phase two.
- `analysis.version`: extension schema; start at `1`.
- Root `revision`: optional content revision for releases/imports.
- Puzzle `id`: durable progress identity; stable for the same exercise.
- Filename: stable across normal updates so the collection progress key survives.
- Git: authoritative historical versioning and rollback.

If a starting position or exercise meaning changes enough that old solved state should not
carry forward, assign a new puzzle ID. Do not add automatic solved-state invalidation unless
real use demonstrates a need.

## Task order

1. Task 20 — data contract and fixtures.
2. Task 21 — deterministic PGN converter.
3. Task 22 — core analysis parser/model.
4. Task 23 — non-mutating browser state.
5. Task 24 — paginated e-ink rendering.
6. Task 25 — move hit testing and preview wiring.
7. Task 26 — Scribe stylus taps.
8. Task 27 — safe collection regeneration/versioning.
9. Task 28 — end-to-end host/device validation.

Task 10 remains independent post-phase-one display maintenance and must not be mixed with
phase-two product work.

## Non-goals

- no engine analysis/evaluation generation on Kindle;
- no general legal-move implementation in the runtime;
- no PGN parser on Kindle;
- no variation editing or handwriting on Kindle;
- no progress change just because a line is browsed;
- no requirement to accept alternative branches during grading in phase two.
