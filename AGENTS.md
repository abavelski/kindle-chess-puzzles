# Agent instructions — Kindle chess monorepo

Updated 2026-11-08. Puzzles and Game Review are independent apps sharing chess, e-ink and Kindle runtime components. The [completed split implementation](archive/tasks/implemented/app-split.md) is archived after owner device acceptance. The active follow-up queue is [app data separation](docs/DATA_SEPARATION_PLAN.md). Prior phase branches and old tasks are history under [archive](archive/README.md).

## Before a task

Read README.md, [architecture](docs/ARCHITECTURE.md), [testing](docs/TESTING.md), and the **one** selected tracker task and its linked contracts. Select the first Ready task with all dependencies Done; set table and detailed status to In progress with owner/branch/start date. Work test-first; avoid unrelated cleanup or later tasks. Archived phase-2 and game-review branch policies are no longer active.

Use a small task branch/PR unless the owner explicitly requests main. The owner authorized the completed split merge and push to main; future implementation should use task branches unless explicitly directed otherwise.

## Keep the process simple

This is a personal app with restorable data. Use the existing test/build/deploy scripts and verify results. Do not add backup systems, transaction journals, automated rollback or exhaustive failure testing unless requested. Basic conflict checks and preserving unrelated data are sufficient for migration.

## Invariants

- Products own behavior/actions/persistence; generic chess model/UI depend on neither product nor platform. Kindle is the device I/O adapter. Do not copy the combined runtime into a second app.
- Preserve v1 puzzle format, grading, progress IDs, game JSON, review navigation and resume, workspace settings, explicit interactive move references, grayscale snapshots and damage reconstruction. Review must never mutate puzzle progress.
- Keep PGN conversion and chess move legality offline. No runtime chess engine or parser.
- Chessnut remains Puzzles-only initially; preserve optional connection, manual Send, seating, input safety, worker and licensing. See [physical board](docs/PHYSICAL_BOARD.md). Do not claim device verification from tests.
- The two apps must have distinct package IDs, binary names, runtime roots, scriptlets, settings, logs and locks. Kindle framebuffer and stock UI are **global exclusive hardware**: use shared ownership locking and robust cleanup/SSH recovery.
- Never destroy or silently rewrite user-owned puzzles, games, progress, review resume or settings. Preserve corrupt and future-schema bytes; use focused migration tests.
- Keep unsafe FFI isolated in fbink-sys; keep coordinates and refresh hardware policy at the boundary. Preserve pinned Scribe ARMv7/glibc 2.35, FBInk, Zig and worker source checks.
- Before marking Done: focused failing and passing tests, scripts/check.sh, and pinned build/stage when packaging changes. Once standalone Review exists, test both app targets. Human checkpoint work stays Waiting for device until the deployed version and observed results are recorded in [device notes](docs/device/ks1-barolo.md).

Update both task status locations and evidence in the **same commit**. Historical original instructions are in [archive/AGENTS_PRE_SPLIT.md](archive/AGENTS_PRE_SPLIT.md).
