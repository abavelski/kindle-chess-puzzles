# Task 06 — Add puzzle collection discovery and durable progress

**Status:** Implemented
**Depends on:** Tasks 01-05

## Outcome

Load/edit multiple v1 puzzle collections on Kindle and persist learning progress separately, matching reference semantics.

## Automated implementation complete

Task 06 now adds:

- configurable Kindle storage defaults at `/mnt/us/kindle-chess/puzzles` and `/mnt/us/kindle-chess/state/progress.json`, with `KINDLE_CHESS_PUZZLE_DIR` and `KINDLE_CHESS_PROGRESS_FILE` overrides for host tests and alternate deployments;
- deterministic collection discovery, title fallback, invalid-file error entries, bundled `puzzles.json` installation when no matching files exist, and in-memory bundled fallback without overwriting invalid user collections;
- a touch-sized paged Scribe collection picker that activates a file only after successful parsing and leaves the current valid board unchanged on failure;
- durable version-1 progress for active file, per-file current puzzle ID, and solved IDs, with protected corrupt/future records and same-directory temp + sync + rename writes;
- dirty-save retry semantics that retain the newest in-memory progress after a write failure and surface a visible warning;
- host coverage for discovery, fallbacks, collection switching/restoration, protected progress, write-failure retry, picker hit testing, and picker/error/warning render snapshots.

The ARMv7 cross-build also publishes a `kindle-chess-task06-armv7` checkpoint artifact. HUMAN CHECKPOINT D passed on the physical Scribe on 2026-10-02; deployment, persistence, computer-copy, invalid-file, and recovery evidence is recorded in `docs/device/ks1-barolo.md`.

## Storage interface

Keep filesystem code in the Kindle platform layer.

Core parser/progress model consumes bytes.

Paths must be configurable so tests use temporary directories and a future Kobo adapter can choose another location.

The Task 06 Kindle defaults are:

- puzzle collections: `/mnt/us/kindle-chess/puzzles`;
- app-owned progress: `/mnt/us/kindle-chess/state/progress.json`.

Earlier device work verified `/mnt/us` as usable for app staging. HUMAN CHECKPOINT D verified the exact Task 06 subdirectories and progress file survive the intended exit/relaunch and computer-copy workflow on-device. Environment overrides keep host tests and alternate deployments off these production defaults.

## Collection discovery

Discover filenames matching:

- exactly `puzzles.json`;
- `puzzles-<nonempty>.json`.

Sort lexicographically.

Ignore unrelated files.

Read enough metadata to show each collection title when valid, otherwise filename/error state.

If no collection exists:

- create/copy bundled examples as `puzzles.json`, then load it.

If matching files exist but none are valid:

- do not overwrite them;
- show bundled examples in memory;
- surface the error.

Selecting an invalid second collection must leave the currently active valid board/collection unchanged.

## Picker

Render/use a collection picker suitable for the Scribe.

Selecting a valid collection:

- activates it only after parse success;
- restores its remembered puzzle ID if valid;
- otherwise selects index 0;
- resets transient attempt state;
- preserves session mode/orientation-lock behavior as specified by core tests.

## Progress

Persist the v1 logical record documented in `docs/PUZZLE_FORMAT.md`.

Remember:

- active collection filename;
- current puzzle ID per collection;
- solved IDs per collection.

Requirements:

- uploaded puzzle JSON is never rewritten for progress;
- completing a solution marks that puzzle solved;
- already-solved completion is idempotent;
- Reset/Free Board do not change solved IDs;
- switching files saves old/current IDs correctly;
- restart restores active file/current puzzle when still present;
- missing remembered file/puzzle falls back safely;
- malformed/future progress is not silently overwritten;
- write failure keeps newest progress in memory and shows a warning;
- later mutation retries a dirty save.

## Safe writes

Implement an app-owned progress write strategy appropriate to the verified filesystem.

Prefer:

1. write temp file in same directory;
2. flush/sync as appropriate;
3. rename/replace atomically when supported;
4. never truncate a future/corrupt progress record before user-deliberate recovery.

Tests use fault injection or a fake storage layer for write/rename failure.

## Automated acceptance tests

Cover:

- discovery filtering/order;
- title fallback;
- no-file bundled default;
- invalid-only preservation;
- successful second-file switch;
- failed second-file switch;
- per-file current puzzle restoration;
- active file restoration after simulated restart;
- solved marker per file;
- no duplicate solved IDs;
- progress save failure/dirty retry;
- corrupt/future progress protection;
- picker/error/warning render snapshots.

## HUMAN CHECKPOINT D

Using at least two copied collections:

1. switch collections;
2. browse to different puzzle indexes in each;
3. solve one puzzle in each;
4. exit/relaunch;
5. confirm active collection, current puzzle, and solved markers restore;
6. edit/add a collection from the computer and relaunch;
7. add malformed `puzzles-broken.json`, select it, and confirm the current valid board remains active;
8. remove/fix the malformed file without losing progress.

## Acceptance criteria

- collection/persistence behavior matches the reference semantics;
- data path is documented;
- puzzle source files remain untouched by progress;
- host tests can run entirely in temp directories;
- physical restart/persistence test passes.

## Suggested commit

`app: add Kindle puzzle collections and progress persistence`
