# Task 28 — Validate and close phase two

**Status:** Ready  
**Depends on:** Tasks 20-27  
**Primary area:** integration + physical Scribe validation

## Outcome
Prove rich book solutions work end to end without regressing phase-one solving/progress.

## Automated gate
Run fmt, strict clippy, workspace tests, snapshots, platform/packaging tests, and documented ARMv7/glibc build. Add end-to-end coverage importing PGN, parsing JSON, selecting main/alternative/sideline nodes, verifying exact displayed FEN, and proving progress bytes remain unchanged.

## HUMAN CHECKPOINT 28A
Use a representative real book collection with nested variations/explanations and verify: finger and stylus move taps; side-line/alternative board jumps; readable pagination; close restores live solve board; browsing alone does not mark progress; solving/wrong rollback/promotion/Free Board/navigation/collections/restart/X exit remain correct; phase-two work did not alter Task-10 ghosting policy.

## Closure
Record phase-two results, mark each task Implemented only when its own criteria pass, archive Tasks 20-28, and update README from planned to implemented.

## Suggested commit
`docs: close rich solution browsing phase`
