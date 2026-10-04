# Phase 2 — Rich solution browsing

## Development branch

Phase-two development is isolated on the dedicated **`phase-2`** branch.

- Tasks 20-28, their tests, status updates, fixtures, and phase-two documentation are committed to `phase-2`.
- `main` is intentionally left as the stable phase-one line while this work is in progress.
- Agents must verify/target `phase-2` before making any phase-two change.
- Do not merge or otherwise advance `main` from phase-two work unless the user explicitly requests it.
- When Phase 2 is ready to close, Task 28 records final validation first; merge/release is a separate explicit decision.

This branch was created from the current planned Phase 2 baseline, so implementation can proceed task-by-task without using `main` as the working branch.

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
display SAN, machine UCI, comments, roles, NAGs, structured text spans, and precomputed FEN.

Keep three concepts separate:

- **live solve board** — existing graded puzzle state;
- **analysis selection** — browser open/closed, selected node, page/path;
- **display board** — selected analysis FEN while previewing, live board otherwise.

Selecting a node never advances `solution_ply`, marks solved, emits `ProgressChanged`, or
mutates the live board. Closing the browser restores the exact live solve position.

## UI direction

Keep the chessboard visible and use the lower/status area for a paginated rich-solution panel.
Prefer deterministic pages over smooth scrolling/animation.

**Tappable moves must look tappable.** Every interactive move reference is rendered as a
high-contrast move chip/button: bold move text inside a thin rectangular outline with padded
hit area. The selected move uses an unmistakable monochrome selected treatment such as
inverted fill or a heavier/double outline. Plain prose is never given this treatment.

This applies everywhere the rich solution is shown:

- moves in the main solution line;
- moves in side lines and alternative lines;
- explicitly referenced moves embedded inside descriptions or explanatory comments.

Rich prose is represented as structured spans. A `move_ref` span points to an analysis node;
tapping it selects that node and displays its precomputed FEN. A plain text span that happens
to contain something looking like SAN (for example `Nd5`) is deliberately **not** tappable.
The runtime never guesses move references from prose.

Move chips get explicit hit rectangles; variations are indented/labeled; selection is visible
without relying on color. Finger and stylus resolve to the same logical action. Stylus decoding
belongs only in the Kindle input adapter; the shared app must not know Linux pen event codes.

## PGN authoring recommendation

Use one PGN game per puzzle, `FEN`/`SetUp` where needed, and a custom `PuzzleId` tag for stable
identity. Preserve recursive annotation variations, comments, and numeric annotation glyphs.
PGN variations do not say whether a branch is another correct solution or explanatory analysis,
so non-main branches default to `sideline`. An explicit first-move comment directive
`[%role alternative]` marks an alternative branch; the converter strips that directive from
the visible comment.

Moves that are actual PGN main-line/RAV moves already have analysis nodes and are automatically
eligible for move chips in the rendered solution tree. A move merely mentioned inside prose is
interactive only when the author explicitly marks it as a reference to an analysis node/path.
Task 20 freezes the exact authoring directive and JSON span syntax; Task 21 resolves those
directives during conversion. Do not parse arbitrary SAN-looking text to infer links.

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

## Validation split

Phase two is intentionally front-loaded toward host-testable work.

- **Automated-only, no physical device required:** Tasks 20, 21, 22, 23, 24, 25, and 27.
  These can be completed by an agent unattended once their automated acceptance tests pass.
  Task 24 uses deterministic renderer snapshots; actual Scribe readability is deferred to Task 28.
- **Physical Scribe + human interaction required:** Task 26 and Task 28.
  Their host tests must pass first, but they cannot be marked Implemented until the explicit
  HUMAN CHECKPOINT in the task file has been performed and recorded.
- Task 25 deliberately stops at shared hit testing/action wiring. It does not duplicate a physical
  finger-input test because the device coordinate path is already phase-one behavior; physical
  interaction with the new move chips is exercised in Tasks 26 and 28.

This keeps device access out of tasks where it would add little evidence and concentrates human
testing on behavior that cannot be simulated faithfully: real Scribe pen events, physical touch,
e-ink readability/affordance, and final native-UI recovery.

## Task order

1. Task 20 — data contract, structured move references, and fixtures.
2. Task 21 — deterministic PGN converter.
3. Task 22 — core analysis parser/model.
4. Task 23 — non-mutating browser state.
5. Task 24 — paginated e-ink rendering with explicit move chips.
6. Task 25 — move-chip hit testing and preview wiring.
7. Task 26 — Scribe stylus taps.
8. Task 27 — safe collection regeneration/versioning.
9. Task 28 — end-to-end host/device validation.

## Non-goals

- no engine analysis/evaluation generation on Kindle;
- no general legal-move implementation in the runtime;
- no PGN parser on Kindle;
- no variation editing or handwriting on Kindle;
- no automatic conversion of SAN-looking prose into tappable moves;
- no progress change just because a line is browsed;
- no requirement to accept alternative branches during grading in phase two.
