# Task 39 — Toolbar alignment policies

**Status:** Implemented
**Working branch:** main
**Depends on:** Task 38 dynamic toolbar layout

## Goal

Support left, right, and full-width placement of the toolbar under the board. Use full width for the app now, with a developer-facing renderer choice and no new user setting.

## Scope

- Preserve the current control order and Settings visibility behavior.
- Keep the analysis icon at its minimum square touch width. Keep text controls at least their current touch-safe width.
- Left and right alignment pack visible controls at the chosen edge using compact widths.
- Full width stretches visible text controls evenly to consume spare space. If only icons are visible, keep their widths and distribute the spare space evenly between them. A single visible control starts at the left edge at its compact width.
- Use one layout function for rectangles consumed by drawing and hit testing. Keep alignment policy and geometry in `chess-render`; do not add a field to durable `Settings` or core `AppState`.
- Continue to report `TooSmall` when minimum widths and gaps do not fit.

## Tests and acceptance

- Start with failing layout tests for left, right, full width, icon-only, and single-control cases.
- Check visible order, minimum touch width, edge placement, gap spacing, and hit targets when FREE/NOTE are hidden.
- Review changed Gray8 snapshots, including a hidden-control configuration.
- Run formatting, Clippy, workspace tests, and the Kindle build gate.

No device lifecycle or input transform changes are in scope. Physical review can follow the next deployment.

## Implementation record — 2026-10-07

The first full-width renderer test failed with the old right-aligned start coordinate. A shared renderer placement function now handles all three policies. `APP_TOOLBAR_ALIGNMENT` selects full width without changing the settings schema. Focused tests cover both edge policies, full-width text stretching, icon-only spacing, and a single icon or text control at the left. Reviewed Scribe-sized Gray8 snapshots include default and both-hidden states.

`cargo fmt --check`, Clippy with warnings denied, the workspace test suite, and `scripts/build-kindle.sh` passed. The validated build was staged and deployed on 2026-10-07. The user confirmed the deployed toolbar works and requested commit and push to `main`.
