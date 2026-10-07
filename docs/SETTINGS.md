# Settings architecture

Settings are application preferences, not puzzle data and not learning progress.

## Current settings

Version 1 starts deliberately small:

- **Free mode button** — show or hide the `FREE` toolbar control.
- **Notes button** — show or hide the `NOTE` toolbar control.

Both default to **ON** so existing behavior is unchanged after upgrading.

Turning off Free mode while Free Board is active returns the app to Solution mode before hiding the control. Turning off Notes immediately hides an open note. Disabled controls are neither rendered nor hit-testable.

The toolbar below the board currently fills its width. Its icon control uses a square touch target; visible text controls stretch evenly to consume the remaining space. Hiding FREE or NOTE removes that control's space and reallocates it among the visible text controls. The renderer also supports compact left and right alignment through the developer-facing `ToolbarAlignment` choice. Alignment is not a persisted preference or a Settings panel control.

## UI flow

A settings icon lives immediately to the left of the close button in the header. The `FILES` control, when present, sits to its left. Tapping it opens a modal panel using the same deterministic layout/hit-testing approach as the collection picker.

While the panel is open, ordinary board and toolbar actions are blocked. The panel matches the Files dialog size, with an X icon in the upper-right corner, two full-width ON/OFF rows, and a full-width Close button anchored at the bottom. Both panel close controls dismiss Settings. The app's global Exit and manual Refresh controls remain available through their existing header behavior.

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
  "show_notes_button": true
}
```

New preferences should be added to this model with explicit defaults and host tests so older files remain predictable.
