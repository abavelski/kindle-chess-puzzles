# Task 03 — Add deterministic layout, rendering, hit testing, and Sashité assets

**Status:** Ready  
**Depends on:** Tasks 01-02

## Outcome

Render every parity UI state to a deterministic project-owned grayscale frame on the host, with the same layout/hit-testing code intended for Kindle.

Do not call FBInk yet.

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

- host tests render every visible parity state;
- snapshots are deterministic across CI;
- hit testing shares layout data rather than duplicating magic coordinates;
- all 64 squares map correctly in both orientations;
- toolbar/modal targets meet the documented minimum size;
- Sashité assets are reproducibly generated and attributable;
- renderer has no FBInk/Kindle dependency;
- workspace checks pass.

## Suggested commits

`test: add e-ink layout and snapshot specifications`

`render: add deterministic grayscale chess UI`
