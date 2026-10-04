# Task 27 — Define collection regeneration and update workflow

**Status:** Implemented  
**Depends on:** Tasks 20-22; Task 21 for generated collections  
**Primary area:** tooling/docs/compatibility tests  
**Validation:** Automated only  
**Physical Scribe required to complete:** No  
**Human interaction required to complete:** No  
**Completion gate:** Host tooling/fixture tests and documentation checks; no device test


## Device/human-testing rule
This task must not request Kindle access. ID/revision comparison, atomic output, and progress separation are host-side/tooling contracts.

## Outcome
Make regeneration/replacement of book JSON safe without losing durable puzzle identity or progress.

## Required behavior
- Document root schema `version`, analysis `version`, content `revision`, and Git-history roles separately.
- Keep collection filename stable across normal updates.
- Keep `PuzzleId` stable for the same exercise across reorder/comments/side-line edits.
- Require a new puzzle ID when starting position/meaning changes enough that solved state should not carry.
- Add converter/update comparison reporting added, removed, duplicate, and FEN-changed IDs before overwrite.
- Write generated output atomically and never touch Kindle progress.
- Report legacy 256 KiB and phase-two size thresholds.
- Cookbook: initial import, regenerate, add/remove puzzles, split oversized collection, Git rollback.

## Tests
Reorder/comment-only retains IDs; duplicate IDs fail; FEN change under same ID is highlighted; output never includes/mutates progress.

## Non-goals
No automatic solved-state invalidation or database migration service.

## Suggested agent
Good local-model task once converter/core contracts are stable.

## Suggested commit
`docs: add safe puzzle collection update workflow`

## Implementation record

Implemented on `phase-2` with deterministic collection comparison, duplicate blocking,
FEN-change reporting, same-directory atomic replacement, legacy/phase-two size reporting,
host tests for identity/progress separation, and the collection update cookbook in
`docs/COLLECTION_UPDATES.md`.
