# Task 49 — Validate game review on the physical Scribe

**Status:** Ready  
**Working branch:** `game-review`  
**Depends on:** Task 48  
**Primary area:** device acceptance + closure docs  
**Validation:** Host gates plus human checkpoint  
**Physical Scribe required:** **Yes**  
**Human interaction required:** **Yes**

## Outcome

Prove the review workspace is readable and usable on the first-generation Scribe without regressing puzzle behavior or lifecycle.

## Automated prerequisite

Before deployment, pass formatting, strict Clippy, workspace tests, Python/tooling tests, renderer snapshots/damage reconstruction, packaging/lifecycle contracts, and the documented ARMv7/glibc release build.

## HUMAN CHECKPOINT 49A

On the physical Scribe:

1. Launch with at least two review games, including one annotated game with a variation.
2. Confirm Refresh and the new review icon are easy to distinguish and tap.
3. Enter Review and open GAMES; select another game and confirm metadata/board update.
4. Step PREV/NEXT through several main-line moves, including first/last disabled behavior.
5. Tap a main-line move and a variation move in the lower panel; verify the board matches the intended positions.
6. From a variation, press PREV/NEXT and confirm predictable return to main-line navigation.
7. Enable FREE, make several scratch moves, RESET, navigate to another authored position, and disable FREE.
8. Check STANDARD and SMALL board readability; specifically confirm SMALL analysis text is comfortably readable and tappable.
9. Test FLIP/LOCK and both finger/stylus taps on representative movetext/picker targets.
10. Switch back to Puzzles and confirm the exact previously active puzzle/attempt/analysis state is restored.
11. Confirm solved/current-puzzle progress did not change merely from review use.
12. Exercise manual Refresh, sleep/wake, and normal Close/relaunch enough to catch obvious repaint/input regressions.

Record observations, build hash, installed binary hash, and preserved puzzle-data/progress hashes in the device document.

## Completion

Only after 49A passes may Task 49 and Phase 3 be marked Implemented/closed. Integration of `game-review` into `main` remains a separate explicit user decision.

## Suggested commit

`docs: validate game review milestone`
