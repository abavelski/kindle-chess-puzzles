# Documentation index

**Current state (2026-11-08):** one shipped Kindle Scribe binary containing Puzzles and Game Review workspaces. The next milestone is two separately built and installed Kindle apps, not Kobo. The split is planned, not yet implemented.

## Active planning
- [Codex task tracker](NEXT_TASKS_APP_SPLIT.md): task IDs, dependencies, statuses, tests, acceptance and handoff.
- [Architecture](ARCHITECTURE.md): actual current boundaries, target dependencies, storage and lifecycle invariants.
- [Portability and reuse](PORTABILITY_AND_APP_REUSE.md): product-first decision and deferred device portability.
- [Build and deploy](BUILD_DEPLOY.md): pinned current Scribe pipeline and app-selector transition.
- [Testing](TESTING.md): automated and human validation gates.

## Active contracts
- Puzzles: [format](PUZZLE_FORMAT.md) and [collection updates](COLLECTION_UPDATES.md).
- Review: [behavior](GAME_REVIEW.md), [file format](GAME_REVIEW_FORMAT.md), [import/update](GAME_REVIEW_IMPORT.md).
- UI settings: [settings](SETTINGS.md).
- Physical board: [Chessnut Move](PHYSICAL_BOARD.md).
- Kindle: [Scribe notes](KINDLE_SCRIBE.md), [lifecycle](EINK_LIFECYCLE.md), [FBInk](FBINK.md), [measured device record](device/ks1-barolo.md).
- Licenses/source: [provenance](PROVENANCE.md).

[Historical documents and former tasks](../archive/README.md) are reference-only. Their old task statuses, branch instructions and relative links may no longer apply.
