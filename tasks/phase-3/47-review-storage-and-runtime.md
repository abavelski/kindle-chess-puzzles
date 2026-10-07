# Task 47 — Add Kindle review-library storage and runtime wiring

**Status:** Implemented  
**Working branch:** `main` (explicit user-requested exception)  
**Depends on:** Tasks 43–46  
**Primary area:** `kindle-platform` storage + Kindle binary effects  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Discover review JSON files from a separate Kindle directory, expose a flat game library to core, and load selected games without affecting puzzle persistence.

## Required behavior

- Add default review directory `/mnt/us/kindle-chess/games` and an environment override for tests/development.
- Discover only `games.json` and `games-*.json`.
- Parse files with the core review parser and expose valid game metadata plus invalid-file diagnostics.
- Flatten valid games across files into stable picker entries keyed by filename + game ID.
- Do not install a bundled review file automatically; an empty directory is a valid state.
- Wire core review-game requests to platform loading/activation, analogous to collection requests but separate from puzzle collection state.
- Startup remains successful when the review directory is absent, empty, or contains invalid files.
- Review actions never dirty or rewrite puzzle `progress.json`.
- Settings persistence stays shared and unchanged.
- Source review files are read-only from the app's point of view.
- Log discovered review files/games with useful context without leaking platform paths into core.

## Tests

Temporary-directory tests cover discovery/sorting, multiple files/multiple games, duplicate IDs scoped safely by file key, malformed file isolation, missing directory, environment override, and puzzle-progress byte stability while switching/navigating reviews.

## Non-goals

No review resume persistence and no packaging UI.

## Suggested commit

`kindle: load offline game review library`


## Implementation record

- Added the default `/mnt/us/kindle-chess/games` review directory plus `KINDLE_CHESS_REVIEW_DIR` override without changing puzzle progress/settings paths.
- Added sorted discovery for only `games.json` and `games-*.json`, parsing each file with `chess_core::parse_review_file`, flattening games by filename + game ID, and isolating invalid-file diagnostics.
- Kept missing/empty/invalid review directories non-fatal and never installs or writes a bundled review file.
- Routed picker selection through a review-specific core effect; the Kindle binary reloads the selected source game and activates it without emitting or persisting puzzle progress.
- Added temporary-directory and runtime regressions for sorting/filtering, multiple files/games, cross-file duplicate IDs, malformed isolation, missing directories, environment override, read-only source files, and byte-stable `progress.json` during review switching/navigation.
- Landed directly on `main` because the user explicitly requested a branch-policy exception.
