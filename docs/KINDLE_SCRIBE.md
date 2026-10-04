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

Task 00 completed on 2026-10-02 and records verified measurements in `docs/device/ks1-barolo.md`: firmware 5.19.6, 32-bit ARMv7 hard-float/glibc 2.35, visible 1860×2480 Y8 framebuffer with 1872-byte stride, direct portrait finger coordinates, separate physical/virtual pen streams, and a successful Kindle FBInk `92e1270` overlay restored with Home. Task 04 subsequently ran the Rust/FBInk application loop on the device and verified finger mapping, controls, stylus filtering, and recovery. Stock-UI input/repaint contention and transient refresh traces remain explicit lifecycle/refresh work; Task 06 subsequently verified production storage persistence through exit/relaunch and computer-copy collection changes; other orientations remain untested.

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

This target and ABI are now confirmed by Task 00 measurements and Task 04 execution of the Rust/FBInk application on the Scribe.

The deployed Task 04 binary had to target glibc 2.35: an initial artifact linked against GLIBC_2.38 and failed in the device loader. CI therefore pins the ARM cross-build job to Ubuntu 22.04, while Task 04's verified device binary was also independently built with Zig targeting `arm-linux-gnueabihf.2.35`. Keep runtime libc compatibility explicit rather than treating a successful cross-build as proof of deployability.

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

The Scribe is a touch/stylus device. Phase one supports finger touch; Task 26
adds read-only tap decoding from the verified virtual pen stream.

Input discovery must use evdev capabilities/name rather than a fixed `eventN` path. Record all plausible devices during Task 00.

The first implementation should support:

- tap board square;
- tap toolbar/control;
- tap modal item;
- previous/next puzzle through visible controls.

Swipe navigation is optional until raw-to-screen coordinate transforms and gesture thresholds are stable.
Task 26 selects only `stylus-custom` (ABS_X/Y and key-event capabilities), never
both physical and virtual pen streams. Its measured inclusive ranges are
0–1860 and 0–2480, normalized to 1860×2480 display pixels. Pen contact/release
uses the same shared hit targets as finger taps; hover and drawing data are ignored.
Physical checkpoint 26A passed on 2026-10-04 by user confirmation: finger and
stylus selected matching analysis targets/positions, hover was inert, and X exit
restored native pen drawing. See `docs/device/ks1-barolo.md` for the tested build.

## Stock UI and lifecycle

Modern Kindle firmware may use a stock UI stack that can repaint the framebuffer or own input. Do not assume a historical `stop framework` recipe is safe for the Scribe.

Task 04 established that the foreground overlay can coexist with stock services but does not own the screen or input: stock UI actions can occur on the same touches and stock repaint can temporarily replace or bleed through the app frame. Task 07 must resolve that ownership/handoff explicitly rather than relying on the Task 04 overlay behavior.

The lifecycle task must document the exact process/service transitions used on the target firmware and include cleanup on:

- normal exit;
- non-zero exit;
- SIGINT;
- SIGTERM.

Never change framebuffer rotation underneath a running stock UI unless the lifecycle design has explicitly taken ownership and restores it.

## Data path policy

Keep paths configurable.

Task 06 uses these Kindle defaults:

- puzzle JSON directory: `/mnt/us/kindle-chess/puzzles`;
- app-owned progress file: `/mnt/us/kindle-chess/state/progress.json`.

They can be overridden with `KINDLE_CHESS_PUZZLE_DIR` and `KINDLE_CHESS_PROGRESS_FILE`, so host tests use temporary directories and a future platform adapter is not coupled to Kindle paths. The platform storage adapter creates the puzzle directory/default `puzzles.json` only when no matching collection exists. Matching invalid collections are preserved rather than overwritten. Progress uses a same-directory temporary file, file sync, atomic rename, and directory sync; malformed or future-version progress is protected from automatic replacement.

Earlier device checkpoints verified `/mnt/us` is usable for application staging. HUMAN CHECKPOINT D passed on 2026-10-02 and verified these exact Task 06 locations through Scribe exit/relaunch and computer-copy changes: active collection, per-collection current puzzle IDs, and solved markers restored, while uploaded collection hashes remained unchanged. A newly copied valid collection was discovered, and a malformed file could be selected and removed without losing progress.

Puzzle JSON remains user/source data. Solved/current/active progress is app-owned and is never written back into a puzzle collection.

Do not bake a Kobo path or Cobalt store path into shared code.

## Licensing note

FBInk is GPLv3+. Sashité Western chess SVG assets used by the reference project are public-domain/CC0 style assets according to their upstream project documentation. Preserve provenance and review final distribution obligations once the exact linking/package strategy is implemented.


## Verified waveform limitation — 2026-10-03

Flashing REAGLD (MTK GLD16, waveform mode 5) produced a white physical panel
on repeated launches despite correct framebuffer pixels and successful FBInk
submission/completion. The device kernel reported `waveform mode[5] not loaded
night_mode[0]` from `wf_lut_get_waveform_mode_slot`. The enum's presence in the
pinned header is not proof that the firmware has loaded that waveform.
Errno-based fallback does not detect this asynchronous failure. REAGLD was
removed; the original flashing AUTO/GC16 regional-clean build was restored.
See `docs/device/ks1-barolo.md` and `docs/PIECE_GHOSTING_FIX.md` for evidence.
