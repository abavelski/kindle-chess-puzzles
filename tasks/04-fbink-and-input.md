# Task 04 — Bring up FBInk display and Kindle touch input

**Status:** Ready  
**Depends on:** Tasks 00 and 03

## Outcome

Run the shared renderer on the first-generation Kindle Scribe and translate real finger touches into logical actions.

This is the first task that produces a live device app loop.

## FBInk pin and build

Using the known-good result from Task 00:

- pin an exact FBInk revision;
- document its license and build flags;
- create the minimal `fbink-sys` boundary;
- cross-compile it for the confirmed Kindle ABI;
- link it into the Rust binary reproducibly.

Do not track floating `master`.

Keep all `unsafe` calls inside the sys/wrapper boundary.

## Display adapter

Implement a safe Kindle display wrapper that can:

- initialize/reinitialize FBInk;
- obtain the device/display state needed by the app;
- present a full Gray8 frame;
- request a correctness-first full refresh;
- propagate errors with context;
- restore/close only state this app changed.

Do not add custom framebuffer ioctls if FBInk already provides the required operation.

## Input discovery

Implement evdev discovery from Task 00 findings.

Do not hardcode `/dev/input/eventN`.

Select the finger touchscreen by device name/capabilities and keep the selection diagnostics loggable.

Decode enough multi-touch events to produce:

- touch down;
- position updates;
- touch up.

Normalize raw coordinates to display coordinates using measured ranges/rotation.

Stylus events should be ignored or separately classified, not doubled into finger taps.

## Gesture policy for this task

Support reliable tap only.

A touch becomes a tap if it ends within configured movement/time thresholds. Keep thresholds host-testable.

Previous/next works through visible on-screen controls at first. Swipe may be added later if it is proven reliable and does not conflict with board interaction.

## Event loop

Create the minimal binary loop:

1. initialize state with a bundled fixture collection;
2. render;
3. present full frame;
4. wait for touch;
5. map to hit target/logical action;
6. dispatch;
7. re-render/present;
8. exit through a visible Exit action or controlled signal path.

Do not add collection filesystem persistence yet.

## Automated tests

Use saved/raw input fixtures from Task 00 to test:

- device selection predicate;
- raw range normalization;
- rotation transform;
- tap recognition;
- drag/non-tap rejection if applicable;
- stylus filtering;
- screen corner/center mapping;
- hit-test action mapping.

Add FFI wrapper validation tests for bad sizes/rectangles before unsafe calls.

## HUMAN CHECKPOINT B

On the physical Scribe:

1. launch from the current safe shell/scriptlet path;
2. verify the rendered board geometry;
3. tap each board corner and center and confirm the intended square is selected;
4. tap every visible control;
5. perform at least ten selections/moves without coordinate drift;
6. verify stylus activity does not create duplicate unintended actions;
7. exit and confirm the stock UI is usable;
8. reboot/recover normally if the app is killed.

Record findings in `docs/device/ks1-barolo.md`.

## Acceptance criteria

- same shared renderer runs on host and Scribe;
- no fixed event-node assumption;
- no raw input codes leak into core;
- touch mapping matches rendered geometry;
- full-frame FBInk presentation is reliable;
- app can enter and leave the experiment without persistent framebuffer/input damage;
- Kindle cross-build is part of the check workflow.

## Suggested commits

`kindle: add pinned FBInk Rust boundary`

`kindle: add Scribe touch input and event loop`
