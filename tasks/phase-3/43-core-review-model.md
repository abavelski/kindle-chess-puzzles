# Task 43 — Add the core game-review model

**Status:** Ready  
**Working branch:** `game-review`  
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
