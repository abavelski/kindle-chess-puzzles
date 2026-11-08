# Task 44 — Add the review workspace state machine

**Status:** Ready  
**Working branch:** `game-review`  
**Depends on:** Task 43  
**Primary area:** `crates/chess-core/src/app.rs` + review state  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Add deterministic top-level Puzzles/Review switching plus review selection, main-line navigation, picker state, and reversible Free Board scratch state.

## Required behavior

- Add explicit workspace state: Puzzles or Review.
- Entering Review with no valid games available must leave Puzzles active and surface a transient message.
- Switching workspaces preserves the complete existing puzzle session: puzzle index, live board/selection, grading cursor, feedback, analysis preview/page, orientation, and durable progress.
- Review state tracks active game, selected analysis node/root, main-line cursor, analysis page/focus state, orientation, and game-picker state.
- PREV/NEXT traverse only the authored main line and have tested root/end disabled states.
- Selecting a main-line node synchronizes the main-line cursor.
- Selecting a variation previews its exact FEN and anchors main-line navigation to the nearest main-line ancestor/branch point.
- Review selection never emits `ProgressChanged`.
- Review FREE clones the selected authored position into a scratch board.
- While FREE is enabled, PREV/NEXT or move selection changes the authored selection and resets scratch to that new position while remaining in FREE.
- Review RESET restores scratch to the selected authored position; disabling FREE restores authored position.
- Review actions must not mutate source game data.

## Tests

Cover repeated workspace switching around in-progress correct/wrong puzzle attempts, open puzzle analysis, selected variation positions, solved progress bytes, review root/main/variation navigation, FREE/reset behavior, game switching, and modal blocking.

## Non-goals

No renderer/layout, storage paths, PGN parsing, or durable review resume file.

## Suggested commit

`core: add game review workspace state`
