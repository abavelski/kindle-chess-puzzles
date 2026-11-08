# Portability and app reuse

## Why this document exists

The current Kindle Chess Puzzles codebase is a good base for further work, but the next two directions add a second axis of variation:

1. run the puzzle app on other e-readers, starting with Kobo Libra H2O;
2. build other chess applications, starting with an annotated grandmaster-game reader.

The goal is to support both without copying the application or platform code. This document records the main refactoring priorities before either expansion starts.

## Current strengths

Several parts of the repository are already well suited for reuse:

- board, FEN and UCI logic are pure Rust;
- rendering is deterministic and produces a project-owned Gray8 frame;
- layout is based on display metrics instead of one fixed screen size;
- rich analysis already models SAN, UCI, comments, variations, NAGs, move references and precomputed positions;
- host tests and snapshots cover most behavior;
- unsafe code is isolated around FBInk;
- build and device validation are reproducible and documented.

This means the next step should be a refactor, not a rewrite.

## Main issue: two independent dimensions are still mixed

The future product matrix is:

| | Kindle | Kobo |
| --- | --- | --- |
| Puzzle trainer | Kindle puzzles | Kobo puzzles |
| Annotated game reader | Kindle games | Kobo games |

For this to remain maintainable, product-specific code and device-specific code must vary independently.

The current repository has the right high-level intent, but some implementation boundaries are still too broad.

### Product logic is mixed into shared core and rendering

`chess-core` contains both generic chess primitives and puzzle-product state. Board/FEN/UCI are generic; puzzle parsing, solved progress, grading, Free Board, collection navigation and puzzle UI state are not.

Likewise, `chess-render` contains useful generic rendering primitives, but its layout and hit targets map directly to puzzle `Action` values. A game-reader application should be able to reuse the board, text, pagination and analysis rendering without inheriting puzzle controls.

The rich `AnalysisTree` is also almost reusable as-is, but its parser currently enforces puzzle-specific rules such as the main line matching the legacy puzzle `solution`. Generic tree validation and puzzle compatibility validation should be separate steps.

### Platform code still knows too much about chess and the puzzle app

`kindle-platform` currently contains logic that depends on chess `Board`, puzzle collections, puzzle progress and puzzle layout. Examples include board-aware refresh planning and puzzle-aware storage.

A hardware/platform layer should know about:

- display;
- input;
- device lifecycle;
- paths;
- filesystem operations;
- packaging/runtime integration.

It should not know what a puzzle, solution, solved ID, chess board or analysis tree is.

### FBInk is currently Kindle-specific

The `fbink-sys` build and C bridge currently select and validate the Kindle target. This is correct for the shipped application, but it prevents the supposedly shared FBInk boundary from being reused directly by Kobo.

Before adding `kobo-platform`, the FBInk build should be parameterized by platform rather than duplicated.

## Recommended dependency shape

Do not create many tiny crates just for architectural purity. A small number of clear boundaries is enough.

A practical target is:

```text
chess-model
  board, FEN, UCI

chess-analysis
  analysis tree, comments, SAN/UCI display data,
  NAGs, move references, precomputed positions

eink-ui
  Gray8 canvas, geometry, fonts, board component,
  text flow, pagination, generic damage

puzzle-app
  puzzle format, puzzle state/actions, progress,
  puzzle layout/rendering/hit testing

game-reader-app
  game/book format, game state/actions,
  game layout/rendering/hit testing

platform-common
  generic filesystem and platform event abstractions

fbink-sys
  FBInk FFI parameterized by target platform

kindle-platform
  Kindle display/input/lifecycle/paths

kobo-platform
  Kobo display/input/buttons/lifecycle/paths
```

Application binaries then become thin compositions, for example:

```text
kindle-puzzles = puzzle-app + eink-ui + kindle-platform
kobo-puzzles   = puzzle-app + eink-ui + kobo-platform
kindle-games   = game-reader-app + eink-ui + kindle-platform
kobo-games     = game-reader-app + eink-ui + kobo-platform
```

The important rules are:

- platform crates must not depend on puzzle or game-reader crates;
- platform crates must not know about `Board`, `Puzzle` or `AnalysisTree`;
- generic UI must not depend on puzzle actions;
- puzzle and game applications must not know about Kindle, Kobo or FBInk.

