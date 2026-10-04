# Task 37 — Direct puzzle goto

Status: Implemented

## Goal

Make large puzzle collections easier to browse by adding a compact `GOTO` control between the existing `PREV` and `NEXT` navigation controls.

## Scope

- Keep the existing previous/next behavior and labels.
- Add a small touch-sized `GOTO` button between them.
- Open a modal visually consistent with the existing collection picker.
- Show a puzzle-number input and an on-screen numeric keypad.
- Provide delete, confirm, and cancel controls.
- Accept only puzzle numbers in the active collection's 1-based range.
- Confirming jumps directly to that puzzle and updates normal durable current-puzzle progress.
- While the modal is open, ordinary board/toolbar/navigation actions are blocked.
- Do not change collection files or puzzle IDs.

## Tests

Added core state-machine coverage for numeric entry, invalid-range prevention, modal blocking, cancel, direct jump, and progress updates. Added renderer layout/hit-test coverage for the compact middle control and modal keypad, plus a deterministic render snapshot state for the dialog.

## Acceptance

- `GOTO` is visibly smaller than `PREV` and `NEXT` while retaining the minimum physical touch target.
- The keypad can enter any valid 1-based puzzle number for the current collection.
- Invalid/empty input cannot navigate.
- Confirm closes the dialog and selects the requested puzzle.
- Cancel closes the dialog without changing puzzles.
- Host checks and Kindle release CI pass before merge.
