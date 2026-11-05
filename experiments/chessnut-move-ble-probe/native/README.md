# Native probe transport

Archived native source; the probe is uninstalled. Exact header/source pins are in
`../notes/SOURCE_PINS.json`. See the root integration handoff for all verified ABI,
MTU, descriptor-layout, callback, buffer and cleanup facts.

Compile against pinned KindleBT, ACE and log.c headers. The generated KindleBT
translation unit adds only the missing descriptor-write callback bridge. Vendor
libraries remain on the device; SDK libraries are build references only.

The worker verifies 32-bit ARM/glibc/daemon identity before opening a session,
drops only itself to the verified Bluetooth identity, and handles versioned bounded
IPC. M3 telemetry/LED operations remain explicit. M4 Normal writes need one-shot
arming and adequate MTU. Separate persistent storage preserves callback ownership
for ordinary, all-off and emergency STOP writes.

Local STOP writes completed but did not reliably stop physical motion in human
tests. The user accepts this limitation; process termination is not a verified
emergency stop. Do not infer final motion/STOP from a local callback.

The curated copy in `support/chessnut-move/` is intended for integration planning;
this branch retains the complete independent build recipe and host contract tests.
