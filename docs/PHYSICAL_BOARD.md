# Chessnut Move — active Puzzles contract

Physical Chessnut Move integration currently lives within the combined Kindle executable and is **Puzzles-only** in the proposed split. Code seams include chess-core board_input.rs/board_session.rs, kindle-platform chessnut.rs, the runtime effect wiring and support/chessnut-move native worker.

## Preserve current behavior

- Opt-in connection and explicit manual Send of the currently visible position; no auto-connect or auto-Send on startup/wake.
- The input path waits for the physical board to reach the sent position, then follows settled ordinary moves, captures and promotion replacements for grading or Free Board.
- After a correct solver move, the authored opponent reply can be sent, then input waits for the board to match. Wrong moves require correction; navigation, manual board edits and sleep disarm input until a new Send.
- WHITE SIDE, BLACK SIDE and CHANGING SIDE physical seating transform sent positions/incoming moves consistently. Kindle Flip is an independent display choice.
- Text/notes/analysis do not disappear during board communication.

## Known limitations; do not invent acceptance

The user reported ordinary physical-board input working. Historical checks leave automatic reply motion requiring separate human acceptance. STOP was ineffective in observed tests. Castling and en passant are not recognized by the existing physical input inference. Keep those limitations explicit during refactoring.

The new Game Review binary should not include or start the Chessnut worker by default. Put product-specific board inference and session intent with Puzzles; raw native-worker transport/device mechanics can remain in a Kindle adapter. Preserve pinned worker checksums, licensing and packaging tests.

Archived evidence: [integration handoff](../archive/docs/CHESSNUT_MOVE_INTEGRATION_HANDOFF.md), [MVP verification](../archive/docs/CHESSNUT_MOVE_MVP_VERIFICATION.md), [KindleBT probe](../archive/docs/CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md). The current [native worker notes](../support/chessnut-move/README.md) remain relevant.
