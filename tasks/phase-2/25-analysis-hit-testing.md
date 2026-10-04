# Task 25 — Wire move hit testing to board previews

**Status:** Ready  
**Depends on:** Tasks 23-24  
**Primary area:** shared layout/hit testing + binary wiring

## Outcome
A normal UI tap on any visibly marked move chip—whether in the solution tree or embedded in a description/explanation—selects that node and displays its precomputed position.

## Required behavior
- Add stable, padded hit targets for all visible move chips from both solution-tree moves and structured prose `move_ref` spans, plus analysis-page controls.
- Convert targets to core actions; device coordinates stay outside core.
- Prevent overlapping chip/text/variation geometry selecting the wrong node.
- Prove plain text that merely resembles SAN/UCI never creates a hit target.
- While preview is open, board taps follow one explicit policy; recommended first policy is disabled.
- Damage updates board squares changed by preview plus analysis selection, not the whole screen.
- Existing board/toolbar/navigation hit targets remain unchanged.

## Tests
Host coordinate tests tap every visible solution-tree chip and inline description/comment chip in a multi-variation page, exercise page controls and chip boundaries, prove SAN-looking plain text is inert, and verify selected-node FEN becomes the displayed board.

## Non-goals
No pen decoding; Task 26 maps stylus contacts to the same logical tap path.

## Suggested commit
`render: make analysis moves tappable`
