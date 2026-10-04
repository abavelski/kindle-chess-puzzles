# Task 25 — Wire move hit testing to board previews

**Status:** Ready  
**Depends on:** Tasks 23-24  
**Primary area:** shared layout/hit testing + binary wiring

## Outcome
A normal UI tap on any visible analysis move selects that node and displays its precomputed position.

## Required behavior
- Add stable hit targets for visible move tokens and analysis-page controls.
- Convert targets to core actions; device coordinates stay outside core.
- Prevent overlapping text/variation geometry selecting the wrong node.
- While preview is open, board taps follow one explicit policy; recommended first policy is disabled.
- Damage updates board squares changed by preview plus analysis selection, not the whole screen.
- Existing board/toolbar/navigation hit targets remain unchanged.

## Tests
Host coordinate tests tap every visible move in a multi-variation page, page controls, token boundaries, outside regions, and verify selected-node FEN becomes the displayed board.

## Non-goals
No pen decoding; Task 26 maps stylus contacts to the same logical tap path.

## Suggested commit
`render: make analysis moves tappable`
