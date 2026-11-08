# App data separation plan

Date: 2026-11-08. Status: Planned; implementation and device migration have not started.

## Goal and current state

Keep each app's library and persistent state within its own data root:

| App | Root | Owned data |
| --- | --- | --- |
| Kindle Chess Puzzles | `/mnt/us/kindle-chess` | `puzzles/`, `state/progress.json`, `state/settings.json`, runtime and logs |
| Kindle Game Review | `/mnt/us/kindle-game-review` | `games/`, `state/review-resume.json`, `state/settings.json`, runtime and logs |

The 2026-11-08 source and SSH audit confirmed that writes, runtimes and logs are
already separate. However, Review's 112 game files remain in
`/mnt/us/kindle-chess/games`; its own games directory is absent. Review reads
those files through automatic legacy fallback. Puzzles has five collection files
and retains an older Review resume. Review already has newer settings and resume
in its own root; those must take precedence throughout migration.

## Planned work

1. **Migrate the game library.** With both apps closed, inventory and hash the
   current files. Copy games into Review's own `games/` directory, preserving
   filenames, IDs and bytes. Refuse conflicting destination files rather than
   overwriting them. Verify all files and the saved review position before
   removing the legacy copies. Make interrupted migration safe to resume and
   retain a temporary verified recovery copy until acceptance.

2. **Make legacy migration explicit.** Remove automatic cross-app fallback from
   normal Review startup. Currently `products/mod.rs` defaults the legacy root
   to Puzzles, and `products/review.rs` falls back independently for games,
   settings and resume. Provide an explicit migration path for older installs;
   ordinary startup must use only Review's root, including when its library or
   state is missing. Keep corrupt and future-schema files protected.

3. **Remove obsolete Review data from Puzzles.** After migration and acceptance,
   remove the verified legacy game copies and superseded
   `kindle-chess/state/review-resume.json`. Preserve puzzle progress, puzzle
   collections and settings byte-for-byte. Do not replace Review's newer resume
   with the older legacy file or discard malformed/future-schema state silently.

4. **Update tests and documentation.** Add failing tests before implementation
   for startup without a Puzzles root, missing Review inputs without fallback,
   migration conflicts/interruption, stable IDs and resume, protected invalid
   bytes, and unchanged puzzle data. Update architecture, build/deploy and Review
   import instructions to describe explicit migration and independent storage.

5. **Consider settings simplification separately.** Both apps currently store
   the legacy settings envelope containing both product profiles. Files are
   already independent, so a schema change is optional and unnecessary for
   folder separation. Any later simplification needs compatibility fixtures.

## Boundaries and completion checks

Keep the private Puzzles-only Chessnut configuration under
`/var/local/kindle-chess/` with its existing restricted permissions. Library
scriptlets remain under `/mnt/us/documents`; process locks and the shared global
display lock remain outside the data roots. These are intentional exceptions.

Use a task branch for implementation. Run focused tests, `scripts/check.sh`, and
both product builds/stages for the changed release; apply pinned packaging checks
when packaging changes. On Kindle, verify Review's complete library, navigation,
current resume and settings with legacy fallback disabled, then verify Puzzles'
collections, progress and settings. Compare protected file hashes, record artifact
hashes, firmware and recovery evidence in the device notes, and obtain owner
acceptance before marking the migration complete or removing recovery copies.

Related contracts: [architecture](ARCHITECTURE.md),
[legacy fallback and rollback](BUILD_DEPLOY.md#standalone-review-data-and-legacy-fallback--split-09),
[Review import](GAME_REVIEW_IMPORT.md), and [testing](TESTING.md).
