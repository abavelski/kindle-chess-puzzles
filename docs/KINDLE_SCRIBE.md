# Kindle Scribe platform notes

Target for the first milestone: **Kindle Scribe, first generation (board name Barolo)**.

This document separates facts supported by current upstream/community documentation from facts that must be measured on the actual device.

## Upstream facts used by the plan

FBInk states that Kindle support covers the Kindle lineup and its release history includes identification of the Kindle Scribe. FBInk has Kindle-specific MediaTek handling and can be built for Kindle with its Kindle target.

Sources:

- https://github.com/NiLuJe/FBInk
- https://github.com/NiLuJe/FBInk/blob/master/README.md
- https://github.com/NiLuJe/FBInk/blob/master/fbink.h

Current KindleModding documentation uses scriptlets/KPM for homebrew and describes KUAL as obsolete on modern setups. KPM packages can provide a `launch.sh` entry point.

Sources:

- https://kindlemodding.org/jailbreaking/whats-next/installing-homebrew/
- https://github.com/KindleModding/KPM
- https://github.com/KindleModding/example_kpm_package

The first-generation Scribe is commonly identified as Barolo and current community reports describe its userspace as `kindlehf` / `armv7l`. Treat that as a build hypothesis until Task 00 records the actual target device.

Reference:

- https://github.com/NiLuJe/KindleTool
- https://github.com/KindleModding/koxtoolchain

## What must be probed on the actual Scribe

Do not hardcode these before Task 00:

- firmware version;
- kernel/userland architecture and ABI;
- libc version;
- framebuffer device, size, stride, bit depth, rotation;
- FBInk-reported device ID/capabilities;
- touch event node(s);
- absolute touch ranges;
- coordinate transform to displayed orientation;
- finger vs stylus event separation;
- whether the stock UI repaints over direct FBInk output;
- whether exclusive input grab is needed;
- safe launch/return behavior for the installed jailbreak/homebrew stack;
- writable locations that are convenient for puzzle files and safe for progress.

Task 00 records these in `docs/device/ks1-barolo.md`.

## Bring-up policy

The first hardware tests should be launched from a shell or simple scriptlet. Do not start by packaging the full app.

Recommended progression:

1. run a device probe;
2. run an FBInk hello/test-pattern binary;
3. prove one clean full-screen frame and restore;
4. identify touch and draw a temporary touch marker;
5. run the app loop;
6. only then solve stock-UI handoff and packaging.

This keeps framebuffer/input uncertainty separate from package-manager uncertainty.

## Cross compilation

Expected Rust target for a hard-float ARMv7 userspace is:

```text
armv7-unknown-linux-gnueabihf
```

Do not consider this confirmed until the device probe and a hello-world execution pass.

Prefer a reproducible Kindle toolchain/sysroot (for example the current KindleModding/KOReader toolchain family) instead of linking against arbitrary host libraries.

FBInk should be pinned to an exact revision after a known-good Scribe test. The build should produce a static or otherwise self-contained FBInk boundary where practical while respecting FBInk's GPLv3+ licensing.

## FBInk boundary

Use FBInk as a library, not by spawning the CLI for every frame.

The Rust wrapper should expose only what this app needs, approximately:

- initialize/reinitialize and obtain display state;
- blit project-owned Gray8 data;
- refresh a rectangle;
- wait for an update when required;
- clean up.

Do not expose the full C API throughout the application.

FBInk documents raw image/pixel support and rectangular refresh APIs; use those rather than device-specific framebuffer ioctls directly unless a verified Scribe problem requires an exception.

## Input

The Scribe is a touch/stylus device. Initial app scope is finger touch.

Input discovery must use evdev capabilities/name rather than a fixed `eventN` path. Record all plausible devices during Task 00.

The first implementation should support:

- tap board square;
- tap toolbar/control;
- tap modal item;
- previous/next puzzle through visible controls.

Swipe navigation is optional until raw-to-screen coordinate transforms and gesture thresholds are stable. Stylus support is explicitly deferred; it should not accidentally trigger duplicate finger actions.

## Stock UI and lifecycle

Modern Kindle firmware may use a stock UI stack that can repaint the framebuffer or own input. Do not assume a historical `stop framework` recipe is safe for the Scribe.

The lifecycle task must document the exact process/service transitions used on the target firmware and include cleanup on:

- normal exit;
- non-zero exit;
- SIGINT;
- SIGTERM.

Never change framebuffer rotation underneath a running stock UI unless the lifecycle design has explicitly taken ownership and restores it.

## Data path policy

Keep paths configurable.

The intended UX is that puzzle JSON files are easy to copy from a computer, while progress is app-owned and never mixed into puzzle source data. Task 00/08 should choose a Kindle path that is writable, survives reboot, and works with the user's actual MTP/USB workflow.

Do not bake a Kobo path or Cobalt store path into shared code.

## Licensing note

FBInk is GPLv3+. Sashité Western chess SVG assets used by the reference project are public-domain/CC0 style assets according to their upstream project documentation. Preserve provenance and review final distribution obligations once the exact linking/package strategy is implemented.
