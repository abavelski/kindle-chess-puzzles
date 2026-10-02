# FBInk integration

Task 04 pins FBInk at:

`92e127008145b2a22fba7c59815d810d716310dd`

This is the full revision corresponding to the `92e1270` build identifier measured from the known-good KOReader FBInk binary during Task 00.

## Source and license

Upstream: `https://github.com/NiLuJe/FBInk`

FBInk is included as the `vendor/FBInk` Git submodule at the exact revision above. The upstream project is licensed under **GPL-3.0-or-later** at this revision; see `vendor/FBInk/LICENSE` and its source headers.

The repository does not track FBInk `master` and does not install FBInk into Kindle system directories.

## Kindle build

The confirmed Kindle Scribe ABI is `armv7-unknown-linux-gnueabihf`. `crates/fbink-sys/build.rs` builds the pinned source only for that Rust target with the ARM hard-float GNU toolchain.

The FBInk make flags are:

```text
KINDLE=1
MINIMAL=1
IMAGE=1
staticlib
```

`IMAGE=1` is required because Task 04 presents the project-owned Gray8 frame through `fbink_print_raw_data`. In a minimal FBInk build this also enables the drawing support needed by the image path.

The C bridge is deliberately small. It exposes only the operations Task 04 needs:

- open/init and target verification;
- reinitialization;
- display-state query;
- full-frame 8-bit grayscale presentation;
- completion wait;
- close/version reporting.

All Rust `unsafe` FFI is confined to `fbink-sys`. `kindle-platform`, `chess-render`, `chess-core`, and the application binary remain `#![forbid(unsafe_code)]`.

## Presentation policy

Task 04 is correctness-first:

- the shared renderer produces a tightly packed 1860×2480 Gray8 frame;
- FBInk state is re-read before presentation;
- the adapter requires the measured Scribe Y8/8-bpp state and known Task 00 orientation;
- the frame is passed to `fbink_print_raw_data` as single-channel luminance with alpha ignored;
- a flashing/full refresh is requested;
- the code waits for refresh completion before returning.

Partial damage refreshes and lifecycle refinements belong to Task 07.

No custom framebuffer ioctl is introduced by Task 04.
