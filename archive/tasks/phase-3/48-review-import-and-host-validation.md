# Task 48 — Add the review import/update workflow and host validation

**Status:** Implemented
**Working branch:** `main` (explicit user request overrides the phase-three branch policy)
**Depends on:** Tasks 41–47  
**Primary area:** docs, tooling workflow, end-to-end host tests  
**Validation:** Automated only  
**Physical Scribe required:** No

## Outcome

Make importing/updating a user's games or annotated master games repeatable, and prove converter -> runtime -> renderer behavior end to end before device testing.

## Required behavior

- Document a one-command PGN -> review JSON workflow using the Task-42 converter.
- Document the Kindle `games/` directory and filename convention.
- Safe overwrite/update behavior validates the new output before replacing an existing review file.
- Generated IDs remain stable across annotation-only updates; collisions fail clearly.
- Add an end-to-end fixture from annotated PGN through converter output, core parsing, review main-line traversal, variation selection, renderer move targets, and exact stored FEN previews.
- Verify PREV/NEXT across the full main line and selected variation -> main-line return semantics.
- Verify review FREE scratch/reset/exit semantics.
- Verify switching back to an in-progress puzzle restores exact state and identical serialized puzzle progress bytes.
- Verify STANDARD/SMALL review snapshots and regional damage reconstruction.
- Run the repository full host gate and Kindle cross-build gate.
- Update README/build-deploy docs with import/use instructions, but do not claim physical acceptance.

## Non-goals

No device validation, no engine analysis, and no durable review resume.

## Suggested commit

`docs: add game review import and host validation`

## Implementation record — 2026-10-07

- Added [the import/update guide](../../docs/GAME_REVIEW_IMPORT.md), README use
  instructions, build/deploy data layout and setup, and host-test documentation.
- Added a deterministic annotated Unicode PGN -> JSON fixture and a Rust test
  that invokes the converter and passes its actual output through runtime
  parsing, full PREV/NEXT traversal, every rendered move/reference target, exact
  stored FEN previews, and variation return navigation in STANDARD/SMALL layouts.
- Verified FREE scratch/reset/exit and navigation resets, immutable game data,
  restoration of an in-progress graded puzzle (after its automatic reply), and
  identical serialized puzzle progress. Regional damage replay reconstructs
  every frame; both new Scribe-resolution snapshot frames were visually inspected.
- Specified the previously uncaught atomic-write error with a failing test, then
  handled it as a clear CLI error. Regression tests cover annotation-only stable
  IDs, duplicate generated/explicit IDs, illegal and oversized updates, and
  failed rename cleanup preserving the prior file.
- Passed `scripts/check.sh` (format, clippy, workspace tests, Python/tooling,
  generated assets, platform and packaging contracts) and
  `scripts/build-kindle.sh` (repeated host gate plus pinned ARMv7 release build
  and ELF/glibc-2.35 validation).
- No physical Scribe test or deployment was performed. Task 49 remains pending.
