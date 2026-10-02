# Kindle Chess Puzzles

A Kindle-first Rust/FBInk experiment for solving chess puzzles on a jailbroken **Kindle Scribe (1st generation)**.

This project starts from the behavior of [abavelski/eink-chess-app](https://github.com/abavelski/eink-chess-app), but it does **not** copy the Kobo/Cobalt runtime architecture. The goal is to keep chess, puzzle, application, layout, and rendering logic platform-neutral and put Kindle-specific framebuffer, input, lifecycle, build, and deployment code behind small adapters. A Kobo backend may be added later without changing the core model.

## First milestone: Kindle parity

The first usable Kindle version should match the currently implemented behavior of the Kobo app:

- 8x8 touch chessboard with the same FEN/UCI semantics;
- the same version-1 JSON puzzle collection format;
- exact stored-line solution checking, with automatic opponent replies;
- wrong-move rollback and clear monochrome result feedback;
- pawn promotion with Queen/Rook/Bishop/Knight choice;
- graded **Solution** mode and ungraded **Free Board** mode;
- reset, previous/next puzzle navigation, and manual board flip;
- automatic orientation toward the FEN side-to-move plus orientation lock;
- optional puzzle description reveal and difficulty display;
- multiple puzzle collections;
- durable active-collection/current-puzzle/solved progress, stored separately from puzzle files;
- Sashité Western SVG chess artwork, converted into deterministic e-ink-friendly assets.

This is still deliberately **not a chess engine**. Ordinary move legality, check/checkmate, castling rules, en-passant rules, and engine analysis are out of scope unless a later task explicitly adds them.

## Architecture direction

```text
shared Rust core
  board + FEN/UCI
  puzzle parser
  solution/application state
  progress
  layout + hit testing
  deterministic grayscale renderer
          |
          +-------------------+
          |                   |
          v                   v
Kindle adapter           future Kobo adapter
FBInk display            FBInk/other display
evdev touch              Kobo input
lifecycle/paths          lifecycle/paths
packaging/deploy         packaging/deploy
```

FBInk is the display boundary, not the application model. Device-specific code should translate platform events into shared `Action` values and present shared rendered frames/damage regions.

## Development style

This repository is **test driven**. Every implementation task starts with failing automated tests, then the smallest implementation to make them pass, then refactoring. Pure logic and rendering are tested on the host; FBInk/input/lifecycle boundaries get contract tests plus explicit physical-device checkpoints.

Start here:

- [AGENTS.md](AGENTS.md) — rules for coding agents;
- [Architecture](docs/ARCHITECTURE.md);
- [Kindle Scribe notes](docs/KINDLE_SCRIBE.md);
- [Puzzle format compatibility](docs/PUZZLE_FORMAT.md);
- [Testing strategy](docs/TESTING.md);
- [Implementation tasks](tasks/README.md).

## Current state

Tasks 00 and 01 are **Implemented**. The physical Scribe checkpoint passed on 2026-10-02, and the repository now has a Rust 1.85.1 workspace plus a platform-neutral `chess-core` covering board/FEN/UCI, promotions, version-1 puzzle collections, collection-name discovery, and progress serialization. Host formatting/clippy/tests and a minimal `armv7-unknown-linux-gnueabihf` cross-build are enforced in CI.

Task 02 is next: the pure puzzle-solving application state machine. The renderer, FBInk runtime, touch adapter, real device persistence, and production deployment toolchain are intentionally not implemented yet.

For the exact safe device-probe procedure, use [docs/device/ks1-barolo.md](docs/device/ks1-barolo.md). It covers the read-only environment report, minimal FBInk smoke test, finger/stylus event capture, host-side evdev decoding, recovery checks, and the verified Task 00 measurements.

The initial target is the Kindle Scribe first generation. Reuse for Kobo is an architectural constraint, not an active implementation target for the first milestone.
