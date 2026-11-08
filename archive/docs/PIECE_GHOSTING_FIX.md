# Fix plan: faint piece ghosts after moves

The subsequent queen c7→d6 framebuffer capture, physical photo, rejected
experiments and Kobo/Cobalt comparison limits are summarized in
[the measured ghosting investigation](GHOSTING_INVESTIGATION.md).

## Dark-square follow-up — 2026-10-03

The regional-clean build at `00f31cb` almost resolved piece traces, but the user
reported faint dark-square residue and native UI traces at startup. The next
white-prepass experiment (`59ebe68f…` binary) made native ghosting worse,
including on light squares, while piece traces remained. The user observed a
brief white region followed by the ghost returning on redraw. That experiment
is rejected and its production white pass has been reverted.

A live framebuffer capture from the rejected build shows the correct chess
frame. Sampled empty light/dark interiors are uniformly 238/184; the white
status interior is uniformly 255. There are no native shapes in these sampled
stored pixels. This supports a panel/controller-history explanation rather
than the app retaining an image of the native UI. It does not identify the
exact controller cause or prove that every pixel is correct.

The REAGLD experiment (`69aa4b0a…` binary) changed clean startup/regional
requests to flashing REAGLD (GLD16 on the pinned MTK path), a ghost-compensation
mode documented by the pinned FBInk API. The driver accepted the request and
reported completion, but the user observed a completely white physical screen
on repeated launches. A framebuffer captured during that white-screen report
matches the earlier correct startup frame byte-for-byte (SHA-256
`fc119182f301fa8377456b43c3add2884478886878964bc296a16ffed85c3fee`).
This is a presentation regression, not a blank rendered frame, app crash or
native framebuffer overwrite. Unsupported-mode fallback did not run because
submission succeeded. The kernel log identifies the cause: `[HWTCON ERR]waveform mode[5] not
loaded night_mode[0] @wf_lut_get_waveform_mode_slot,608`. REAGLD maps to mode 5,
whose day-mode waveform is not loaded on this device. Its asynchronous driver
error is not reflected in the successful FBInk submission/completion results.

REAGLD is rejected and reverted. Current production returns to the original
`00f31cb` branch behavior: flashing AUTO (GC16 on MTK) for startup and regional
clean requests, partial AUTO for ordinary selection/UI updates, and no white
intermediate pass. Tests cover exact final-pixel payloads and baseline waveform
mapping. The residual dark-square ghosting from that original build remains
unresolved; framebuffer captures and successful ioctls cannot establish physical
panel cleanliness. A future waveform experiment must include user visual
confirmation before being treated as a working display mode.

## Problem

After a piece moves, a very light image of that piece can remain visible on its
previous square.

This should be treated as an e-ink presentation/refresh-quality issue first,
not as a chess-state or renderer-clearing bug.

## Why this is the likely cause

The current implementation already has the right software ownership boundaries:

- `chess-render::calculate_damage` compares the previous and current complete
  Gray8 frames, so pixels erased from the origin square are damage.
- damage history advances only after a successful presentation;
- host damage tests reconstruct the new frame from the submitted dirty regions;
- the app renders a fresh project-owned frame on every interaction.

The physical device record also contains the same symptom from Task 04:
temporary traces remained on the previous square after piece moves even with a
single app instance. A later full flash cleared them.

The Phase 1 refresh policy explains why this can still happen:

- the initial/recovery frame is flashing;
- ordinary changed rectangles use non-flashing `WFM_AUTO`;
- `RefreshPolicy::default()` has `full_every = None`, so there is no periodic
  cleaning refresh;
- explicit grayscale/fast modes are disabled until measured on the Scribe.

In `crates/fbink-sys/c/fbink_bridge.c`, the current mapping is:

- `RefreshMode::Full` -> `is_flashing = true`, `WFM_AUTO`;
- `RefreshMode::AutoPartial` -> `is_flashing = false`, `WFM_AUTO`.

At the pinned FBInk revision
`92e127008145b2a22fba7c59815d810d716310dd`, the Kindle MTK refresh path
turns flashing AUTO into GC16 with `UPDATE_MODE_FULL`, while a non-flashing
request uses `UPDATE_MODE_PARTIAL`.

