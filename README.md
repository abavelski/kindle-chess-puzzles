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
- [Phase 2: rich solution browsing](docs/PHASE_2.md);
- [Puzzle format compatibility](docs/PUZZLE_FORMAT.md);
- [Safe collection updates](docs/COLLECTION_UPDATES.md);
- [Testing strategy](docs/TESTING.md);
- [Implementation tasks](tasks/README.md).

## Current state

**Phase one is closed.** Tasks 00-09 establish the first Kindle Scribe parity release:
platform-neutral puzzle/application/rendering logic, pinned FBInk presentation, measured
finger input, multiple collections and durable progress, damage-aware e-ink updates,
supervised display/input ownership, reproducible ARMv7/glibc-2.35 builds, Scriptlet/KPM
packaging, and reference-parity validation.

The validated hardware target is the **first-generation Kindle Scribe (Barolo), firmware
5.19.6**. Physical checkpoints covered puzzle solving, promotions, Free Board, navigation,
orientation, descriptions, collection switching, restart persistence, repeated touch use,
normal/crash exit recovery, and library launch/exit. See
[the parity matrix](docs/PARITY.md), [measured results](docs/RESULTS.md), and
[the device record](docs/device/ks1-barolo.md).

The user confirmed natural idle and power-button sleep/wake with the app open on
2026-10-04. Library launch after a full device reboot remains unverified. **Phase two is closed**, with rich book
solutions, main lines, side lines, alternative lines, comments, explicit move references,
and finger/stylus tap-to-preview positions. The results are in
[docs/PHASE_2.md](docs/PHASE_2.md) and the completed tasks are archived under
[tasks/implemented/phase-2/](tasks/implemented/phase-2/). Residual Scribe panel ghosting is no
longer an active investigation: the existing top-left **Refresh** button is the accepted
manual workaround when a ghost becomes visible.

## Build, install, and use

The tested workflow is documented in [BUILD_DEPLOY.md](docs/BUILD_DEPLOY.md). From a fresh
checkout:

```sh
git submodule update --init --recursive
scripts/check.sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
```

Deploy over the previously configured SSH transport with the device address supplied at
runtime:

```sh
scripts/deploy-kindle.sh --host root@DEVICE_IP --port 2222
```

The installed layout keeps runtime and user data separate:

- runtime: `/mnt/us/kindle-chess/runtime/`;
- puzzle collections: `/mnt/us/kindle-chess/puzzles/`;
- progress: `/mnt/us/kindle-chess/state/progress.json`;
- logs: `/mnt/us/kindle-chess/logs/`;
- library Scriptlet: `/mnt/us/documents/kindle-chess.sh`.

With Scriptlets/SH_Integration installed, launch **Kindle Chess Puzzles** from the Kindle
library. A verified direct-shell fallback is
`sh /mnt/us/kindle-chess/runtime/launch.sh`. Use the app's top-right **X** to exit;
it flushes pending progress and lets the supervisor return display/input ownership to the
native UI.

For install/update/uninstall details, KPM commands, and manual recovery, follow
[BUILD_DEPLOY.md](docs/BUILD_DEPLOY.md) and
[EINK_LIFECYCLE.md](docs/EINK_LIFECYCLE.md). Recovery should preserve
`puzzles/` and `state/`; do not delete user data to repair a runtime install.

The first toolbar button is an icon-only book toggle for analysis. Tap it again to
return to the live puzzle board. It is inverted while analysis is open and disabled
for collections without rich analysis. The solution panel shows compact inline PGN
movetext: numbered SAN moves, parenthesized variations, and braced comments.
Standard move-quality annotations appear as `!`, `?`, `!!`, `??`, `!?`, `?!`
attached to the move; other annotation codes retain their `$N` form. Bold SAN tokens and explicit prose move references are tappable.
Selected moves have an inverted rectangular background; plain prose remains inert.
Overflow uses previous/next analysis pages.
Analysis opens directly on solution text without a separate title/page-count line.

Phase two keeps the app a physical-board puzzle tool rather than a chess engine. Existing
version-1 collections remain valid. Rich solution data is designed as an additive extension
that retains the legacy `solution` main line, with precomputed positions for interactive
browsing. PGN remains an offline authoring/import format. Stylus tap support for the rich
solution browser is explicitly part of the phase-two plan.
