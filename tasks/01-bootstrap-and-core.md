# Task 01 — Bootstrap the Rust workspace and shared core

**Status:** Ready  
**Depends on:** Task 00 for confirmed cross-target facts; host work may begin earlier if it does not bake in assumptions.

## Outcome

Create the testable Rust workspace and port the platform-neutral compatibility core from the reference app.

No FBInk or Kindle input yet.

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

- all core tests pass on the host;
- `chess-core` contains no FBInk/Kindle/Linux-device dependency;
- `cargo fmt --check`, clippy, and workspace tests pass;
- reference v1 fixtures parse with the expected IDs/FEN/solutions;
- progress references IDs, not indexes;
- the confirmed Kindle target can compile a minimal Rust binary once cross-toolchain setup is available.

## Out of scope

- solution checking state machine;
- rendering;
- touch;
- persistence to real files;
- device deployment.

## Suggested commits

`test: specify reference puzzle core behavior`

`core: add platform-neutral board and puzzle model`
