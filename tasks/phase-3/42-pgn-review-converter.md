# Task 42 — Build the deterministic PGN review converter

**Status:** Ready  
**Working branch:** `game-review`  
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
