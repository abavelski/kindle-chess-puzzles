# Task 45 — Render the game-review workspace

**Status:** Ready  
**Working branch:** `game-review`  
**Depends on:** Task 44  
**Primary area:** `crates/chess-render`  
**Validation:** Automated snapshots  
**Physical Scribe required:** No

## Outcome

Render review mode with the existing board language, a header workspace toggle, review controls, and always-visible annotated movetext.

## Required behavior

- Add a small monochrome workspace icon immediately beside Refresh; show a clear selected treatment in Review mode.
- Keep puzzle mode unchanged apart from the new icon.
- In Review mode, reuse the right-side header picker slot as **GAMES**; Settings and Close remain in place.
- Render compact game identity/move context without assuming all PGN headers fit.
- Reuse dynamic toolbar placement for review controls: FREE, RESET, LOCK, FLIP.
- Render the second row as two large PREV/NEXT main-line controls; no puzzle GOTO in Review mode.
- Make the lower panel always show inline analysis movetext/variations/comments/NAGs.
- Generalize the analysis panel input so it consumes a small read-only analysis view rather than puzzle `AppState` directly; puzzle analysis and review share one renderer.
- Selected review moves use the existing monochrome selection affordance.
- Main-line navigation can focus the page containing the newly selected move; explicit page browsing still works for overflow.
- STANDARD and SMALL board modes both preserve touch dimensions; SMALL keeps the larger analysis text scale.
- Review-free-board rendering clearly reflects scratch board state without changing authored selection markers.

## Snapshots

Review: root, middle main line, final main line, selected variation, long multi-page annotations, black-to-move FEN, Unicode headers, FREE scratch, flipped orientation, STANDARD/SMALL, game picker open, and no-games message. Existing puzzle snapshots should change only where the header toggle is intentionally added.

## Non-goals

No Kindle input decoding or storage implementation.

## Suggested commit

`render: add game review workspace`
