# Task 54 — Generic button properties

Status: Implemented

## Scope

Refactor renderer buttons to carry independent optional icon and text properties
and an explicit display type (icon, text, or icon with text). Share toolbar
presentation definitions between layout and drawing. Add deterministic icons for
FREE, LOCK, RESET and FLIP, retaining their current text-only display type.
Keep existing pixels, geometry, hit targets, selected/disabled states, workspace
behavior and settings unchanged. No Kindle deployment or device checkpoint.

## Acceptance

- Tests cover display-type selection with both properties present and either
  property missing, plus selected/disabled rendering.
- Existing toolbar controls share their presentation definitions in layout and
  rendering; size follows display type rather than property presence.
- Existing snapshots pass without changing checksums.
- Run scripts/check.sh and scripts/build-kindle.sh.

## Validation — 2026-10-09

The first button-property test failed on the missing shared button types before
implementation. Focused property, layout and rendering tests now pass, covering
both content properties, missing properties, display-type sizing, combined
content, selected/disabled states and the four distinct monochrome icons.

`scripts/build-kindle.sh` passed using a temporary host virtual environment with
`requirements-tools.txt`. It ran the complete `scripts/check.sh` gate (formatting,
Clippy, all workspace and snapshot tests, Python suites, asset generation checks
and script contracts), then validated the ARMv7/glibc-2.35 release. All existing
snapshot checksums are unchanged. `git diff --check` passed. No deployment,
physical-device checks or commits were performed.
