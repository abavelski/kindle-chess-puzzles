# Task 56 — Workspace Lock and Flip button settings

Status: Implemented

## Scope

Add LOCK BUTTON and FLIP BUTTON ON/OFF rows alongside existing button visibility
settings. Save both preferences independently for Puzzles and Game Review,
using additive version-1 fields defaulting to ON for existing settings files.
Hidden buttons have no rendered/hit-testable targets and their direct actions
are inert; hiding controls preserves the current board orientation and lock.
Keep progress and uploaded collections untouched. Preserve existing board sizes,
sidebar bottom placement and workspace sessions. No commit or deployment.

## Dependencies

Implemented workspace settings, generic buttons and small-board sidebar work.

## Acceptance

- Failing tests first specify defaults/migration, independent toggles,
  persistence/reload, modal actions and orientation/progress preservation.
- New settings rows have matching render/hit targets in both workspaces.
- Both board sizes compact around hidden controls and preserve minimum targets.
- Inspect new settings and hidden-control snapshots before updating checksums.
- Run full host gates and the pinned Kindle cross-build.

## Validation — 2026-10-09

Core and renderer tests failed first on the missing preferences/actions/rows.
Defaults, old-file migration, invalid new-field types, active-workspace toggles,
modal behavior, preserved orientation/lock and unchanged puzzle progress pass.
The runtime test exercises SettingsChanged through the real SettingsStore and
reloads distinct Puzzle/Review values after edits from both workspaces.

Renderer tests cover each visibility combination in STANDARD/SMALL and
portrait/landscape, row hit targets, minimum touch sizes, compacted toolbar
controls and bottom-anchored SMALL navigation. Inspected all 32 new settings and
toolbar snapshot frames plus existing changed settings frames before accepting
checksums. Other existing snapshots remain unchanged. The modal blank-area
check now samples below the five settings rows rather than its center, which
is occupied by a new row.

`scripts/build-kindle.sh` passed: formatting, Clippy, complete workspace tests,
Python/asset/platform/packaging gates and the pinned ARMv7/glibc-2.35 release.
`git diff --check` passed. No commit, deployment or physical-device actions.

Inspectable frames: `target/lock-flip-snapshots/*.pgm`;
PNG example: `target/lock-flip-settings.png`;
all-state contact sheet: `target/lock-flip-contact.png`.

### Committed deployment — 2026-10-09

The user explicitly authorized commit, push, build and deployment. Implementation
was committed to `main` as `1a6fca3` and pushed. The committed build passed all
quality gates and was installed on the recorded Scribe. Installed binary
matches the build; the loader resolves dependencies and all 119 collection/state
hashes are unchanged. Physical acceptance remains pending; deployment evidence
is recorded in `docs/device/ks1-barolo.md`.