## Kobo support

Do not start the Kobo port by copying `kindle-platform`.

First make the platform boundary generic, then add Kobo-specific code only where the hardware and lifecycle actually differ.

The existing `DisplayMetrics { width, height, dpi }` approach should be kept. Add host layout/snapshot coverage for both the Scribe and Libra H2O profiles. Do not hardcode the Libra framebuffer orientation from product specifications: probe the real device for framebuffer geometry, rotation, touch ranges and page-button events and record those measurements under `docs/device/`.

Physical page buttons should become logical platform events such as:

```text
Tap(x, y)
PageForward
PageBackward
Sleep
Wake
ExitRequested
```

The active application decides what those events mean.

Refresh policy should also remain platform-specific. The app/renderer may describe dirty and clean regions, but the platform decides how those map to FBInk refresh modes.

## Annotated grandmaster-game reader

The rich-analysis work already provides most of the difficult runtime model for a game reader:

- SAN and UCI;
- comments;
- variations;
- NAGs;
- clickable move references;
- precomputed FEN positions;
- paginated e-ink text.

Keep PGN conversion offline. The device runtime should consume deterministic prepared data rather than parse PGN or run a chess engine.

A game record can wrap the reusable analysis tree with metadata such as players, event, date, result and optional opening information.

The game-reader state should remain independent from puzzle state. It should not inherit puzzle grading, solved IDs, wrong-move rollback, Free Board or puzzle navigation merely because the UI reuses the same board and analysis components.

## Refactoring order

Keep every step behavior-preserving where possible.

1. Update architecture and agent guidance so current boundaries are explicit.
2. Add Scribe and Libra host layout/device profiles and snapshots.
3. Extract generic board/FEN/UCI into a clearly reusable chess-model layer.
4. Separate generic analysis-tree validation from puzzle compatibility validation.
5. Split reusable e-ink/board/text components from puzzle-screen rendering and hit testing.
6. Move puzzle-aware storage and board-aware refresh logic out of `kindle-platform`.
7. Parameterize the FBInk boundary for Kindle and Kobo.
8. Introduce common platform events and device-profile concepts.
9. Bring up and document the physical Kobo backend.
10. Build the Kobo puzzle binary using the existing puzzle application.
11. Define the annotated-game data contract and implement the game-reader application.
12. Compose Kindle and Kobo game-reader binaries from the same app code.

A successful architecture should make steps 10 and 12 mostly composition and device validation rather than large feature implementations.

## Agent and token-efficiency guidance

The repository is already designed for agent-driven work, but the documentation can be made cheaper to consume.

Root `AGENTS.md` should contain only stable repository-wide rules. Completed phase-specific branch policies and historical implementation detail should move to historical documents rather than being mandatory reading.

Area-specific rules should live close to the code, for example:

```text
crates/chess-model/AGENTS.md
crates/puzzle-app/AGENTS.md
crates/eink-ui/AGENTS.md
crates/kindle-platform/AGENTS.md
crates/kobo-platform/AGENTS.md
```

Each implementation task should explicitly state:

- files/docs to read;
- files likely to change;
- files that must not change;
- expected API or behavior;
- the first failing test;
- focused validation command;
- final validation command;
- whether physical hardware is required;
- suggested model capability.

Prefer small tasks with narrow acceptance criteria over large architectural prompts. Cheap models are well suited to mechanical moves, dependency cleanup, focused tests and small adapters once the architecture is frozen.

Also keep a full `scripts/check.sh` gate, but add focused checks for model, analysis, UI, Kindle and Kobo work so an agent does not need to interpret unrelated failures during every small task.

## Definition of success

The refactor is successful when:

- the existing Kindle puzzle behavior remains unchanged;
- the same puzzle application runs on Kindle and Kobo without forking product logic;
- the annotated-game reader reuses chess, analysis and UI components without inheriting puzzle state;
- adding a new app normally does not require modifying platform crates;
- adding a new device normally does not require modifying product crates;
- both supported device profiles are covered by deterministic host tests;
- hardware-specific facts are isolated in measured device documentation;
- most future tasks can be executed from a small, explicit context set.
