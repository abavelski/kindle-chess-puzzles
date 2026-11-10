# Architecture — independent Kindle chess apps

Updated 2026-11-08. The split is implemented and device-accepted by the owner. See [the archived implementation](../archive/tasks/implemented/app-split.md) for evidence. The [former architecture](../archive/docs/ARCHITECTURE_PRE_SPLIT.md) is retained for provenance.

## Current system (from source)

The eleven Cargo workspace members are chess-model, eink-ui, chess-core, chess-render, fbink-sys, kindle-platform, kindle-app, the compatibility app/kindle-chess, and independent apps/kindle-puzzles, apps/kindle-game-review and apps/kindle-chess-play. SPLIT-02 extracts Board/FEN/UCI and generic analysis graph validation into chess-model, which depends only on serde_json; chess-core re-exports those types for compatibility. Puzzle legacy-solution projection stays in puzzle.rs and the non-empty review main-line rule stays in review.rs. Two distinct product entrypoints now construct only their own product repositories/state and call the common runtime. The product-aware pipeline now stages both independently: Puzzles keeps installed binary kindle-chess and its legacy package identity, while Review uses kindle-game-review and its own package/root. The combined Cargo entrypoint is retained only as a regression target. Standalone screens have no workspace switch; combined regression snapshots retain it. Its combined product adapter now runs through kindle-app::runtime, which owns the shared input/power/display loop; the common supervisor and runtime enforce global display ownership. The compatibility main.rs still opens both libraries/stores and routes one AppState for regression testing, while the shared runtime owns the device loop. Standalone Review never constructs puzzle state/progress or a board service; standalone Puzzles never loads review games/resume.

- crates/chess-core/src/puzzle_app.rs owns puzzle state, typed actions/effects, grading, collections/progress and physical-board intent without a review library/session. crates/chess-core/src/review_app.rs owns a separate games-only ReviewApp with typed actions, library/picker, scratch board, preferences and resume; neither product state owns the other. The transitional app.rs delegates to both and retains only workspace composition plus the legacy settings envelope; review_state.rs and review_resume.rs offer separation seams. Its analysis.rs now re-exports the generic model; product parsers own their additional validation.
- crates/eink-ui owns product-neutral Gray8/geometry, embedded fonts/pieces, board and analysis widgets and damage calculations. It depends only on chess-model/fontdue. chess-render owns product layouts and hit targets, with read-only ScreenState implementations for PuzzleState, ReviewApp and the transitional AppState. Typed product hit mappings reject workspace toggles and actions from the other product; the legacy combined API stays available for regression snapshots.
- crates/kindle-platform now depends only on eink-ui and fbink-sys and provides display/input/power, opaque atomic file operations and generic refresh requests. kindle-app owns puzzle/review repositories, protected progress/settings/resume stores, chess-aware refresh decisions and the Puzzles board-session adapter over the native worker transport. Product state remains platform-neutral in chess-core.
- tools/kindle_build.py and packaging/kindle select product-specific executables, output directories, build receipts, KPM IDs, scriptlets, roots and runtimes; both use the shared display-owner lock.
- support/chessnut-move is a separately built native worker, excluded from the Cargo workspace.

## Product-first monorepo

Implemented dependency boundaries retain coherent product modules in chess-core while generic model/UI and device mechanics are separate crates:

| Layer | Current location | Owns |
| --- | --- | --- |
| Reusable chess model | crates/chess-model | Board, FEN, UCI, generic analysis tree and validation |
| Reusable UI | crates/eink-ui | Gray8 canvas, geometry, piece artwork, board/text/analysis widgets, damage |
| Puzzles | crates/chess-core product modules (puzzle_app, puzzle, progress, collection, board_input/session) | Puzzle format, grading, collection/progress, actions, puzzle screen |
| Review | crates/chess-core product modules (review_app, review, review_state/resume) | Review format, navigation, resume, actions, review screen |
| Device | crates/kindle-platform and fbink-sys | FBInk, input, power, generic paths/files, raw BLE transport |
| Products on Kindle | apps/kindle-puzzles, apps/kindle-game-review and crates/kindle-app composition | Thin product entrypoints, repositories and shared device runtime |

