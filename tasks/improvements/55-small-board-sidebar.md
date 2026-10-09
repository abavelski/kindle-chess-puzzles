# Task 55 — Small-board sidebar controls

Status: Implemented

## Scope

Only SMALL board mode: align the board at the left content edge, move the
existing toolbar and navigation controls into the space to its right, use
available icons in the toolbar, retain text navigation with narrower widths,
and begin the full-width analysis/status panel directly below the board.
Wrap sidebar toolbar controls when necessary to preserve minimum touch sizes.
Keep STANDARD rendering, actions, settings, uploaded data and modal sizes unchanged.
No commit or deployment; provide inspectable screenshots from snapshot fixtures.

## Dependencies

Implemented board-size settings and generic button presentation (Tasks 40, 54).

## Acceptance

- Failing tests first specify left alignment, sidebar control bounds, minimum
  touch sizes, non-overlap, correct hit targets and reclaimed analysis space.
- Both workspaces and optional toolbar visibility work in portrait/landscape.
- Available toolbar icons appear only in SMALL; NOTE and navigation retain text.
- Inspect changed SMALL snapshot frames before accepting new checksums.
- STANDARD snapshot checksums stay unchanged.
- Run the workspace quality gates and Kindle cross-build; do not deploy.

## Validation — 2026-10-09

The left-alignment/sidebar acceptance test failed first on the centered board.
Focused tests cover both workspaces, portrait/landscape, optional controls,
minimum physical touch sizes, non-overlap, icon policy and logical hit targets.
Inspected the nine changed puzzle frames, selected FREE/LOCK and hidden-control
frames, review fixtures/import frames and workspace-settings frame before
accepting the SMALL checksums. All STANDARD checksums remain unchanged.

On the portrait Scribe, six puzzle toolbar controls need two balanced lines to
retain the existing 10 mm minimum touch size; navigation remains a separate
text row. Review and reduced puzzle toolbars fit on one line above navigation.
Analysis starts 12 pixels below the board's coordinate frame at 300 dpi.

`scripts/build-kindle.sh` passed with the host dependencies in
`/tmp/kindle-buttons-venv`. This includes formatting, Clippy, all workspace tests,
Python suites, asset/script/packaging contracts and ARMv7/glibc-2.35 release
validation. `git diff --check` passed. No commit, deployment or device actions.

Snapshot-derived PNG examples are under `target/`: `small-sidebar-analysis.png`,
`small-sidebar-notes.png`, `small-sidebar-review.png`, `small-sidebar-hidden.png`,
`small-sidebar-locked.png` and `small-sidebar-examples.png` (puzzle/review pair).
The original test frames are in `target/small-sidebar-snapshots/` and
`target/review-import-small.pgm`. Export puzzle snapshots with
`UI_SNAPSHOT_DIR` set to an absolute output directory when running the
`smaller_board_visual_states_match_reviewed_gray8_snapshots` test.

### Follow-up — left toolbar alignment

SMALL now defaults to the existing renderer `ToolbarAlignment::Left` policy;
STANDARD retains `FullWidth`. Alignment remains a developer option, not a saved
Settings control. A new test failed on stretched/spread controls before the
change, then passed for both workspaces. Inspected refreshed SMALL snapshots
before updating checksums; navigation geometry is unchanged.

### User-authorized test deployment — 2026-10-09

The user subsequently requested build and deployment. Rebuilt with the complete
quality gates, staged and installed on the recorded Scribe. Installed binary
matches the release receipt, the loader resolves dependencies and all 119
collection/state hashes are unchanged. Physical testing remains pending; see
`docs/device/ks1-barolo.md`. No commit was made.

### Follow-up — bottom sidebar placement

Both SMALL button bars now anchor as a group at the bottom of the space beside
the board. The navigation touch rectangles end exactly at the board coordinate
frame bottom; the toolbar sits above navigation, preserving compact left
alignment, wrapping and gaps. Analysis and STANDARD geometry remain unchanged.
The portrait/landscape and optional-control test failed on the previous top
placement, then passed in both workspaces. Inspected all refreshed SMALL frames
before accepting new checksums.

The complete `scripts/build-kindle.sh` gate passed after bottom placement,
including formatting, Clippy, workspace/snapshot tests, Python/platform/package
contracts and ARMv7/glibc-2.35 release validation. `git diff --check` passed.
Refreshed screenshot examples are under the existing `target/small-sidebar-*`
paths. This follow-up has not been committed or redeployed.

### Committed deployment — 2026-10-09

The user explicitly authorized commit, push, build and deployment. Implementation
was committed to `main` as `1a6fca3` and pushed. The committed build passed all
quality gates and was installed on the recorded Scribe. Installed binary
matches the build; the loader resolves dependencies and all 119 collection/state
hashes are unchanged. Physical acceptance remains pending; deployment evidence
is recorded in `docs/device/ks1-barolo.md`.
