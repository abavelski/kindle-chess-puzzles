# Task 25 — Wire move hit testing to board previews

**Status:** Implemented  
**Depends on:** Tasks 23-24  
**Primary area:** shared layout/hit testing + binary wiring  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Host geometry/action/integration tests; no device test


## Device/human-testing rule
This task must not request Kindle access. It validates logical coordinates, hit rectangles, action dispatch, and displayed FEN. The already-verified phase-one Kindle touch transform is reused; real finger/stylus interaction with these targets is deferred to Tasks 26 and 28.

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


## Implementation record

Implemented on the `phase-2` branch in the shared renderer and binary wiring.

- Added renderer-owned padded hit rectangles for every visible analysis tree move and
  structured prose `move_ref` chip. Visual chip rectangles take precedence over padding;
  if padded targets for different nodes overlap, the ambiguous padding is inert instead of
  selecting the wrong node.
- Added shared analysis hit targets for node selection, previous/next analysis pages, and
  close, all mapping directly to the Task-23 core actions. Device coordinates and Linux
  input details remain outside core.
- Added `RenderOutput::hit_test_app` so analysis geometry is resolved before the existing
  phase-one layout targets. Board-square taps are disabled while analysis preview is open,
  while existing toolbar, puzzle navigation, Refresh, X, promotion, and collection hit
  behavior remains owned by the existing layout path.
- Wired the Kindle application loop to the shared render-output hit test without changing
  finger/stylus decoding or other Kindle input behavior; stylus support remains Task 26.
- Host tests tap every visible tree and inline-reference chip across pages, verify the exact
  precomputed node FEN is displayed, exercise adjacent chip boundaries and page/close
  controls, prove plain SAN-looking prose creates no extra target, preserve old controls,
  and verify preview selection damage stays regional to the board and analysis panel.
- GitHub Actions host validation passed on CI run 157 using the repository's full
  `scripts/check.sh` gate. No physical Scribe test is required for this task.