Source:
https://github.com/NiLuJe/FBInk/blob/92e127008145b2a22fba7c59815d810d716310dd/fbink.c

That is consistent with a correct framebuffer containing the empty square while
the e-ink panel still retains a faint physical remnant from the previous
partial update.

The pieces and board are not purely 1-bit content: antialiasing and gray square
backgrounds make piece erasure a grayscale transition. Repeated partial updates
are therefore the wrong place to assume that an old high-contrast silhouette
will always be fully cleaned.

## Recommended fix

Do **not** change chess state, piece assets, renderer clearing, damage detection,
or the general regional-update architecture.

Add a targeted **clean regional refresh** for board squares whose piece
occupancy changed.

The important distinction is:

1. **refresh coverage** — which rectangle is updated; and
2. **refresh strength** — PARTIAL vs a flashing/FULL GC16 update.

The current platform code couples "full refresh strength" with "whole screen
coverage". Decouple those concepts so a dirty board square can receive a
clean/full waveform without forcing a whole-screen flash.

### 1. Prove the software frame is already clean

Before changing waveform behavior, add/keep a focused host regression test for
a representative move:

1. render the position before the move;
2. dispatch the move;
3. render the position after the move;
4. assert that the origin square in the new Gray8 frame is byte-for-byte the
   correct empty-square rendering;
5. replay only `calculate_damage` / `compact_damage` regions over the old
   frame and assert that the result equals the complete new frame.

This prevents a waveform workaround from hiding a future renderer/damage bug.

### 2. Derive changed chess squares outside core

Keep refresh policy out of `chess-core`.

In the outer runtime, capture the board before dispatch and compare it with the
board after dispatch/effect processing. `AppState::board()` and
`Board::piece_at()` are already sufficient.

For each logical square 0..63, mark it changed when the piece occupancy differs
before vs after.

This naturally covers:

- normal moves: origin + destination;
- captures: origin + destination;
- promotions: origin + destination;
- automatic opponent replies: all affected origin/destination squares;
- Free Board moves.

A first selection does not change occupancy, so it must remain a fast partial
update and must not flash.

For puzzle navigation/reset, many occupancies may change. Treat that as a board
clean operation; if orientation also changes, prefer one clean update for the
whole board rather than trying to map old/new square positions individually.

### 3. Map changed logical squares to complete display-square rectangles

Use the current renderer layout rather than hard-coded device coordinates.

`Layout::square_rect(display_square)` already exposes a full square rectangle.
Convert logical square to display square with the current orientation:

- white-facing: `display = logical`;
- black-facing: `display = 63 - logical`.

For an ordinary move, orientation should be unchanged. If the before/after
orientation differs, fall back to a clean whole-board rectangle.

Refresh the **whole affected square**, not only the pixel-diff silhouette. This
gives the panel a uniform cleaning pass over the entire cell that contained the
old piece.

### 4. Add a regional clean refresh mode in the Kindle platform layer

Do not make the renderer or core know about waveforms.

Extend the platform presentation plan so each submitted rectangle can carry a
refresh-strength hint, for example:

- `Partial` — existing non-flashing AUTO behavior;
- `Clean` — flashing AUTO for that rectangle;
- `WholeScreenClean` — existing recovery/first-frame behavior.

Names are flexible; the architectural requirement is that "clean/full waveform"
must no longer imply "replace all damage with the viewport".

At the C bridge, the clean regional mode should use the same proven mapping as
the existing full refresh:

- `cfg.is_flashing = true`;
- `cfg.wfm_mode = WFM_AUTO`.

On the pinned Scribe/FBInk path this becomes GC16 +
`UPDATE_MODE_FULL`, but only for the supplied rectangle.

Keep the existing first-frame and `--full-refresh` whole-screen behavior
unchanged.

### 5. Build one presentation batch

For a move:

- replace/cover ordinary dirty regions inside each occupancy-changed square with
  the complete square rectangle and mark those rectangles `Clean`;
- leave status/header/toolbar/selection-only damage as ordinary `Partial`;
- avoid duplicate/overlapping submissions;
- submit the full batch and keep the existing single final completion wait;
- advance `previous` only after the whole batch succeeds, exactly as today.

