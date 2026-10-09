# Task 50 — Book-style comment prose

**Status:** In progress — physical validation pending
**Working branch:** main
**Depends on:** Tasks 30 and 49

## Scope

Remove renderer-added braces around analysis comments in puzzle solutions and
Game Review. Preserve authored text, variation parentheses, move ordering,
numbering, NAGs, explicit move references and their hit targets. Keep original
PGN/JSON files and puzzle grading/progress semantics unchanged.

Build and deploy for user validation. The user requested committing all changes
and merging the latest main on 2026-10-09; physical validation remains pending.

## Acceptance

- Specify brace-free comment prose with failing tests first, including nested
  variations and promotion comments. Preserve literal authored braces in prose.
- Verify comment text remains inert and explicit move references stay tappable.
- Intentionally inspect and update affected puzzle/review snapshots, including
  STANDARD/SMALL review layouts and pagination.
- Pass full host gates and the pinned ARMv7/glibc-2.35 release build.
- Deploy while the app is closed and verify installed binary and unchanged
  puzzle/game/progress/settings hashes.
- Await user confirmation of puzzle solution and game-review readability/taps.

## Host implementation

The initial notation test failed on renderer-inserted braces. The shared flow
now separates comment prose with whitespace rather than braces, leaving the
source model and authored punctuation intact. Tests specify nested variation
order, black-start/promotion comments, inert plain text, explicit references,
and literal authored braces. Inspected Gray8 frames cover nested puzzle prose,
promotion, and STANDARD/SMALL review text. Snapshot checksums were updated
intentionally; pixel comparison confirms the puzzle board/header/controls match
the previous frame. Existing end-to-end review traversal verifies move hit
targets, exact FENs, Free Board, puzzle session/progress restoration and regional
damage replay. User physical acceptance remains pending.

Full host checks and `scripts/build-kindle.sh` passed, including the pinned
ARMv7/glibc-2.35 release build. Staged and deployed with the existing scripts
for physical validation on 2026-10-07. Installed binary hash matches host output;
all ten puzzle/game/state file hashes are unchanged. No commit or push was made.
See the device record for build identity and validation checklist.
