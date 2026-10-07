# Task 41 — Freeze the game-review data contract

**Status:** Implemented  
**Working branch:** `main` (explicit user-requested exception)  
**Depends on:** none  
**Primary area:** docs + fixtures + contract tests  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Define version-1 review collection JSON for complete games while reusing the existing `analysis.version = 1` tree contract.

## Required behavior

- Root document has `version = 1`, optional title/source metadata, and a non-empty `games` array.
- Each game has a stable non-empty unique `id`, starting `fen`, standard display metadata (White, Black, Result, Event, Site, Date, Round, optional source), and a required analysis tree.
- The nested analysis tree uses the existing node/move/FEN/NAG/comment/structured-span invariants.
- A review game has no graded `solution` field.
- Require at least one authored main-line move.
- Define deterministic ordered-main-line semantics from the root.
- Freeze filename conventions `games.json` / `games-*.json` and an 8 MiB initial per-file cap unless measurements justify another explicit limit.
- Add valid fixtures covering standard start, custom FEN/SetUp, black-to-move, comments, NAGs, nested variations, promotions/castling, and Unicode player/event names.
- Add invalid fixtures for duplicate IDs, missing/invalid root FEN, malformed graphs, invalid moves/FENs, empty main line, and dangling move references.
- Keep puzzle JSON compatibility unchanged.

## Tests

Contract validation is executable independently of the production Rust parser. `tests/test_game_review_contract.py` validates the frozen JSON shape, shared analysis-tree invariants, filename rules, and file-size limit against the Task-41 fixtures. The production Rust review parser remains Task 43.

## Non-goals

No PGN converter, AppState, rendering, storage, or persistence.

## Suggested commit

`test: freeze game review data contract`

## Implementation record

- Added `docs/GAME_REVIEW_FORMAT.md` as the authoritative version-1 review collection contract.
- Added valid and intentionally invalid fixtures under `tests/fixtures/game-review/`.
- Added `tests/test_game_review_contract.py` and wired it into `scripts/check.sh`.
- Preserved puzzle JSON, grading, progress, runtime parsing, storage, and rendering unchanged.
- This task landed directly on `main` because the user explicitly requested that branch-policy exception.
