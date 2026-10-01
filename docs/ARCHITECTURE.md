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
OpenCollectionPicker
SelectCollection(key)
CloseModal
Exit
```

`dispatch(Action)` should be deterministic and host-testable. It may return small effects for the outer runtime, such as "persist progress" or "exit", but it must not call FBInk or read `/dev/input`.

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

Core parsing/progress serialization takes bytes/strings, not paths.

The Kindle adapter chooses:

- puzzle directory;
- progress path;
- logs/debug path.

Keep paths configurable for tests.

Puzzle collections remain editable source data. The app never writes them for progress. Progress writes should use a safe temp-write + sync + rename pattern when supported by the selected filesystem, with failure surfaced to the app.

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
