# Task 48 — Add the review import/update workflow and host validation

**Status:** Ready  
**Working branch:** `game-review`  
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
