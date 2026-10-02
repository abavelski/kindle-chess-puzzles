# Task 02 — Build the pure application state machine

**Status:** Implemented  
**Depends on:** Task 01

## Outcome

Make puzzle solving behavior deterministic and fully testable without rendering or a device.

The result should be an `AppState` + logical `Action` model that represents the current board, puzzle attempt, modes, orientation, feedback, transient UI state, and progress mutations.

No FBInk, evdev, file paths, or shell commands belong here.

## Implemented

Task 02 adds a pure application state machine to `chess-core`:

- `AppState` owns the active collection, current puzzle, physical board, solution cursor/feedback, mode, orientation/lock, description visibility, pending promotion, progress, and transient message state;
- logical `Action` values cover board taps, promotion choice/cancel, puzzle navigation, reset/flip/mode/orientation/description controls, collection activation, and transient messages;
- `Effect::ProgressChanged` is the only external effect, so storage remains an outer-runtime concern;
- solver moves are compared through the existing UCI parser, wrong moves restore an exact pre-move board snapshot, and one stored opponent reply is auto-applied directly without chess-legality checks;
- promotions are staged without mutating the board, support q/r/b/n choices for both colors, and use the same atomic board/UCI helpers as stored replies;
- Free Board is ungraded, Solution -> Free preserves the visible position, Free -> Solution resets the attempt, and puzzle/reset transitions preserve the reference-mode rules;
- automatic side-to-move orientation, orientation lock/manual flip behavior, description reveal/toggle behavior, remembered puzzle IDs, and solved-once progress semantics match the reference app.

The implementation stays platform-neutral: no renderer, FBInk, evdev, filesystem, lifecycle, or Kindle dependency was added.

## Validation

GitHub Actions run 6 on the implementation commit chain verified:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`, including 12 Task 02 state-machine tests plus the existing 17 core compatibility tests;
- the existing Task 00 Python/tooling checks through `scripts/check.sh`;
- the minimal `kindle-chess` ARMv7 hard-float cross-build smoke test.

## State to model

At minimum:

- active collection key/metadata supplied by the outer runtime;
- active puzzle index/id;
- board;
- mode: `Solution | FreeBoard`;
- next solution ply;
- feedback: none/correct/wrong/complete;
- pending promotion;
- flipped;
- orientation lock;
- description visibility;
- modal state where useful;
- in-memory progress;
- transient warning/error text.

## Logical actions

Support pure actions such as:

- tap square;
- choose/cancel promotion;
- previous/next puzzle;
- reset;
- flip;
- toggle mode;
- toggle orientation lock;
- toggle description;
- complete collection selection supplied by runtime.

The exact enum may evolve, but platform input must not leak into it.

## Red tests first: solution checking

Specify and implement:

- selecting/deselecting does not advance a solution;
- correct one-move puzzle completes;
- wrong move restores the exact board and leaves solution cursor unchanged;
- three-ply line accepts solver move, auto-applies opponent reply, then accepts final solver move;
- automatic reply is coordinate-based and does not ask whether it is chess-legal;
- automatic opponent promotion applies its suffix directly;
- completion ignores further board taps in Solution mode until reset/navigation/mode transition;
- Reset restores original FEN and fresh attempt;
- changing puzzle clears transient feedback/description/promotion.

## Red tests first: promotion

Specify state transitions before UI exists:

- moving a white/black pawn to last rank enters pending-promotion state without mutating the board;
- cancel restores/keeps original board and attempt;
- q/r/b/n choice forms the correct UCI;
- wrong underpromotion is rejected and board restored;
- correct underpromotion can complete a puzzle;
- Free Board promotion never changes solved state.

## Red tests first: modes/orientation/description

Specify:

- Free Board accepts arbitrary physical moves;
- Free Board does not advance solution state or solved state;
- switching Solution -> Free keeps current visible board;
- switching Free -> Solution resets FEN/attempt but keeps orientation;
- Reset in Free Board keeps Free Board mode;
- puzzle navigation keeps current mode;
- selecting a new puzzle auto-orients to side-to-move when orientation lock is off;
- lock preserves current orientation across navigation/collection activation;
- Flip works while locked;
- unlocking does not jump the current board, but later navigation resumes auto-orientation;
- completing a puzzle reveals description;
- manual description toggle works before solving and in Free Board;
- revealing a description does not mark solved;
- reset/navigation hides description;
- solving marks the current puzzle solved exactly once.

## Effects

If dispatch needs external work, return explicit effects rather than performing it.

Examples:

- `ProgressChanged`;
- `RequestFullRedraw` is **not** a core effect; rendering/damage derives from before/after view state;
- collection bytes/path reads stay outside core;
- exit/lifecycle stays outside core.

Keep progress mutation deterministic so the runtime can persist after a state change.

## Acceptance criteria

- [x] all behaviors above are host unit/state-machine tests;
- [x] no rendering/platform dependency in `chess-core`;
- [x] no mutable global state;
- [x] wrong moves cannot partially mutate the board;
- [x] auto replies and promotions use the same tested UCI helpers as the parser;
- [x] reference Kobo solution fixtures produce the same logical outcomes;
- [x] all workspace checks pass.

## Suggested commits

`test: specify puzzle application state transitions`

`core: add puzzle solving application state machine`
