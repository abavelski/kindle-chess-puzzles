# App data separation plan

Updated: 2026-11-09. Status: DATA-01 and DATA-02 Done. Device migration and both deployments verified.
Follow-up to the [completed app split](../archive/tasks/implemented/app-split.md).

## Goal and current state

Each app should use only its own data root:

| App | Root | Data |
| --- | --- | --- |
| Puzzles | `/mnt/us/kindle-chess` | `puzzles/`, progress, settings, runtime and logs |
| Game Review | `/mnt/us/kindle-game-review` | `games/`, review resume, settings, runtime and logs |

DATA-01 removed automatic legacy reads; each app now reads and writes only its
own configured paths. Before migration, the last device audit
found 112 game files under Puzzles and newer Review settings/resume under Review.
Preserve those newer files when moving the legacy data.

This is a small personal app. Keep deployment and migration to running the
scripts and checking the results. Data can be restored and earlier versions
rebuilt from Git. No new backup system, transaction journal, rollback command,
exhaustive failure matrix or extra approval stages are required. Keep basic
protections against overwriting existing files or touching unrelated puzzle data.

## Tasks

| ID | Deliverable | Depends on | Status |
| --- | --- | --- | --- |
| DATA-01 | Independent app data access, tests and builds | — | Done |
| DATA-02 | Device migration script and result verification | DATA-01 | Done |

Use a task branch. Update the table and task status together, recording the
implementation commit and test results. DATA-01 owner: Codex; branch: `task/data-01-independent-storage`; started 2026-11-08.

## DATA-01 — Separate data access and run the tests

**Status:** Done (2026-11-08). Owner: Codex; branch: `task/data-01-independent-storage`; started 2026-11-08.

Implementation: `69b0749`. Focused test-first evidence: three regression tests failed with legacy fallback enabled; all six `review_migration` tests and the `products` startup/persistence test passed after removal. `scripts/check.sh` passed with the pinned `requirements-tools.txt` dependency in `.venv`. Both apps passed pinned build, stage and `verify_app_artifacts.py` locally (ARMv7, glibc 2.35, Rust 1.85.1, Zig 0.13.0). Artifacts are under `target/kindle/{puzzles,review}`. No device access or deployment.

- Remove automatic legacy fallback from `crates/kindle-app/src/products/mod.rs`
  and `products/review.rs`, including normal startup's use of
  `KINDLE_GAME_REVIEW_LEGACY_ROOT`. Keep the existing app-specific path overrides.
- Review reads only its own games, settings and resume. Missing inputs use its
  empty-library warning or defaults. Puzzles continues using its existing root.
  Neither app reads or changes the other's data during ordinary operation.
- Keep the settings schema, game IDs, resume format, package identities and
  Chessnut behavior unchanged. There is no need to simplify the legacy settings
  envelope as part of this task. Retain existing corrupt/future-file protection.
- Update the relevant startup/storage tests: missing Review inputs must not fall
  back to populated Puzzles data; each app's writes and relaunch stay independent;
  puzzle progress and existing Review state remain intact. Reuse existing fixtures
  and regression coverage instead of creating another test framework.
- Update architecture, build/deploy, Review import and settings docs to explain
  independent storage and the one-time migration script.

Start with focused failing tests, make them pass, then run the existing checks and
build/stage both apps:

```sh
scripts/check.sh
scripts/app.sh build puzzles
scripts/app.sh stage puzzles
python3 tools/verify_app_artifacts.py --app puzzles
scripts/app.sh build review
scripts/app.sh stage review
python3 tools/verify_app_artifacts.py --app review
```

Use the existing pinned toolchain and host dependencies. Fix relevant failures;
record the results. Once these pass, DATA-01 is Done. Device migration is DATA-02.

## DATA-02 — Implement the migration script and verify the result

**Status:** Done (2026-11-09). Owner: Codex; branch: `task/data-02-migration-script`; started 2026-11-08. Device migration/deployment authorized 2026-11-09.

Implementation: `15bc2e8` (on top of DATA-01 `69b0749` and evidence `8c20e54`).
The migration test initially failed because the tool was absent; all four focused
tests now pass: ordinary migration/rerun, newer Review state/identical game,
conflicting game/invalid state, and active-app refusal. Tests execute the actual
shell file operations against temporary local directories, use the existing Rust
settings/resume parsers, and assert unrelated Puzzles bytes stay unchanged.
`scripts/check.sh` passed, including these tests. Both apps were rebuilt, staged
and verified again locally with the pinned toolchain because the new helper,
tool, tests and check script change the source receipts. Final artifacts are in
`target/kindle/{puzzles,review}`. Host preparation on 2026-11-08 used no device
access. On 2026-11-09 the owner authorized migration and deployment: all 112 games
moved with matching hashes, newer Review settings/resume stayed unchanged, and
the superseded legacy resume was removed. A rerun reported zero moves/issues.
Both verified packages were installed and their runtime/scriptlet bytes checked.
Device launches/framebuffer captures showed Review restored at Euwe–Reshevsky,
ply 16 Nc6, and Puzzles at puzzle 506w (261/500). Both exited cleanly; all 123
library/state/identity files were unchanged after launch, including puzzle
progress and both apps' settings. Firmware: 5.19.6; prepared version: `1d04599`.
See [deployment evidence and observations](device/ks1-barolo.md#app-data-separation--migration-and-deployment-2026-11-09).
These are actual device runtime/framebuffer observations; physical panel quality,
touch navigation and Bluetooth were not retested.

Provide one host-run command, proposed as:

```sh
python3 tools/migrate_app_data.py --host root@DEVICE_IP --port 2222
```

Use the established SSH transport and the Kindle's existing shell utilities.
Do not add a device Python dependency. The command performs the migration and
prints what moved, what was already present and anything requiring attention.

- Require both apps to be closed; refuse to run if either is active. Do not add
  a new locking system or automatically kill apps.
- Move legacy game files from Puzzles' `games/` into Review's `games/`, preserving
  names and bytes. Check the transferred files before removing their old copies.
  Existing identical files are already migrated; different files with the same
  name must be reported without overwriting either. A rerun should handle files
  already moved without error. No journal is needed.
- Keep existing Review settings and resume. If absent, copy the corresponding
  legacy files, retaining Review's settings profile and saved move. Leave
  malformed or unsupported state untouched and report it using existing parsing
  rules. Do not change the settings schema.
- Remove the obsolete legacy review resume once its contents are migrated or
  superseded by existing valid Review state. Leave unresolved files in place and
  report them. Leave Puzzles collections, progress, settings, Chessnut config,
  runtimes and unrelated files alone.
- Add a few focused tests using temporary directories: ordinary migration,
  existing newer Review state, conflicting filenames and running the script
  twice. Check that Puzzles data is unchanged. Add the tests to `scripts/check.sh`
  and run that gate. Reuse DATA-01's builds unless build inputs changed.

With both apps closed, run migration, then deploy the DATA-01 builds using the
existing deploy scripts. Verify the resulting paths and game-file count, launch
Review and check its library and saved position, then launch Puzzles and check
collections and progress. Record the commands, deployed version and observed
results in [the device notes](device/ks1-barolo.md). Mark DATA-02 Done when those
checks pass; until device verification, use Waiting for device.

If something fails, fix the reported problem and rerun, or restore the data and
redeploy an earlier Git version. No automated rollback feature or recovery drill
is required.

Related docs: [architecture](ARCHITECTURE.md), [build/deploy](BUILD_DEPLOY.md),
[Review import](GAME_REVIEW_IMPORT.md), [settings](SETTINGS.md),
[testing](TESTING.md).
