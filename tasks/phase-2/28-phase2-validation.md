# Task 28 — Validate and close phase two

**Status:** Ready  
**Depends on:** Tasks 20-27  
**Primary area:** integration + physical Scribe validation  
**Validation:** Automated tests + physical-device human checkpoint  
**Physical Scribe required to complete:** **Yes**  
**Human interaction required to complete:** **Yes**  
**Completion gate:** Automated gate passes, then **HUMAN CHECKPOINT 28A** is performed and recorded


## Device/human-testing rule
Run the complete automated gate first. This task **cannot be marked Implemented** until a human performs checkpoint 28A on the physical Scribe and records the observed UX, touch/stylus behavior, e-ink readability, refresh workaround if needed, and native recovery.

## Outcome
Prove rich book solutions work end to end without regressing phase-one solving/progress.

## Automated gate
Run fmt, strict clippy, workspace tests, snapshots, platform/packaging tests, and documented ARMv7/glibc build. Add end-to-end coverage importing PGN, parsing JSON, selecting main/alternative/sideline nodes and inline move references, verifying exact displayed FEN, proving plain SAN-looking prose is inert, and proving progress bytes remain unchanged.

## HUMAN CHECKPOINT 28A — REQUIRED HUMAN / PHYSICAL DEVICE

**Required to close Phase 2. This cannot be replaced by snapshots, synthetic input, or an agent simulation.**

After the automated gate passes, a human must use a representative real book collection with
nested variations and explanations on the first-generation Scribe and verify:

- tappable moves are visually obvious as chips/buttons on the physical e-ink screen;
- unmarked SAN-looking prose is visibly plain and inert;
- finger taps work on main-line, side-line, alternative, and description/comment move references;
- stylus taps work on the same classes of references;
- every tested chip jumps to the exact expected board position;
- pagination and long explanations are readable at normal viewing distance;
- closing analysis restores the live solve board;
- browsing alone does not mark progress;
- solving, wrong rollback, promotion, Free Board, navigation, collections, restart persistence, and X exit remain correct;
- native Kindle input/display behavior recovers after exit;
- if residual panel ghosting appears, the top-left Refresh control clears it as the accepted workaround.

Record the tested commit/build and the human pass/fail observations before closing the phase.

## Closure
Record phase-two results, mark each task Implemented only when its own criteria pass, archive Tasks 20-28, and update README from planned to implemented.

## Suggested commit
`docs: close rich solution browsing phase`
