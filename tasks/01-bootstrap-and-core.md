# Task 01 — Bootstrap the Rust workspace and shared core

**Status:** Implemented  
**Depends on:** Task 00 for confirmed cross-target facts; host work may begin earlier if it does not bake in assumptions.

## Outcome

Create the testable Rust workspace and port the platform-neutral compatibility core from the reference app.

No FBInk or Kindle input yet.

## Implemented

Task 01 created a Rust 1.85.1 workspace with:

- `chess-core` for all platform-neutral compatibility behavior;
- placeholder `chess-render` and `kindle-platform` crates that establish dependency direction without implementing later tasks;
- a minimal `kindle-chess` binary used only as a cross-compilation smoke test;
- exact pinned `serde` / `serde_json` dependency versions and a committed lockfile;
- `scripts/check.sh` for formatting, clippy, Rust tests, and the existing Task 00 tooling checks;
- `scripts/check-kindle.sh` for the confirmed `armv7-unknown-linux-gnueabihf` target;
- GitHub Actions host and ARMv7 hard-float cross-build jobs;
- reference-compatible puzzle, promotion, collection metadata, and progress fixtures;
- `docs/PROVENANCE.md`.

The shared core now implements board/FEN/UCI behavior, staged and atomic promotion, direct stored moves, version-1 puzzle parsing, collection filename discovery rules, and version-1 progress state.

No solution state machine, rendering, touch handling, real filesystem persistence, FBInk integration, or device deployment was added.

## Validation

GitHub Actions run 3 on the completion commit chain verified:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`, including 17 Task 01 compatibility tests;
- the existing six Task 00 evdev decoder tests and probe-script syntax checks;
- a minimal `kindle-chess` cross-build for `armv7-unknown-linux-gnueabihf` using an ARMv7 hard-float Linux GCC linker.

The cross-build is a target/compile smoke test. It does **not** claim that the generic CI sysroot is the final Kindle deployment sysroot, nor that this binary has been executed on-device. Those device/runtime concerns remain for later platform/build tasks.

## Work

Create the workspace described in `docs/ARCHITECTURE.md`, initially including at least:

- `chess-core`;
- `chess-render` placeholder crate if useful for dependency setup;
- Kindle binary/platform placeholders that compile without hardware behavior.

Add:

- pinned Rust toolchain;
- `rustfmt`/clippy policy;
- CI or repository check script;
- test fixtures;
- license/provenance notes for copied/adapted source/assets.

Port/reimplement in `chess-core`:

- Color/Piece/PieceKind;
- Board with square index `0 = a8`, `63 = h1`;
- FEN parsing/validation;
- square <-> algebraic conversion;
- UCI parsing/formatting;
- selection/deselection/completed-move result;
- direct move for stored replies;
- staged promotion and atomic promotion;
- version-1 puzzle collection parser;
- optional title/description/difficulty;
- collection filename filtering/sorting;
- progress v1 serialization/model.

Do not port Cobalt app/UI code.

## Red tests first

Port or recreate tests covering:

- initial board and FEN positions;
- invalid FEN fields/ranks/pieces;
- move result versus selection result;
- moving onto occupied square;
- white/black promotion staging;
- q/r/b/n promotion;
- direct stored reply;
- square/UCI round trips;
- valid and invalid UCI;
- valid v1 collection;
- duplicate/blank IDs;
- unsupported version;
- >256 KiB rejection;
- difficulty string/number/null;
- collection title trimming;
- filename filtering/order;
- progress round trip, deduplicated solved IDs, future-version rejection.

Use fixtures equivalent to the Kobo examples/promotion examples.

## Compatibility rule

The parser behavior described in `docs/PUZZLE_FORMAT.md` is the source of truth.

Keep the 256 KiB v1 limit initially even though Kindle does not require it.

## Acceptance criteria

- [x] all core tests pass on the host;
- [x] `chess-core` contains no FBInk/Kindle/Linux-device dependency;
- [x] `cargo fmt --check`, clippy, and workspace tests pass;
- [x] reference v1 fixtures parse with the expected IDs/FEN/solutions;
- [x] progress references IDs, not indexes;
- [x] the confirmed Kindle target compiles a minimal Rust binary in CI with an ARMv7 hard-float cross linker.

## Out of scope

- solution checking state machine;
- rendering;
- touch;
- persistence to real files;
- device deployment.

## Suggested commits

`test: specify reference puzzle core behavior`

`core: add platform-neutral board and puzzle model`
