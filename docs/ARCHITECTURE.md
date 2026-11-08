# Architecture — current state and two-app target

Updated 2026-11-08. This describes a target and does **not** claim a completed split. Follow [the task tracker](NEXT_TASKS_APP_SPLIT.md) for implementation. The [former architecture](../archive/docs/ARCHITECTURE_PRE_SPLIT.md) is retained for provenance.

## Current system (from source)

The six Cargo workspace members are chess-model, chess-core, chess-render, fbink-sys, kindle-platform, and app/kindle-chess. SPLIT-02 extracts Board/FEN/UCI and generic analysis graph validation into chess-model, which depends only on serde_json; chess-core re-exports those types for compatibility. Puzzle legacy-solution projection stays in puzzle.rs and the non-empty review main-line rule stays in review.rs. The only shipped executable is kindle-chess. Its main.rs opens puzzle collections and review games, creates puzzle progress and review-resume stores, routes both workspaces through one AppState, handles Chessnut, and owns the Kindle input/display loop.

- crates/chess-core/src/app.rs mixes puzzle grading, review navigation, workspace switching and actions; review_state.rs and review_resume.rs offer separation seams. Its analysis.rs now re-exports the generic model; product parsers own their additional validation.
- crates/chess-render/src/layout.rs and render.rs mix generic Gray8, typography/board, damage tracking and product-specific targets.
- crates/kindle-platform/src/storage.rs knows puzzles, review libraries and progress; refresh.rs contains board-specific policy; chessnut.rs contains chess semantics mixed with a platform transport.
- tools/kindle_build.py and packaging/kindle currently pin one executable, output directory, build receipt, KPM ID, scriptlet, root path and runtime.
- support/chessnut-move is a separately built native worker, excluded from the Cargo workspace.

## Target product-first monorepo

A reasonable dependency shape, without forcing every name change at once:

| Layer | Proposed location | Owns |
| --- | --- | --- |
| Reusable chess model | crates/chess-model | Board, FEN, UCI, generic analysis tree and validation |
| Reusable UI | crates/eink-ui | Gray8 canvas, geometry, piece artwork, board/text/analysis widgets, damage |
| Puzzles | crates/puzzle-app | Puzzle format, grading, collection/progress, actions, puzzle screen |
| Review | crates/review-app | Review format, navigation, resume, actions, review screen |
| Device | crates/kindle-platform and fbink-sys | FBInk, input, power, generic paths/files, raw BLE transport |
| Products on Kindle | apps/kindle-puzzles and apps/kindle-game-review | Thin runtime composition and application identity |

Preserve coherent existing crates if interfaces are demonstrably clean; avoid a forest of tiny crates. Binaries must be **different entrypoints**, not duplicate names or a hidden workspace toggle in the old binary.

### Dependency and behavioral rules

1. Generic model has no product or platform dependency. Move the rule relating rich analysis to legacy puzzle solutions into puzzle validation. Keep the offline PGN converter; no engine/PGN parser on device.
2. Generic UI accepts product-neutral view data/hit targets. Each product maps targets to its own typed actions. Both reuse the same deterministic canvas, text and board renderer.
3. Products own behavior and persistence schemas but receive storage/display/event services from composition. Product crates never depend on Kindle or FBInk.
4. Platform crates expose device mechanics; they must not parse Puzzle, ReviewGame, AppState or Progress. Keep e-ink refresh safety and Scribe input/power behavior.
5. Only Puzzles initially owns Chessnut board logic and worker. The Review app must not connect or send on launch, and the two apps must not concurrently own the Bluetooth device.

### Separate identities and migration

Preserve Puzzles installation identity: the existing KPM ID kindle_chess, library scriptlet kindle-chess.sh and data root /mnt/us/kindle-chess. Proposed Review identity: kindle_game_review, kindle-game-review.sh and /mnt/us/kindle-game-review. Each needs its own executable, manifest, receipt, staged artifacts, install/uninstall lifecycle, state/settings, logs and process lock.

The legacy combined app stores games at /mnt/us/kindle-chess/games and resume at /mnt/us/kindle-chess/state/review-resume.json. A standalone Review app needs a **tested, non-destructive** copy or read-only fallback that preserves existing game IDs, resume and review settings. Never move/delete/overwrite those files implicitly; preserve puzzle progress bytes and malformed/future-schema files.

The framebuffer and native UI are global hardware resources: install paths are independent but simultaneous launches must be rejected by a **shared display-owner lock**. Supervisors must restore native Xorg/awesome and support documented SSH recovery on all exits. Do not let two scripts seize the framebuffer.

The currently stable target remains Scribe gen 1, ARMv7 hard float, glibc 2.35, pinned FBInk/Zig and the native Chessnut worker. Receipts and signatures must attest the **selected product**, correct binary, source and staged package; different apps must not overwrite artifacts.

## After product split: hardware portability

Kobo and other devices are a second axis. Only then introduce platform events such as tap, page-forward/back, sleep/wake, and independent framebuffer/device profiles. Do not infer rotation, input geometry, button mappings or FBInk ABI from product descriptions: probe hardware and record findings.

[Portability decisions](PORTABILITY_AND_APP_REUSE.md) | [Task sequence](NEXT_TASKS_APP_SPLIT.md) | [Build/deploy](BUILD_DEPLOY.md)
