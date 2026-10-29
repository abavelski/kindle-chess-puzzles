# Task 38 — Dynamic right-aligned toolbar

**Status:** Implemented (host validation)
**Working branch:** main

## Goal

Keep the six controls below the board in one compact, right-aligned row. When Settings hides FREE or NOTE, the remaining controls close the gap and stay at the right edge.

## Scope and architecture

- Preserve control order: analysis icon, FREE, NOTE, LOCK, RESET, FLIP.
- Give the icon a square minimum touch target and text buttons a fixed touch-safe width. Calculate positions from display metrics, visibility, and button type; do not store coordinates in core settings.
- Keep `Settings` responsible for FREE/NOTE visibility. `chess-render::Layout` owns placement and supplies the same rectangles to drawing and hit testing. Hidden controls have no rectangle and cannot receive taps.
- Keep right alignment as a renderer policy for this task. A future alignment preference would belong in versioned core settings, with layout interpreting it; this task adds no settings field or UI switch.
- Keep the toolbar within its existing band so damage tracking covers moved controls without a full-screen refresh.

No new UI abstraction crate is needed. The existing core action/state and renderer layout boundaries already separate behavior from presentation. The small toolbar specification in layout is the reusable abstraction for these two button types.

## Acceptance and verification

- Start with a failing host test showing hidden controls still consume space.
- For default, FREE-hidden, NOTE-hidden, and both-hidden states, visible controls remain ordered, do not overlap, have minimum touch targets, and end at the toolbar's right edge.
- Taps at visible control centers resolve to the corresponding action; hidden controls are absent.
- Review deterministic Gray8 snapshots for default and hidden configurations and update expected hashes deliberately.
- Run formatting, Clippy, workspace tests, and the existing Kindle cross-build gate. No platform behavior or puzzle data changes.

The physical Scribe appearance can be checked on the next deployment; this host-only layout task does not change the device input transform or framebuffer adapter.