Preserve coherent existing crates if interfaces are demonstrably clean; avoid a forest of tiny crates. Binaries must be **different entrypoints**, not duplicate names or a hidden workspace toggle in the old binary.

### Dependency and behavioral rules

1. Generic model has no product or platform dependency. Move the rule relating rich analysis to legacy puzzle solutions into puzzle validation. Keep the offline PGN converter; no engine/PGN parser on device.
2. Generic UI accepts product-neutral view data/hit targets. Each product maps targets to its own typed actions. Both reuse the same deterministic canvas, text and board renderer.
3. Products own behavior and persistence schemas but receive storage/display/event services from composition. Product crates never depend on Kindle or FBInk.
4. Platform crates expose device mechanics; they must not parse Puzzle, ReviewGame, AppState or Progress. Keep e-ink refresh safety and Scribe input/power behavior.
5. Only Puzzles initially owns Chessnut board logic and worker. The Review app must not connect or send on launch, and the two apps must not concurrently own the Bluetooth device.

### Separate identities and migration

Preserve Puzzles installation identity: the existing KPM ID kindle_chess, library scriptlet kindle-chess.sh and data root /mnt/us/kindle-chess. Review identity: kindle_game_review, kindle-game-review.sh and /mnt/us/kindle-game-review. Each needs its own executable, manifest, receipt, staged artifacts, install/uninstall lifecycle, state/settings, logs and process lock.

Standalone Review reads games, settings and resume only from `/mnt/us/kindle-game-review`; Puzzles keeps `/mnt/us/kindle-chess`. Missing Review inputs use defaults or the empty-library warning, with no automatic legacy fallback. Existing malformed/future files remain protected. The old combined app's games and review resume under `/mnt/us/kindle-chess` require explicit one-time migration; [DATA-02](DATA_SEPARATION_PLAN.md#data-02--implement-the-migration-script-and-verify-the-result) tracks the migration script and device verification. Preserve filenames, game IDs, saved moves, Review's newer settings/resume and all unrelated puzzle bytes.

The framebuffer and native UI are global hardware resources: install paths are independent but simultaneous launches must be rejected by a **shared display-owner lock**. Supervisors must restore native Xorg/awesome and support documented SSH recovery on all exits. Do not let two scripts seize the framebuffer.

The currently stable target remains Scribe gen 1, ARMv7 hard float, glibc 2.35, pinned FBInk/Zig and the native Chessnut worker. Receipts and signatures must attest the **selected product**, correct binary, source and staged package; different apps must not overwrite artifacts.

## After product split: hardware portability

Kobo and other devices are a second axis. Only then introduce platform events such as tap, page-forward/back, sleep/wake, and independent framebuffer/device profiles. Do not infer rotation, input geometry, button mappings or FBInk ABI from product descriptions: probe hardware and record findings.

[Portability decisions](PORTABILITY_AND_APP_REUSE.md) | [Task sequence](../archive/tasks/implemented/app-split.md) | [Build/deploy](BUILD_DEPLOY.md)

## Play MVP (owner accepted 2026-11-09)

`chess-core::play` owns transient Play state and validated authoritative server
snapshots. `chess-render::play` supplies a dedicated screen/hit path without
extending the persisted Workspace schema. `kindle-app::products::play` composes
that state with a bounded HTTP worker and the same foreground device runtime.
Play alone calls the trusted-LAN desktop HTTP API; chess legality and Maia stay
on the desktop. It reads only `/mnt/us/kindle-chess-play/config.json`, writes no
save-game state, and never opens either older product's stores. Its own binary,
root, package, scriptlet, logs and process lock share the existing global display
owner lock. See [Play's tracker and operator setup](KINDLE_PLAY_MVP.md).

The post-MVP Play feature branch adds human-side selection and a transient
read-only history cursor. Server snapshots remain the immutable live authority;
Start/past-ply views reuse the same authored UCI replay as snapshot validation,
and Latest restores the exact server FEN. Analysis and Play share neutral move
chip/pager painting through `eink-ui::move_list`; no Review tree or persisted
settings are reused for Play. See the [complete-build improvement tracker](KINDLE_PLAY_IMPROVEMENTS.md).
