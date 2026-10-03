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
- [FBInk integration](docs/FBINK.md);
- [Puzzle format compatibility](docs/PUZZLE_FORMAT.md);
- [Testing strategy](docs/TESTING.md);
- [Implementation tasks](tasks/README.md).

## Current state

Tasks 00-06 are **Implemented**. M1 is complete and the first live Scribe loop is validated: the repository has platform-neutral `chess-core`, deterministic `chess-render`, a pinned FBInk boundary, capability-based finger input discovery, measured Scribe touch normalization, and a bundled-puzzle event loop. HUMAN CHECKPOINT B passed on 2026-10-02 after correcting 32-bit sysfs capability-mask parsing and ensuring the cross-build targets the Scribe's glibc 2.35 baseline.

The physical Task 04 checkpoint verified board geometry, corner/center mapping, every visible finger control, repeated-move alignment, stylus filtering, and normal recovery after termination. It also documented stock-UI input/repaint contention and temporary refresh traces; those remain later lifecycle/refresh work rather than Task 04 fixes.

Task 05 is **Implemented**, with HUMAN CHECKPOINT C passed on the physical Scribe on 2026-10-02. The live binary now bundles one six-puzzle parity collection covering one-move and three-ply solving, white/black/manual/automatic promotion, Free Board, navigation, orientation lock/flip, descriptions, feedback, difficulty, and solved display. Previous/Next are visibly disabled at collection ends, and host integration tests drive complete action→state→render flows through deterministic Scribe snapshots.

Task 06 is **Implemented**, with **HUMAN CHECKPOINT D** passed on the physical Scribe on 2026-10-02. Collection switching, remembered puzzle IDs and solved markers after restart, computer-copy discovery, invalid-file preservation, and normal recovery were verified. M2 is complete. The Kindle loop now discovers multiple version-1 collections, provides a paged FILES picker, preserves invalid source files, and stores app-owned progress separately with protected corrupt/future records and atomic dirty-retry writes. Defaults are `/mnt/us/kindle-chess/puzzles` for collections and `/mnt/us/kindle-chess/state/progress.json` for progress, with environment overrides for tests and alternate deployments.

Task 07 is **Implemented with the natural suspend/resume checkpoint deferred by the user** on 2026-10-03. Host-tested pixel damage, batched rectangle presentation, provisional refresh policy, timings, and a supervised overlay launcher are implemented. Exclusive finger input and native exit repaint passed promotion and crash-recovery probes. The user confirmed that only the sleeping test remains; resume reliability is still unverified. See [e-ink/lifecycle notes](docs/EINK_LIFECYCLE.md).

For the exact safe device-probe procedure, use [docs/device/ks1-barolo.md](docs/device/ks1-barolo.md). It covers the read-only environment report, minimal FBInk smoke test, finger/stylus event capture, host-side evdev decoding, recovery checks, and the verified Task 00 measurements.

The initial target is the Kindle Scribe first generation. Reuse for Kobo is an architectural constraint, not an active implementation target for the first milestone.

## Build and install

Task 08 provides a pinned release build, inspected stage, scp deployment and
Scriptlet/KPM artifacts. See [build/deploy commands](docs/BUILD_DEPLOY.md).
Clean-checkout builds and Scribe deployment, KPM installation/update, and
uninstall/reinstall preserve puzzle/progress data. Checkpoint F still needs
library launch/exit and post-reboot launch observations. Task 07's natural
suspend/resume test is deferred by the user.
