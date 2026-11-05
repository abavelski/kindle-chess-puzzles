# Archived Chessnut Move / KindleBT probe

Standalone, independently built reference CLI used on a first-generation Kindle
Scribe, firmware 5.19.6, ARMv7 hard-float/glibc 2.35. The probe is now **uninstalled**
from Kindle. Raw device/build logs, manual reports and generated/downloaded
artifacts have been removed; private identifiers are preserved outside Git.

The authoritative outcome and next-phase design input are in
[the integration handoff](../../docs/CHESSNUT_MOVE_INTEGRATION_HANDOFF.md).
[Device results](notes/DEVICE_RESULTS.md) contain the concise acceptance matrix.
The [source pins](notes/SOURCE_PINS.json) preserve exact upstream provenance.
The reusable subset is kept separately in `support/chessnut-move/` for main.

## Findings

H1–H3 passed: native loading/identity, connection/discovery, full position/status
telemetry, negotiated MTU 300 and human-confirmed green LED/all-off clearing.
Human-confirmed e2→e4 and e4→e2 movements exercised the 35-byte Normal command.
The corrected `done` flow re-enabled realtime after motion and verified three
target reports. An explicitly requested longer e2↔e7 probe was also exercised.

STOP and Ctrl+C sent the SDK STOP successfully at the local stack but did not
reliably interrupt movement in physical tests. The user accepts that limitation;
it remains a failed H4 criterion. Do not claim full interruptible-motion acceptance,
reliable sleep/wake/reconnect, captures or arbitrary puzzle rearrangement.

## Source retained on this branch

This isolated workspace retains the complete Rust controller, C worker, unit/mock
and native contract tests, pinned build/stage/deploy/rollback code, and operator
scripts. It is not part of the parent app's workspace or packaging. No global
Bluetooth, stock UI, framebuffer or main-app change is part of this experiment.

Host checks (no board):

```sh
sh experiments/chessnut-move-ble-probe/scripts/check.sh
```

Reproducible ARM build/stage, only if explicitly needed for later investigation:

```sh
sh experiments/chessnut-move-ble-probe/scripts/build.sh
sh experiments/chessnut-move-ble-probe/scripts/stage.sh
```

Rust 1.85.1, Zig 0.13.0, the pinned SDK/header sources and target libraries are
recorded in `scripts/delivery.py` and `notes/SOURCE_PINS.json`. Downloaded SDKs are
build references only; never stage proprietary Amazon libraries. The builder checks
ARM ELF, interpreter, GLIBC <=2.35, dependency names and COPY relocations.

The old install location was `/mnt/us/chessnut-ble-probe/`, ownership marker
`chessnut-ble-probe-v1`. Removal used its ownership-checked uninstaller, verified
no active process/lock and checked 121 original app/data files before/after.
Do not reinstall or execute board/motor tests just to inspect this archive.

## Operator CLI reference

`preflight` opens no session. `packet-fen '<six-field FEN>'` is offline encoding.
`session --address <private address>` offers explicit telemetry, battery, pieces,
watch, LED, motion-enable, target preview, confirm/cancel and stop commands.
Read-only one-shot modes never arm motion. The guided `move-board.sh` default
requires white king e1, black king e8, white pawn e2, all other squares empty;
its last target is e7 and `--return` expects e7 and targets e2. It refuses piped
approval and waits for confirmation, with private native logs and a quiet prompt.
`done` requests STOP, restores reports and verifies the final placement.

All these device entry points were removed during cleanup. Their source remains
for reproducibility, not as the production puzzle integration UI. The future app
must have its own scoped integration tasks and acceptance checks.

## Privacy and branch policy

`local-kindle/chessnut-move/devices.private.json` is ignored and mode 0600. It holds
the observed BLE address/type/source, macOS board identifier and Kindle SSH endpoint.
No real address belongs in this archive. Example test addresses are synthetic.
`notes/my_output*.txt`, private files, logs and build output are ignored.

Commit/push this experiment branch for provenance; **do not merge it to main**.
Only the curated handoff and standalone reusable reference package are transferred
selectively. The puzzle app, progress, collections, renderer and runtime remain
unchanged until the next integration plan is approved.
