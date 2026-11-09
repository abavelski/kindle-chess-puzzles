# Documentation index

**Current state (2026-11-08):** Puzzles and Game Review are independently built, installed and launched Kindle apps sharing chess, e-ink and Kindle runtime components. Both were deployed on Scribe firmware 5.19.6 and accepted by the device owner. Kobo remains deferred. The active follow-up is [app data separation](DATA_SEPARATION_PLAN.md).

## Current reference

- [App data separation plan](DATA_SEPARATION_PLAN.md): runtime isolation and test/build gates, followed by an explicit device migration script.
- [Completed split implementation](../archive/tasks/implemented/app-split.md): archived task statuses, host/build evidence and device acceptance.
- [Architecture](ARCHITECTURE.md): current boundaries, dependencies, storage and lifecycle invariants.
- [Portability and reuse](PORTABILITY_AND_APP_REUSE.md): product-first decision and deferred device portability.
- [Build and deploy](BUILD_DEPLOY.md): pinned per-product Scribe pipeline.
- [Testing](TESTING.md): automated and human validation gates.

## Active contracts

- [Kindle Chess Play improvements](KINDLE_PLAY_IMPROVEMENTS.md): staged close icon, both-color support, clickable move history, and required device acceptance. Companion [desktop server plan](https://github.com/abavelski/chess-games-db/blob/master/docs/tasks/kindle-play-server/IMPROVEMENTS_PLAN.md).

- Puzzles: [format](PUZZLE_FORMAT.md) and [collection updates](COLLECTION_UPDATES.md).
- Review: [behavior](GAME_REVIEW.md), [file format](GAME_REVIEW_FORMAT.md), [import/update](GAME_REVIEW_IMPORT.md).
- UI settings: [settings](SETTINGS.md).
- Physical board: [Chessnut Move](PHYSICAL_BOARD.md).
- Kindle: [Scribe notes](KINDLE_SCRIBE.md), [lifecycle](EINK_LIFECYCLE.md), [FBInk](FBINK.md), [measured device record](device/ks1-barolo.md).
- Licenses/source: [provenance](PROVENANCE.md).

[Historical documents and former tasks](../archive/README.md) are reference-only. Their old task statuses, branch instructions and relative links may no longer apply.
