# Task 05 — Complete single-collection interaction parity

**Status:** Awaiting HUMAN CHECKPOINT C  
**Depends on:** Tasks 02-04

## Outcome

With one bundled/explicit collection, make the Scribe app behave like the current Kobo app for all interaction features that do not require multi-file persistence.

## Automated implementation complete

Task 05 now adds:

- a single six-puzzle `parity-puzzles.json` collection containing the one-move, three-ply, white queen promotion, white knight underpromotion, black rook promotion, and automatic-opponent-promotion cases required by Checkpoint C;
- that parity collection as the bundled live Kindle collection, without adding filesystem collection/progress loading;
- explicit `can_previous_puzzle` / `can_next_puzzle` state so navigation remains no-wrap while unavailable directions render visibly disabled;
- end-to-end host integration tests that dispatch real `Action` values, assert resulting `AppState`, and lock 11 deterministic Scribe-resolution rendered flow snapshots;
- updated Task 03 renderer goldens reflecting disabled Previous/Next appearance at collection ends.

GitHub Actions run 23 verified formatting, clippy with warnings denied, all workspace/core/render/platform tests, Sashité regeneration, Python/tooling checks, and the glibc-2.35-compatible ARMv7 FBInk-linked cross-build.

The task is intentionally not marked Implemented until HUMAN CHECKPOINT C passes on the physical Scribe.

## Required behavior

### Solution mode

- exact UCI matching;
- automatic stored opponent reply;
- correct intermediate feedback;
- wrong move rollback and obvious wrong result;
- complete result;
- ignore extra board taps after completion;
- Reset restarts from FEN;
- optional description auto-reveals on completion.

### Promotion

- stage on last rank;
- modal for Queen/Rook/Bishop/Knight;
- Cancel;
- q/r/b/n suffix before checking;
- wrong underpromotion rollback;
- direct automatic opponent promotion.

### Free Board

- toggle from toolbar;
- accepts arbitrary physical-board moves;
- does not grade or mutate solved state;
- entering keeps the current visible board;
- Reset restores FEN and stays in Free Board;
- navigation stays in current mode;
- returning to Solution resets FEN/attempt while preserving orientation.

### Navigation

Because the Scribe has no Kobo page-turn buttons:

- provide explicit previous/next touch controls;
- keep no-wrap behavior;
- disable or visibly inert the unavailable direction at ends;
- navigation loads the target puzzle and clears transient attempt feedback.

Optional swipe may supplement controls but cannot be the only navigation method.

### Orientation

- new puzzle faces side-to-move when unlocked;
- Flip affects presentation only;
- orientation lock preserves manual orientation across puzzle navigation;
- Flip still works while locked;
- unlock does not immediately jump the board.

### Metadata and feedback

Header shows:

- current/total;
- puzzle ID;
- optional difficulty in parentheses;
- solved marker once progress integration exists.

Description toggle:

- works before solving;
- works after solving;
- works in Free Board;
- never advances or solves.

Use monochrome selected/control states.

## TDD

Most behavioral tests should already exist from Task 02. Add integration tests that drive:

`Action -> AppState -> ViewState -> Rendered snapshot`

for complete flows.

Examples:

- wrong move -> same board + wrong snapshot;
- correct three-ply flow -> opponent reply visible;
- promotion cancel/choose -> modal snapshots/state;
- Free -> arbitrary move -> Solution -> original FEN;
- Flip/lock/navigation sequence;
- reveal description before solve without progress mutation.

## HUMAN CHECKPOINT C

On the Scribe, run the reference fixtures:

1. one-move puzzle: wrong then correct;
2. three-ply puzzle with automatic reply;
3. white promotion;
4. underpromotion;
5. automatic opponent promotion;
6. Free Board arbitrary moves + reset;
7. previous/next at both collection ends;
8. black-to-move auto-orientation;
9. orientation lock + flip + navigation;
10. description toggle before and after solve.

Confirm the Sashité pieces are legible at normal reading distance and all controls are comfortably tappable.

## Acceptance criteria

- single-collection feature behavior matches documented Kobo parity;
- host state/render tests cover every visible state;
- physical checklist passes;
- no collection/progress filesystem work is sneaked into this task;
- all checks/cross-build pass.

## Suggested commit

`app: complete single-collection puzzle interaction parity`
