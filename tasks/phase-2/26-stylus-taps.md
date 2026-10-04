# Task 26 — Support Scribe stylus taps

**Status:** Ready  
**Depends on:** Task 25 and verified Task-00 device notes  
**Primary area:** Kindle input adapter

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

## HUMAN CHECKPOINT 26A
On the first-generation Scribe, tap main and side-line moves with finger and stylus, confirm identical node selection, then confirm native pen behavior after X exit.

## Non-goals
No handwriting, annotations, eraser, or pressure-sensitive UI.

## Suggested commit
`kindle: map Scribe pen contacts to UI taps`
