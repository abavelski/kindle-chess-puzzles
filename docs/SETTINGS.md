# Settings architecture

> **Split constraint:** preserve both existing workspace settings profiles and schema version checks. The future standalone Review app will have its own settings root and a tested non-destructive migration ([SPLIT-09](NEXT_TASKS_APP_SPLIT.md)). The split is not implemented yet.


Settings are application preferences, not puzzle data and not learning progress.

## Current settings

Version 1 starts deliberately small:

- **Free mode button** — show or hide the `FREE` toolbar control.
- **Notes button** — show or hide the `NOTE` toolbar control.
- **Lock button** — show or hide the `LOCK` toolbar control.
- **Flip button** — show or hide the `FLIP` toolbar control.
- **Board size** — choose STANDARD or SMALL.
- **Physical board** (Puzzles only) — WHITE SIDE, BLACK SIDE or CHANGING SIDE.
  This seating preference is global; it is separate from workspace display settings.

All button visibility preferences default to **ON** so existing behavior is unchanged after upgrading.

Turning off Free mode while Free Board is active returns the app to Solution mode before hiding the control. Turning off Notes immediately hides an open note. Hidden controls are neither rendered nor hit-testable. Hiding Lock or Flip preserves the current orientation and lock state; their direct actions are inert while hidden. Each workspace saves its own button visibility and board-size choices.

In STANDARD mode, the toolbar below the board fills its width. Its icon control uses a square touch target; visible text controls stretch evenly to consume the remaining space. Hiding any optional control removes that control's space and reallocates it among the visible text controls. The renderer also supports compact left and right alignment through the developer-facing `ToolbarAlignment` choice. Alignment is not a persisted preference or a Settings panel control. SMALL moves the board to the left and places the toolbar and narrower text navigation at the bottom of the space beside it. Available toolbar icons replace text labels; NOTE stays text. The SMALL toolbar uses compact left alignment. Controls wrap as needed to preserve minimum touch sizes, and analysis starts directly below the board.

## UI flow

A settings icon lives immediately to the left of the close button in the header. The `FILES` control, when present, sits to its left. Tapping it opens a modal panel using the same deterministic layout/hit-testing approach as the collection picker.

While the panel is open, ordinary board and toolbar actions are blocked. The panel matches the Files dialog size, with an X icon in the upper-right corner, four full-width ON/OFF rows plus the board-size row, Puzzles-only physical-seat and Chessnut-connection rows, and a full-width Close button anchored at the bottom. Both panel close controls dismiss Settings. The app's global Exit and manual Refresh controls remain available through their existing header behavior.

## Ownership

The layers keep the existing dependency direction:

- `chess-core` owns the versioned `Settings` value, settings actions, modal state, and the `SettingsChanged` effect.
- `chess-render` owns the header icon, modal geometry, rendering, and hit targets. It reads settings but never persists them.
- `kindle-platform` owns `SettingsStore` and filesystem paths.
- `app/kindle-chess` wires `SettingsChanged` to the store alongside progress effects.

This keeps preferences reusable by a future non-Kindle frontend.

## Persistence

The Kindle default is:

```text
/mnt/us/kindle-chess/state/settings.json
```

Override it with `KINDLE_CHESS_SETTINGS_FILE`. When `StoragePaths::new` is given a custom progress path, settings default to `settings.json` in the same directory.

The settings document is versioned independently from `progress.json`. Writes use the same atomic replace strategy as progress. If an existing settings file is malformed or has a future version, the app uses defaults in memory, reports a warning, and protects the source file from overwrite.

Version 1 currently serializes as:

```json
{
  "version": 1,
  "show_free_mode_button": true,
  "show_notes_button": true,
  "show_lock_button": true,
  "show_flip_button": true,
  "small_board": false,
  "physical_board_seat": "changing",
  "game_review": {
    "show_free_mode_button": true,
    "show_notes_button": true,
    "show_lock_button": true,
    "show_flip_button": true,
    "small_board": false
  }
}
```

Missing Lock/Flip fields default to ON. Files without `game_review` initialize both profiles from the flat values; subsequent edits affect only the active workspace.

New preferences should be added to this model with explicit defaults and host tests so older files remain predictable.

## Physical-board seating

Tap PHYSICAL BOARD to cycle WHITE SIDE → BLACK SIDE → CHANGING SIDE. Older
settings default to CHANGING SIDE, preserving their canonical wire layout. Values
are stored as the additive version-1 root field `physical_board_seat` with values
`white`, `black`, `changing`; invalid values retain malformed-file protection.

| Seat | White puzzle | Black puzzle |
| --- | --- | --- |
| WHITE SIDE | Original placement | Rotate 180° |
| BLACK SIDE | Rotate 180° | Original placement |
| CHANGING SIDE | Original placement | Original placement |

Changing side means you move to the solver's end (or turn the physical board).
For a fixed seat, mismatched solver colors rotate pieces on the physical squares
without changing their colors. The same self-inverse mapping converts sensor
reports back to canonical moves and orients automatic reply targets. It is fixed
from the puzzle's original solver color for the whole sent session; preview turn,
Kindle Flip/orientation lock and reply turn never change it.

Changing seating saves settings and pauses physical input without moving pieces,
changing grading/progress, or changing Kindle orientation. Tap Send to apply the
new seating and reacquire the physical baseline. Review's settings and controls
remain independent, and its dialog has no physical-seat row.
