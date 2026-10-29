# Architecture

## Goal

Build a Kindle Scribe puzzle app now without making the core a Kindle application.

The reusable unit is not "FBInk code." The reusable unit is the complete puzzle experience: board model, puzzle parsing, solving state, progress, layout, hit testing, and deterministic rendering. Kindle-specific code should be a thin edge that supplies input, storage paths, lifecycle, and an FBInk-backed display.

## Proposed workspace

```text
Cargo.toml
crates/
  chess-core/
    board.rs
    fen.rs
    uci.rs
    puzzle.rs
    progress.rs
    settings.rs
    app.rs
    view.rs
  chess-render/
    canvas.rs
    layout.rs
    hit_test.rs
    pieces.rs
    render.rs
    damage.rs
  fbink-sys/
    build.rs
    src/lib.rs
  kindle-platform/
    display.rs
    input.rs
    storage.rs
    lifecycle.rs
    device.rs
app/
  kindle-chess/
    main.rs
assets/
  sashite-western/
tests/
  fixtures/
scripts/
tasks/
docs/
```

Names may change during Task 01, but dependency direction must not.

## Dependency direction

```text
chess-core  <----- chess-render
    ^               ^
    |               |
    +-------- app ---+------ kindle-platform ---- fbink-sys
```

Rules:

- `chess-core` has no platform dependencies.
- `chess-render` may depend on core view types but not on Kindle/FBInk.
- `fbink-sys` contains raw FFI only.
- `kindle-platform` owns FBInk presentation, Linux input, paths, and device lifecycle.
- the binary owns the event loop and composes the layers.

A future Kobo binary should replace `kindle-platform`, not fork core/render behavior.

## Core model

### Board

Keep the semantics of the reference Kobo app:

- square indexes are stable and independent of orientation;
- FEN uses standard six-field notation;
- ordinary moves are physical-board moves, not legal-chess validation;
- selecting/deselecting is distinct from completing a move;
- promotion is staged before board mutation;
- stored opponent replies can move pieces directly by UCI coordinates.

Orientation is presentation state, never a change to square identity.

### Puzzle collection

Version-1 JSON is normalized into a runtime model containing at least:

- collection title;
- ordered puzzles;
- id;
- FEN;
- optional description;
- optional string/number difficulty;
- non-empty UCI solution line.

Keep the 256 KiB compatibility limit for version 1 initially even though Kindle storage does not impose the Cobalt store limit. Removing it later should be an explicit format-policy decision.

### App state

Use a single explicit state machine rather than spreading interaction state through UI code.

Suggested shape:

```text
AppState
  collections metadata
  active collection
  active puzzle
  board
  mode: Solution | FreeBoard
  solution cursor / feedback
  pending promotion
  flipped
  orientation_locked
  description_visible
  settings
  settings_open
  progress
  transient error/warning
```

Input is a logical `Action`, such as:

```text
TapSquare(square)
ChoosePromotion(kind)
CancelPromotion
PreviousPuzzle
NextPuzzle
Reset
Flip
ToggleMode
ToggleOrientationLock
ToggleDescription
OpenSettings
ToggleFreeModeSetting
ToggleNotesSetting
ToggleLockSetting
ToggleFlipSetting
ToggleBoardSizeSetting
CloseSettings
OpenCollectionPicker
SelectCollection(key)
CloseModal
Exit
```

`dispatch(Action)` should be deterministic and host-testable. It may return small effects for the outer runtime, such as "persist progress", "persist settings", or "exit", but it must not call FBInk or read `/dev/input`. Settings hold independent preferences for Puzzle and Game Review. The active workspace selects the profile used by controls and rendering; persistence always saves both profiles. Settings belong in core state, while the platform layer owns their filesystem persistence.

### View state

The renderer should consume a read-only view model derived from `AppState`, not poke mutable application internals. This makes snapshots straightforward and prevents device rendering code from changing behavior.

## Rendering

Render to a project-owned `Gray8` canvas.

Benefits:

- host snapshots use the same rendering code as the device;
- FBInk is only responsible for copying/presenting pixels and requesting refreshes;
- damage can be calculated in project coordinates;
- a future Kobo backend can reuse the frame;
- visual behavior does not depend on stock device fonts/widgets.

The renderer owns:

- board/frame geometry;
- coordinates;
- piece composition;
- toolbar/buttons;
- modal/picker layout;
- text placement;
- correct/wrong/complete marks;
- solved marker;
- descriptions;
- hit rectangles.

The renderer does **not** own interaction state.

