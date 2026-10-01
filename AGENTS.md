# AGENTS.md

This repository is intended to be implemented largely by coding agents. Follow this file as the repository-level operating contract.

## Read before changing code

Read, in order:

1. `README.md`
2. `docs/ARCHITECTURE.md`
3. `docs/TESTING.md`
4. `docs/KINDLE_SCRIBE.md`
5. `docs/PUZZLE_FORMAT.md`
6. the single task file you are implementing under `tasks/`

The task file is the scope boundary. Do not opportunistically implement later tasks.

## Development rule: red -> green -> refactor

Every behavioral change starts with a failing automated test.

1. Add the smallest test that demonstrates the missing behavior.
2. Run it and confirm it fails for the expected reason.
3. Implement the smallest production change that makes it pass.
4. Run the focused tests.
5. Run the full relevant suite.
6. Refactor only while tests remain green.

If a behavior can be tested on the host, it must not rely only on a physical-device test.

## Architectural constraints

Keep these layers separate:

- **core**: board state, FEN/UCI helpers, puzzle parsing, solution state machine, progress, app actions/view state;
- **render**: deterministic layout, hit testing, grayscale frame generation, damage calculation, generated piece assets;
- **Kindle platform**: FBInk FFI, framebuffer presentation, evdev/input discovery, device paths, lifecycle, packaging/deploy;
- **binary**: wires the platform adapter to shared app state.

Core and renderer code must not import Kindle, FBInk, KPM, shell, framebuffer, or `/dev/input` concepts.

Do not put device coordinates directly into app state. Layout converts display metrics to rectangles; input converts device events to logical coordinates; hit testing converts coordinates to shared `Action` values.

Do not make FBInk the renderer. Render into a project-owned grayscale frame first, then use FBInk to present the whole frame or dirty rectangles.

## Product compatibility

The first milestone is parity with the currently implemented Kobo app, not with every old roadmap idea.

Required parity includes:

- puzzle file version 1 and its current validation semantics;
- exact stored UCI solution matching with automatic opponent replies;
- promotion choice;
- Solution and Free Board modes;
- reset, previous/next, flip;
- automatic side-to-move orientation and orientation lock;
- description visibility toggle and automatic reveal on solve;
- optional difficulty in the header;
- multiple collections;
- persistent active collection, per-collection current puzzle, and solved IDs;
- monochrome correct/wrong/complete feedback;
- Sashité Western pieces.

The old Kobo tasks for **Unsolved only** navigation and **version-2 branching/rich solutions** are not parity requirements. Do not implement them unless a new task explicitly adds them.

This app is not a chess engine. Do not add general move legality, check/checkmate evaluation, engine analysis, PGN, castling validation, or en-passant behavior unless explicitly tasked.

## Puzzle and progress compatibility

Treat `docs/PUZZLE_FORMAT.md` as a compatibility contract.

- Do not silently change version-1 JSON semantics.
- Keep uploaded puzzle collections immutable.
- Store learning progress separately.
- Durable progress references puzzle IDs, never array indexes.
- Preserve malformed/future progress instead of overwriting it automatically.
- Keep file/path selection outside core so Kindle and a future Kobo backend can choose different storage locations.

## Rendering rules

E-ink is monochrome-first.

- State must be understandable without color.
- Prefer stable, high-contrast geometry.
- Avoid animation as a correctness requirement.
- Keep piece rendering deterministic.
- Keep source SVG artwork; generate target raster assets reproducibly.
- Never update the whole screen just because it is easier once damage tracking exists.
- Never optimize waveform/refresh behavior before correctness is established on the device.

Snapshot tests should exercise all visible states introduced by a task.

## Kindle hardware rules

Do not guess destructive lifecycle commands.

Before changing the stock Kindle UI, framebuffer mode, rotation, power state, or services:

- verify the exact behavior on the target Scribe;
- document the command/state transition;
- add cleanup/trap behavior;
- preserve a manual recovery route.

The first device tasks use direct shell/scriptlet launch. KPM packaging comes later.

Do not assume an input event node number. Discover input devices by capabilities/name.

Do not assume framebuffer resolution, stride, rotation, or touch range. Probe them.

## Unsafe/FFI rules

Keep `unsafe` code inside the smallest possible FBInk/sys boundary.

- Pin the FBInk revision once a known-good Scribe build is established.
- Never follow floating `master` in reproducible builds.
- Wrap raw pointers/file descriptors in safe Rust types.
- Convert FBInk errors into typed Rust errors with context.
- Add compile/contract tests around the wrapper where host execution is impossible.

## Tests and quality gates

Before completing a task, run what exists at that point, normally:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Once the Kindle cross-build exists, also run its build/check command.

For rendering changes, update/add snapshot tests intentionally. Do not overwrite snapshots just to make CI green; inspect the difference first.

For device-boundary tasks, complete the physical checklist in the task file and record findings in the requested device document.

## Task discipline

Implement one numbered task at a time.

At task start:

- read its dependencies and acceptance criteria;
- identify the tests that should fail first;
- avoid unrelated cleanup.

At task completion:

- all automated acceptance tests pass;
- required device checkpoint is recorded;
- docs reflect any verified hardware fact that changed an assumption;
- change the task status from `Ready` to `Implemented`;
- keep later tasks untouched unless a verified finding invalidates them.

If a task uncovers a hardware fact that requires replanning, update the plan before coding around it.

## Commits

Prefer small commits that describe behavior, for example:

- `test: specify solution state transitions`
- `core: add exact puzzle solution state machine`
- `render: add deterministic board snapshots`
- `kindle: present grayscale frames through FBInk`

Do not mix broad formatting churn with behavioral changes.

## Definition of done

A feature is done only when:

- behavior is specified by tests where host-testable;
- tests are green;
- platform-neutral boundaries remain intact;
- e-ink states are visually testable without color;
- required physical-device checks pass;
- no uploaded puzzle data is modified as a side effect;
- documentation matches the implementation.
