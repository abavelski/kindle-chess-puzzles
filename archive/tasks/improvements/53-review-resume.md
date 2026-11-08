# Task 53 — Persist Game Review position

**Status:** Implemented
**Working branch:** main
**Depends on:** Tasks 49 and 52

## Scope

Persist the active review collection filename, stable game ID and authored move
position in a separate versioned review-resume.json. Restore it on launch while
starting in Puzzle; switching to Game Review displays the remembered position.
Preserve review orientation and analysis paging. Free Board experiments stay
transient. Keep puzzle state, progress.json and uploaded collections untouched.
The initial request excluded commits and deployment. Subsequent user requests
authorized Kindle deployment and committing/pushing to main.

## Acceptance

- Red-first tests cover restart at root/main-line/variation positions, switching
  workspaces, game/file selection, stable IDs after reordering, and independent
  puzzle progress.
- Missing/invalid game files or removed/changed moves fall back safely.
- Malformed/future resume files are preserved; failed writes retain latest state
  and retry, including exit. Writes use the existing atomic storage boundary.
- Run scripts/check.sh and scripts/build-kindle.sh. No renderer changes required;
  existing review snapshots verify the restored visible states.

## Validation — 2026-10-09

Red-first core, platform and runtime tests failed on the missing resume APIs and
runtime integration, then passed after implementation. Coverage includes every
fixture main-line/variation/root position, collection/game identity after
reordering, regenerated node IDs, unchanged puzzle progress and game-source
bytes, workspace switching, orientation, transient Free Board experiments,
missing games, removed/changed moves, protected malformed/future/unreadable files,
and latest-state retry after failed writes, including exit from Puzzle.

`scripts/build-kindle.sh` passed with the pinned requirements-tools.txt installed
in a temporary host virtual environment. It ran the complete `scripts/check.sh`
gate (formatting, Clippy, workspace/snapshot tests, Python tests and script
contracts), then validated the ARMv7/glibc-2.35 release. `git diff --check` passed.
No commits or deployment were performed; no physical-device checks were run.

## Deployment — 2026-10-09

The user subsequently authorized Kindle deployment. Staged the tested receipt
and deployed with `scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222`
using the documented empty-password SSH transport. App process and supervisor
lock were absent. The installer completed; the installed binary SHA-256 matches
`23027062186b46891e10844ac183197933cedb36ee46f2f62c863e5cb9181ba4`.
The device loader resolved dependencies; all 118 puzzle/game/state file hashes
are unchanged. The app was not launched automatically. Physical review resume
interaction/relaunch validation remains unperformed. No commits were made.
