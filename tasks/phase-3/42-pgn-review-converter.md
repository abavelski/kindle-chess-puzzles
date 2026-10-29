# Task 42 — Build the deterministic PGN review converter

**Status:** Implemented  
**Working branch:** `main` (explicit user-requested exception)  
**Depends on:** Task 41  
**Primary area:** host tooling only  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Convert ordinary single- or multi-game PGN into the Task-41 review JSON without requiring puzzle-specific tags.

## Required behavior

- Reuse/refactor the existing python-chess tree conversion instead of maintaining two independent PGN walkers.
- Parse standard headers, starting FEN/SetUp, main line, recursive variations, comments, and NAGs.
- Emit SAN/UCI and complete post-move FEN for every node.
- Preserve deterministic child order and deterministic JSON encoding.
- Preserve explicit structured move-reference directives; never infer links from SAN-looking prose.
- Generate a stable game ID by default from canonical identity/move data; annotations-only edits should not unnecessarily change identity.
- Allow an explicit PGN tag/CLI option to override the generated ID.
- Support multiple games in one input PGN and reject duplicate resulting IDs with context.
- Preserve standard display metadata and optional source metadata.
- Report encoded size and fail before writing partial output on malformed/illegal games.
- Do not change puzzle-converter output for existing inputs.

## Tests

Fixtures cover a user's normal PGN with no custom tags, annotated grandmaster games, multiple games, RAVs, Unicode metadata, custom FEN, generated-ID stability, explicit-ID override, duplicates, malformed input, and byte-stable output. Existing puzzle-converter tests remain green.

## Non-goals

No Rust/runtime changes and no engine evaluation generation.

## Suggested commit

`tools: add deterministic game review converter`


## Implementation record

- Added `tools/pgn_review_converter.py` for ordinary single- and multi-game PGN input with deterministic review JSON encoding and the frozen 8 MiB review limit.
- Reused the existing `python-chess` analysis-tree walker for main lines, recursive variations, SAN/UCI, complete post-move FEN, NAGs, comments, roles, and explicit structured move references.
- Generated stable default game IDs from canonical seven-tag identity metadata, starting FEN, and main-line UCI moves; comment, NAG, and variation-only edits therefore do not churn identity.
- Added `GameId`/configurable tag overrides plus a single-game `--game-id` CLI override, duplicate-ID diagnostics, source metadata support, and atomic output writes.
- Added Task-42 fixtures and automated tests, kept the existing puzzle converter fixture contract unchanged, and wired the new suite into `scripts/check.sh`.
- Landed directly on `main` because the user explicitly requested a branch-policy exception.
