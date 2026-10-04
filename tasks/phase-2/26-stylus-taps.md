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

## Implementation preparation — 2026-10-04

Host implementation is prepared on `phase-2`; checkpoint 26A is pending.
The adapter discovers exactly one `stylus-custom` device by name and ABS_X/Y +
key-event capabilities. It never consumes the duplicate physical Wacom stream.
The Task-00 virtual pen ranges (X 0–1860, Y 0–2480, inclusive) normalize to the
existing display coordinates and shared hit targets. SYN_REPORT batches contact
movement; BTN_TOUCH release produces at most one tap. Hover, pressure, tilt,
eraser/tool departure, invalid positions, drags, and long contacts do not activate
UI targets. The existing finger grab and lifecycle cleanup remain in place;
pen input is read-only and has no exclusive grab.

Tests cover synthetic pen tap/hover/movement/out-of-range/repeated contacts,
the recorded five-contact Task-00 trace, matching finger/pen move-chip targets,
and the narrow two-file poll boundary including interruption and errors.

Do not change status to Implemented until the human confirms all six checkpoint
items and the tested build and observations are recorded below/in the device notes.

A prerequisite repair adds the missing visible ANALYSIS entry button in rich
puzzles' status area. The finding is recorded in `docs/PHASE_2.md`; existing
NOTE/toolbar behavior is preserved. A geometry/action test and a visually
reviewed closed-panel snapshot cover this entry point.

## Checkpoint readiness

Implementation commit `7c6e1e6` is installed on the first-generation Scribe,
firmware 5.19.6. The full automated gate and ARMv7/glibc-2.35 build passed;
installed binary hash and loader dependencies were verified. Exact build hashes,
commands, and data-preservation checks are recorded in `docs/device/ks1-barolo.md`.

For 26A, launch from the Kindle library, tap the header title, select the separate
**Stylus checkpoint 26A** collection, and open **ANALYSIS**. Compare several main
and side-line chips using finger and pen (page controls expose further moves),
check hover-only, then X exit and native pen drawing. The app is stopped and
ready for the human to launch. Status remains Ready until those observations pass.