Toolbar placement is derived in `chess-render::Layout` from display metrics, the core settings visibility flags, and a renderer alignment policy. The renderer uses those same rectangles for pixels and hit testing. Icon and text controls have different minimum widths. Left and right policies pack controls at their chosen edge; the current full-width policy stretches text controls evenly, spreads icon-only rows across their gaps, and keeps a single control at the left. No device coordinates or placement policy enter core state. The saved board-size preference selects the standard board or a board at 60% of its width and height (rounded down to whole squares). SMALL aligns the board at the left content edge and places toolbar and navigation controls at the bottom of the space to its right. Available toolbar icons replace their text labels; NOTE retains text. The SMALL toolbar uses compact left alignment and wraps into balanced rows when needed to preserve minimum touch sizes, while text navigation narrows to the sidebar width. The full-width text panel starts directly below the board and uses larger text. STANDARD geometry and presentation are unchanged. Dialog bounds retain their standard dimensions. A future user-selectable alignment would add a versioned core preference while leaving geometry in the renderer.

Renderer button definitions carry an optional icon, optional text, and an explicit
`ButtonType` (`Icon`, `Text`, or `IconAndText`). Toolbar layout and rendering use
the same definitions; display type controls sizing and visible content even when
both properties exist. Combined content places the icon left of the text; a
single remaining property is centered. FREE, LOCK, RESET and FLIP use vector icons in SMALL and retain their
text-only policy in STANDARD in both workspaces. Toolbar layout and drawing
share the same board-size presentation policy.

## Sashité pieces

Keep the original SVG files as source-of-truth assets and their license/provenance.

Do not parse SVG on the Kindle unless measurements later justify it. Prefer a reproducible host build step that turns the SVGs into grayscale/alpha assets at one or more canonical sizes. The renderer chooses the closest generated asset or uses deterministic scaling.

Generation must be reproducible and tested; generated files should never be hand-edited.

## Display backend

The Kindle display adapter owns:

- FBInk initialization/reinitialization;
- framebuffer/device state discovered from FBInk;
- copying a full frame or dirty rectangle to the display;
- selecting refresh settings;
- waiting for completion only when necessary;
- clean shutdown/restoration of state that this project changed.

Start with correctness-first full refreshes. Introduce partial refresh/damage policy only in its dedicated task.

FBInk supports Kindle builds and Kindle Scribe identification. Pin an exact known-good revision after device bring-up.

## Input backend

The Kindle input adapter owns:

- enumerating `/dev/input/event*`;
- identifying the touchscreen by capabilities/name, not by event number;
- decoding multi-touch events;
- normalizing raw ranges and rotation into display coordinates;
- collapsing finger interaction into tap/swipe intent;
- ignoring or separately classifying stylus events until explicitly supported.

The shared app should never see Linux input event codes.

Because the Scribe has no Kobo-style page-turn buttons, parity navigation maps to logical Previous/Next actions using on-screen controls first; swipe gestures may be added when the touch transform is verified.

## Storage

Core parsing plus progress/settings serialization takes bytes/strings, not paths.

The Kindle adapter chooses:

- puzzle directory;
- review-game directory;
- progress path;
- settings path;
- separate review-resume path;
- logs/debug path.

Keep paths configurable for tests.

Puzzle collections remain editable source data. The app never writes them for progress or preferences. Progress and settings are separate versioned documents; both use a safe temp-write + sync + rename pattern when supported by the selected filesystem, with malformed/future documents protected from overwrite and failures surfaced to the app.

## Lifecycle

Lifecycle is platform code.

Do not hardcode framework/service shutdown commands from memory. Device probing and dedicated lifecycle tasks must determine what is required to prevent the stock UI from repainting or competing for input.

Every transition away from the stock UI must have:

- finite waits;
- signal traps/cleanup;
- restoration of changed framebuffer state;
- a log;
- a documented manual recovery path.

## Portability contract

A later Kobo port is successful if it can reuse without modification:

- `chess-core`;
- `chess-render`;
- puzzle fixtures;
- progress serialization;
- snapshot tests;
- most app-state tests.

Only platform display/input/storage/lifecycle/build/deploy code should differ.

No Kobo backend is part of the first Kindle milestone.

### Workspace settings compatibility

`settings.json` remains version 1: existing flat `show_free_mode_button`,
`show_notes_button` and `small_board` fields now represent Puzzle preferences.
An additive `game_review` object stores the workspace preferences for review.
The additive `show_lock_button` and `show_flip_button` fields default to true
in both profiles; they control visibility without changing orientation/lock.
Loading an old file without that object copies its global values into both
profiles; subsequent edits affect only the active workspace. Migration stays
in memory until a settings edit is saved. Missing preference fields retain their
existing defaults; malformed/future files remain protected from overwrite.
Settings edits never persist puzzle progress or modify uploaded collections.

### Review resume

`review-resume.json` version 1 stores the last review collection filename, stable
game ID, authored UCI move path and selected FEN, orientation/lock and analysis
page/focus. Core captures/restores these values without touching puzzles; paths
and atomic writes stay in the platform adapter. Runtime saves on review actions,
retries failures (including exit), and restores the review session at startup
without switching away from Puzzles. Source updates with missing/ambiguous paths
or changed FEN fall back to the selected game's root; missing games use the first
available game. Malformed/future/unreadable documents are protected. Scratch
boards, piece selection, promotion dialogs and game pickers remain transient.
