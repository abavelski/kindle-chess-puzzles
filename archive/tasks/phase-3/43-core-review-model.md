# Task 43 — Add the core game-review model

**Status:** Implemented  
**Working branch:** `main` (explicit user-requested exception)  
**Depends on:** Tasks 41–42  
**Primary area:** `crates/chess-core`  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Parse review collections into a platform-neutral model and make the existing `AnalysisTree` validation reusable outside puzzles.

## Required behavior

- Add `ReviewCollection`, `ReviewGame`, typed review metadata, and a stable review-game key suitable for a flattened multi-file picker.
- Extract/generalize the analysis parser so common graph/FEN/UCI/SAN/NAG/span validation is shared.
- Preserve the puzzle-only invariant that an analysis main path exactly matches legacy `solution`.
- For reviews, validate a non-empty ordered main line without introducing a synthetic `solution`.
- Expose efficient main-line node lookup by ply, main-line membership, and nearest main-line ancestor/branch point.
- Keep all review types free of Kindle paths, FBInk, Linux input, and PGN parsing.
- Keep all existing puzzle parser public behavior and fixture results unchanged.

## Tests

Start with Task-41 fixtures. Add regressions proving puzzle analysis rejects a main-line mismatch exactly as before while review analysis accepts the same reusable tree when no puzzle solution projection is required.

## Non-goals

No workspace state, rendering, storage paths, or persistence.

## Suggested commit

`core: add validated game review model`


## Implementation record

- Added platform-neutral `ReviewCollection`, `ReviewGame`, `ReviewMetadata`, typed `ReviewResult`, and tuple-like `ReviewGameKey` types in `chess-core`.
- Reused the existing `AnalysisTree` parser for review collections by separating shared graph/FEN/UCI/SAN/NAG/span validation from puzzle-only solution projection and review-only non-empty-main-line validation.
- Cached the root-inclusive ordered main line, per-node main-line ply membership, and nearest main-line ancestor/branch point for constant-time review navigation lookups.
- Added Task-41 valid/invalid fixture coverage in Rust plus a regression proving the puzzle parser still rejects a main-line/legacy-solution mismatch with the same error while the review parser accepts the reusable tree without a synthetic solution.
- Kept PGN parsing, storage paths, workspace state, rendering, and persistence out of the core review model.
- Landed directly on `main` because the user explicitly requested a branch-policy exception.
