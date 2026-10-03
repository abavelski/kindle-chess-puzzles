# Testing strategy

The app should be mostly testable without an e-reader.

## Principle

Use the physical Scribe to validate hardware facts, not to discover ordinary application bugs.

The test stack has four layers:

1. pure core unit/state-machine tests;
2. deterministic renderer/layout/hit-test tests;
3. platform contract/cross-build tests;
4. explicit physical-device checkpoints.

## 1. Core tests

These are the primary TDD surface.

Test:

- FEN parsing/validation;
- square/index/UCI conversion;
- board selection/deselection/move/promotion staging;
- puzzle version-1 parsing and validation;
- progress serialization/version handling;
- collection filtering/order;
- exact solution state transitions;
- wrong-move rollback;
- automatic opponent replies;
- completion;
- Reset;
- Free Board transitions;
- orientation rules;
- description visibility;
- collection switch restoration;
- solved-state behavior.

Prefer table-driven tests for parser cases and explicit state assertions for interaction flows.

Port known fixtures from the reference Kobo repository so compatibility is executable rather than documented only.

## 2. Renderer tests

The shared renderer produces a deterministic Gray8 frame.

Maintain snapshot/golden tests for at least:

- normal white-to-move board;
- black-oriented board;
- selected square;
- correct intermediate state;
- wrong result;
- complete result;
- solved marker;
- Free Board selected state;
- description visible/hidden;
- orientation lock selected state;
- promotion chooser;
- collection picker;
- file/progress warning;
- optional difficulty string and number.

Also test layout invariants numerically:

- board is square;
- all 64 hit rectangles map to the correct logical square in both orientations;
- controls have minimum physical touch size based on display DPI;
- no control overlaps board/modal bounds;
- text regions stay inside the viewport.

Snapshots should be reviewed, not blindly regenerated.

## 3. Damage tests

Once partial refresh is introduced, test damage independently of FBInk.

Examples:

- selection changes one square/highlight region;
- piece move damages origin and destination plus feedback if it changes;
- result icon damages its overlay region;
- puzzle navigation damages board/header/description as required;
- modal open/close damages its complete bounds;
- full redraw remains available.

The damage calculator should conservatively overdraw rather than miss changed pixels.

## 4. Platform contract tests

Keep FBInk FFI thin.

Host tests can cover:

- wrapper configuration mapping;
- frame stride/rectangle validation;
- errors before unsafe calls;
- input event decoding using recorded event fixtures;
- raw touch-range normalization;
- rotation transforms;
- storage path selection;
- safe progress write behavior in temporary directories.

The CI should also cross-compile the Kindle binary once the toolchain is established. A cross-build passing is not a device test, and it is not sufficient evidence of runtime libc compatibility: Task 04 caught a GLIBC_2.38 artifact on a glibc 2.35 Scribe despite a green build. Keep the deployment baseline explicit and verify produced artifacts on-device at the relevant human checkpoint.

## Reference compatibility fixtures

Copy or recreate small fixtures from `abavelski/eink-chess-app`:

- one-move puzzle;
- alternating three-ply puzzle;
- white queen promotion;
- white underpromotion;
- black promotion;
- automatic opponent promotion;
- two valid collections;
- malformed collection;
- version-1 progress with solved IDs.

For the same fixture and logical actions, core state should match the documented reference behavior.
Task 09 records those expectations in `tests/fixtures/parity-vectors.json` and executes
them through `AppState`, including start FEN/side, solver plies, automatic replies,
final board placement, completion, and solved state.

Do not make tests depend on Cobalt.

## Commands

Once the workspace exists, the default quality gate is:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Add a repository script such as `scripts/check.sh` when Task 01 creates the workspace.

After cross compilation exists, include the documented Kindle build command in `scripts/check.sh` or CI.

## Physical-device test record

Hardware tasks must record:

- device/firmware identifier;
- build/commit tested;
- commands/package used to launch;
- pass/fail per acceptance item;
- unexpected artifacts (ghosting, coordinate offset, stale pixels, stock UI repaint);
- recovery steps if anything fails.

Keep the first device record at `docs/device/ks1-barolo.md` and append/update verified facts rather than relying on chat history.

## Human checkpoints

A task with a **HUMAN CHECKPOINT** is not complete until the listed device checks pass.

Agents may prepare binaries/scripts and automated tests, but must not mark the task Implemented from host tests alone.

## Test-driven task template

For each task:

1. write the acceptance test names first;
2. make them fail for the missing behavior;
3. implement the smallest behavior;
4. make focused tests green;
5. run all workspace tests;
6. run render snapshots if UI changed;
7. cross-build if platform code changed;
8. perform the human checkpoint if required;
9. update task status and verified docs.
