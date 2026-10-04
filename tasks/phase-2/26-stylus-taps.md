# Task 26 — Support Scribe stylus taps

**Status:** Ready  
**Depends on:** Task 25 and verified Task-00 device notes  
**Primary area:** Kindle input adapter  
**Validation:** Automated tests + physical-device human checkpoint  
**Physical Scribe required to complete:** **Yes**  
**Human interaction required to complete:** **Yes**  
**Completion gate:** Automated gate passes, then **HUMAN CHECKPOINT 26A** is performed and recorded


## Device/human-testing rule
Implementation and synthetic/recorded-event tests should be completed off-device first. This task **cannot be marked Implemented** until a human performs checkpoint 26A on the physical Scribe and the result is recorded.

## Outcome
A Scribe pen contact/release activates the same logical UI targets as a finger tap, especially analysis moves, without drawing support.

## Required behavior
- Re-read `docs/device/ks1-barolo.md`; use verified pen capabilities, never guessed event nodes.
- Decode one pen contact/release as a logical tap using existing display-coordinate normalization.
- Ignore pressure, tilt, hover motion, and drawing data for product behavior.
- Aggregate movement so one physical pen tap emits one action.
- Preserve finger behavior and native input recovery on normal/crash exit.
- Keep Linux event codes out of core/render.

## Host tests
Recorded/synthetic sequences: tap, hover-only, contact movement noise, out-of-range, repeated taps.

## HUMAN CHECKPOINT 26A — REQUIRED HUMAN / PHYSICAL DEVICE

**Required to mark Task 26 Implemented. This cannot be replaced by automated tests or an agent simulation.**

A human with the first-generation Scribe must:

1. launch the built app on the device;
2. tap multiple main-line and side-line move chips with a finger;
3. tap the same targets with the stylus;
4. confirm finger and stylus select the same analysis nodes and board positions;
5. include at least one hover-without-contact check so hover does not activate a target;
6. exit with X and confirm native Kindle pen behavior is restored.

Record the tested commit/build and pass/fail observations in the task/device documentation before changing the status to Implemented.

## Non-goals
No handwriting, annotations, eraser, or pressure-sensitive UI.

## Suggested commit
`kindle: map Scribe pen contacts to UI taps`
