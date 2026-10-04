# Task 03 — Add deterministic layout, rendering, hit testing, and Sashité assets

**Status:** Implemented  
**Depends on:** Tasks 01-02

## Outcome

Render every parity UI state to a deterministic project-owned grayscale frame on the host, with the same layout/hit-testing code intended for Kindle.

Do not call FBInk yet.

## Implemented

Task 03 adds a platform-neutral `chess-render` implementation with:

- a project-owned `Gray8` frame with explicit width, height, stride, clipping, rectangle operations, grayscale/alpha blits, deterministic line/polygon drawing, and PGM export for host inspection;
- DPI-derived `Layout`, `Rect`, and shared `HitTarget` geometry used by both rendering and input mapping, including all 64 board squares, toolbar controls, previous/next navigation, and promotion choices/cancel;
- a Scribe 1860x2480 @ 300 DPI reference layout with a 1696x1696 board (212 px squares) and controls/modal targets at or above the 10 mm minimum touch size;
- deterministic 5x7 renderer-owned text, board coordinates, status/description text, string/number difficulty, solved marker, selected controls/squares, correct/wrong/complete feedback, and the promotion modal;
- the 12 Sashité Western source SVGs used by the reference app, preserved under `assets/sashite-western/`, plus a reproducible host generator and checked-in vector-layer output consumed without runtime SVG parsing;
- full-frame damage reporting through shared region types, leaving partial-refresh optimization to Task 07;
- 13 Scribe-resolution Gray8 snapshot hashes covering white/black orientation, selection, correct/wrong/complete/solved, Free Board, orientation lock, description, promotion, long description, and numeric difficulty.

No FBInk, evdev, framebuffer, filesystem, or Kindle runtime dependency was added to `chess-render`.

## Validation

GitHub Actions run 11 verified:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo test --workspace`, including the 10 new renderer/layout/canvas/asset/snapshot tests and all existing core tests;
- `python3 scripts/generate_sashite.py --check`, proving the checked-in piece layer reproduces from the source SVGs;
- the existing Task 00 Python/tooling checks;
- the minimal `kindle-chess` ARMv7 hard-float cross-build smoke test.

## Canvas

Create a `Gray8` frame/canvas type with explicit:

- width/height;
- stride;
- pixel buffer;
- clipping;
- rectangle fill/stroke;
- image/alpha blit;
- deterministic text/glyph drawing strategy.

Do not depend on the host window system.

## Layout

Build layout from display metrics, not Kindle constants.

The board should remain square and prominent on the Scribe while leaving room for:

- header: index/id/difficulty/solved marker;
- board coordinates/frame;
- toolbar;
- previous/next navigation controls;
- description/status region;
- promotion modal;
- collection picker/warnings later.

Use physical size/DPI where useful for minimum touch targets.

Layout must support both orientations without changing logical square identity.

## Hit testing

The renderer/layout layer owns coordinate-to-control mapping.

Given a normalized display coordinate, return a logical action target:

- board square;
- toolbar action;
- previous/next;
- modal choice;
- picker row/cancel where applicable.

Tests must prove all 64 displayed squares map back to the correct logical square in white and flipped orientations.

## Sashité Western pieces

Copy the same open/public-domain Sashité Western SVG sources used by the reference app and preserve upstream attribution/provenance.

Keep SVG as source of truth.

Add a reproducible host-side generation step that produces grayscale/alpha piece assets appropriate for e-ink. Prefer generated canonical-size assets over runtime SVG parsing.

Requirements:

- both colors;
- pawn/knight/bishop/rook/queen/king;
- deterministic output;
- no hand-edited generated blobs;
- approximately the visual scale of the reference board;
- clear separation on light/dark squares.

## UI parity visuals

Implement rendering for:

- board and coordinates;
- selection;
- correct intermediate hint;
- wrong centered result mark/icon;
- complete centered result mark/icon;
- solved marker;
- Free Board selected-state control;
- reset;
- flip;
- orientation-lock selected state;
- description-toggle selected state;
- previous/next;
- promotion modal;
- optional description;
- optional string/number difficulty.

Use monochrome/grayscale differences plus shape/border, never color alone.

## Snapshot tests

Add stable snapshots/goldens for at least:

- white orientation;
- black orientation;
- selected square;
- wrong;
- complete;
- solved;
- Free Board;
- orientation lock;
- description visible;
- promotion chooser;
- long multiline description;
- difficulty string/number;
- representative Scribe metrics from Task 00.

Also test layout constraints numerically.

## Damage preparation

Define `Rect`/region types now, but do not optimize refreshes yet. The renderer may initially report one full-frame damage rectangle.

## Acceptance criteria

- [x] host tests render every visible parity state;
- [x] snapshots are deterministic across CI;
- [x] hit testing shares layout data rather than duplicating magic coordinates;
- [x] all 64 squares map correctly in both orientations;
- [x] toolbar/modal targets meet the documented minimum size;
- [x] Sashité assets are reproducibly generated and attributable;
- [x] renderer has no FBInk/Kindle dependency;
- [x] workspace checks pass.

## Suggested commits

`test: add e-ink layout and snapshot specifications`

`render: add deterministic grayscale chess UI`
