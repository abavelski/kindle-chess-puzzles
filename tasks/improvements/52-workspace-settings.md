# Task 52 — Separate workspace settings

**Status:** In progress — physical validation pending
**Working branch:** main
**Depends on:** Tasks 40 and 49

## Scope

Keep independent Free Board button, Notes button and board-size preferences for
Puzzle and Game Review. Settings edits and their side effects apply only to the
active workspace. Preserve both profiles across restart in settings.json; old
global settings initialize both profiles to the existing preferences. Keep
malformed/future settings protected and puzzle progress/collections untouched.

## Acceptance

- Failing tests demonstrate independent settings and inactive-session preservation.
- Test old-settings migration and save/reload of distinct profiles, including edits
  saved from Game Review and edits saved after switching back to Puzzle.
- Renderer settings controls, geometry and hit targets follow the active profile;
  existing UI appearance stays unchanged for identical preferences.
- Run full host gates and the pinned Kindle build. Record deployment if performed;
  physical switching/restart acceptance remains pending until verified.

## Validation — 2026-10-09

The initial host tests failed because review edits changed puzzle preferences
and disabled its paused Free Board session. Independent profiles and workspace
side effects now pass. Tests cover legacy migration, distinct profiles across
restart, preserving inactive sessions, the actual runtime save path from either
workspace, malformed/future file protection, active toolbar/board geometry, and
restoring settings-panel pixels. Inspected the new review settings Gray8 frame;
existing snapshot expectations are unchanged.

Full host gates and `scripts/build-kindle.sh` passed, including the pinned
ARMv7/glibc-2.35 release. `scripts/stage-kindle.sh` prepared the install package.
Binary SHA-256: `d9b0487730ee66f2f88778077bbb7f7c31a75032708d04f6623d4664a16b6027`.
Source SHA-256: `6d183e785bfe40782778fe33ef5b4b7e23668da254bda8b8014f7da992301836`.
Read-only SSH attempts to 192.168.1.20:2222 timed out; no device files were
changed. Deployment and physical switching/relaunch validation remain pending.

## Deployment — 2026-10-09

After the user restored device availability, deployed the staged release with
`scripts/deploy-kindle.sh --host root@192.168.1.20 --port 2222`. App process
and supervisor lock were absent before install. Installed binary hash matches
the release above; loader dependencies resolve. All 118 collection/state file
hashes are unchanged. Physical workspace-specific settings and restart
validation remain pending.
