# Task 40 — Board size setting

**Status:** Implemented
**Working branch:** main
**Depends on:** Task 39 toolbar alignment policies

## Scope

Add a persistent Settings choice between STANDARD and SMALL board sizes. SMALL
uses 60% of the standard board width/height, rounded down to whole squares, at
the same top position and horizontally centered. Keep toolbar/navigation widths
and spacing, move both rows up with the board, and expand the full-width bottom
text panel with larger description, topic, and analysis text. Existing settings
files default to STANDARD. Preserve puzzle data, grading, preview, and progress.
Initially leave changes uncommitted for review and Scribe testing; commit and
push after the user accepts the deployed change.

## Acceptance

- Failing tests first for additive settings persistence and smaller layout.
- Modal setting hit target toggles size and emits SettingsChanged without
  modifying board, grading, analysis preview, or progress.
- Layout and square hit testing work in both orientations and board sizes;
  controls retain touch-safe dimensions and dialogs fit in both layouts.
- Review deterministic snapshots for standard/small Settings, descriptions,
  topics, analysis, and small-board promotion/collection/GOTO dialogs.
- Run fmt, Clippy, workspace tests, and the Kindle build gate.

No hardware lifecycle/input-transform changes. Physical readability review can
follow deployment; host tests cover shared behavior.

## Implementation record — 2026-10-07

The initial persistence test failed because the new preference was discarded;
the initial layout test failed with a 1672-pixel board instead of 1000 pixels.
Settings version 1 now accepts an additive `small_board` boolean, defaults old
files to STANDARD, and persists through the existing atomic settings store.
Tapping BOARD SIZE switches between STANDARD and SMALL without changing the
live board, grading cursor, selected analysis preview, or durable progress.

On the 1860×2480 / 300-DPI Scribe layout, SMALL reduces the board from 1672 to
1000 pixels per side (60% rounded down to whole squares), retains its top and
horizontal center, and moves the control rows and text panel up by 672 pixels.
Control and dialog dimensions remain touch-safe. Description and analysis font
scale increases from 3 to 4; topic scale increases from 4 to 5. The expanded text
panel uses modest padding rather than consuming its extra height in margins.

Reviewed Gray8 previews cover the updated standard Settings panel plus the
small board, description, Settings, GOTO, collection picker, promotion, analysis,
topic, and flipped board. Existing standard-layout snapshots remain unchanged
except for the added Settings row. Tests cover additive serialization, atomic
save/reload, both board orientations, portrait/landscape hit targets, modal
bounds, state preservation, and complete pixel reconstruction through partial
damage when switching either way. The user accepted the deployed board-size
change on the physical Scribe on 2026-10-07 and requested commit/push.

Validation passed: `cargo fmt --check`, Clippy with warnings denied, all workspace
tests, and `scripts/build-kindle.sh` (including Python/tooling/platform/package
checks and the ARMv7/glibc 2.35 release build). The build used the pinned host
Python dependency in a temporary virtual environment.

Deployed for physical testing on 2026-10-07; installed binary and preserved data
hashes and the user's physical acceptance are recorded in
`docs/device/ks1-barolo.md`.