Do not introduce a second renderer or direct framebuffer writes.

### 6. Do not enable arbitrary periodic full-screen flashing yet

`RefreshPolicy::full_every` already exists, but Phase 1 deliberately left it
unset because no cleaning cadence was measured.

The targeted square clean should be tested first. If ghosting still accumulates
elsewhere after real use, then measure and add a periodic whole-screen clean
cadence. Do not choose a number only by intuition.

Likewise, explicit non-flashing `WFM_GC16` (`GrayPartial`) can be tested as a
lower-flash alternative, but it should only replace the regional clean approach
if device testing shows the old-piece silhouette is reliably gone.

## Tests to add

### Host renderer/damage regression

- moving a piece produces a clean empty origin square in the new frame;
- replaying damage reconstructs that frame exactly.

### Board-change / clean-region planning

- selection only -> no clean square;
- deselection only -> no clean square;
- ordinary move -> origin + destination clean;
- capture -> origin + destination clean;
- promotion choice -> origin + destination clean;
- automatic opponent reply -> all changed piece squares clean;
- wrong solution move that rolls back to the original board -> no clean square
  unless a visible committed board difference remains;
- reset/navigation with many changed occupancies -> clean board;
- orientation change with board change -> clean board;
- feedback/header-only change -> partial only.

### Platform refresh mapping

- partial request remains non-flashing AUTO;
- clean regional request is flashing AUTO without expanding to the viewport;
- explicit whole-screen recovery still expands to the viewport;
- failure in any clean/partial submission does not advance damage history;
- a mixed clean + partial batch performs one final completion wait.

### C bridge contract

Update the existing mock/contract coverage to verify that the new regional
clean mode sets the same FBInk fields as the existing flashing request and that
unsupported-mode fallback behavior remains unchanged.

## Device verification

Use an optimized release build and keep the existing safe launcher/recovery
path.

1. Reproduce the ghost with the current default build.
2. Run the existing `--full-refresh` mode and confirm the previous-square
   ghost disappears. This is the strongest quick confirmation that the problem
   is refresh physics rather than stale software pixels.
3. Install the targeted regional-clean build.
4. Make at least 30 piece moves covering light and dark origin squares,
   captures, long moves, promotion, and automatic replies.
5. After every move, inspect the origin square immediately. There should be no
   recognizable previous-piece silhouette.
6. Verify selections remain responsive and do not flash.
7. Verify promotion/collection modals, navigation, flip, reset, description,
   progress persistence, X exit, and native recovery still behave as in Phase 1.
8. Record touch-to-submit/completion timing and whether the localized clean flash
   is visually acceptable.
9. If localized clean updates are artifact-free, keep them. If they still leave
   a trace, only then test a whole-board clean after committed moves or a
   measured periodic whole-screen cadence.

## Acceptance criteria

The fix is complete when:

- the software-frame/damage regression proves the old square is actually erased;
- an ordinary selection remains a partial non-flashing update;
- committed piece moves clean only the affected board cells (or the board when
  orientation/navigation makes that safer);
- no visible previous-piece silhouette remains after the device move test;
- no whole-screen flash is introduced for every tap;
- no core/renderer Kindle dependency is added;
- existing damage batching, one-final-wait behavior, failure handling, input
  ownership, lifecycle recovery, persistence, and parity tests remain green.

## Files expected to change when implementing

Primarily:

- `app/kindle-chess/src/main.rs` — derive piece-occupancy changes and build the
  presentation intent;
- `crates/kindle-platform/src/display.rs` — accept per-region refresh strength
  without conflating it with whole-screen coverage;
- `crates/kindle-platform/src/refresh.rs` — represent/test the new policy;
- `crates/fbink-sys/src/lib.rs` and
  `crates/fbink-sys/c/fbink_bridge.c` — expose/map the regional clean mode;
- renderer/platform tests — regression and presentation-plan coverage.

Avoid changing `chess-core` behavior, piece artwork, FEN/solution logic,
storage, input discovery/grab, or lifecycle scripts for this fix.
