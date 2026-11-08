# Task 46 — Wire review hit testing and the GAMES picker

**Status:** Implemented  
**Working branch:** `game-review`  
**Depends on:** Tasks 44–45  
**Primary area:** shared layout/hit testing + core picker actions  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Every visible review control and authored move is tappable through shared logical coordinates, and GAMES selects from a flat paginated game list.

## Required behavior

- Map the workspace icon to a core workspace-toggle action.
- Map review FREE/RESET/LOCK/FLIP and PREV/NEXT to review-specific actions where puzzle semantics differ.
- Reuse existing analysis move-chip hit rectangles for both puzzle and review views.
- Keep plain SAN/UCI-looking prose inert.
- Add a paginated GAMES modal visually consistent with the existing collection picker.
- Picker entries use stable filename + game-ID keys and compact labels derived from White/Black/Event/Date/Result.
- Invalid-file entries may display errors but cannot activate a game.
- Selecting a game closes the picker, selects the root position, resets review navigation deterministically, and does not change puzzle progress.
- Modal priority/blocking follows existing Settings/GOTO/collection rules.
- Board taps are active only in review FREE mode; otherwise authored review positions are read-only.

## Tests

Tap centers and boundaries for every review control, every visible main/variation/reference move, page controls, picker rows, invalid rows, workspace toggle, and both board orientations/sizes. Prove puzzle hit targets remain unchanged apart from the added workspace icon.

## Non-goals

No Linux/evdev changes and no physical stylus checkpoint.

## Suggested commit

`render: wire game review controls and picker`
