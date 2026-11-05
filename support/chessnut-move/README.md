# Chessnut Move integration reference

A curated, standalone reference extracted from the physically tested Kindle probe
on 2026-11-05. It is **not connected to the puzzle app**, has no device-app entry point,
and is intentionally outside the parent Cargo workspace. Adding it here does not
enable Bluetooth, alter packaging or authorize motor movement.

Read the [integration handoff](../../docs/CHESSNUT_MOVE_INTEGRATION_HANDOFF.md) first.
The complete probe source, operator scripts and historical build recipe remain on
`experiment/chessnut-move-kindlebt-probe`. The archived source commit is
[`376c097`](https://github.com/abavelski/kindle-chess-puzzles/commit/376c0974c267e2d7ec483842a89b297ce3c99582). Do not merge that branch to integrate
this reference.

## Contents

- `src/`: strict FEN-to-SDK conversion, decoded/raw telemetry, GATT discovery, preflight supervision and
  bounded subprocess IPC. These are reference building blocks for the future
  platform/service adapter, not imports for `chess-core` or `chess-render`.
- `native/`: actual worker and supporting headers, exact descriptor completion
  bridge, MTU request/observation and safe buffer/lifecycle handling. Requires the
  pinned public/vendor headers to build for Kindle. Its explicit native `arm` gate
  is not a substitute for application target validation or human authorization.
- `scripts/build_tools.py`: tested ELF, relocation, manifest, dependency and
  anchored KindleBT callback patch helpers. No downloader, SSH or installer.
- `tests/`: portable Rust/native/build contracts; optional real-header adapter test.
- `Cargo.lock`, `SOURCE_PINS.json`, `BUILD_PINS.json`: exact versions, source URLs,
  checksums and successful ARM artifact metadata. No proprietary payloads included.
- `licenses/`: notices for KindleBT, log.c and the protocol SDK. Preserve all
  dependency notices if a later package bundles them.

The standalone Rust library has no device binary; its feature-gated mock worker
is solely a host test fixture. `TargetFen::packet()` only encodes;
it cannot send a move by itself. `ipc::Client` can launch an explicitly supplied
worker and send typed operations. Future integration tasks must add lifecycle,
opt-in connection, physical/logical synchronization, authorization, cancellation
semantics and app-state tests. Do not copy the probe's manual timing assumptions
into an automatic puzzle loop.

## Host verification

Rust 1.85.1, Python 3.12+, a host C compiler and no board are required:

```sh
sh support/chessnut-move/scripts/check.sh
```

The optional adapter contract test additionally needs the exact pinned KindleBT,
ACE and log.c include directories, obtained privately from the pinned sources:

```sh
sh support/chessnut-move/scripts/check-native-adapter.sh \
  /path/to/kindlebt/include /path/to/ace/sdk/include /path/to/log.c/src
```

This verifies actual header layouts and callbacks on the host. It is not a live
BLE test or a cross-build. Use the archived probe build recipe plus the preserved
build helpers/pins when planning the app's future worker packaging. No libraries
are installed by these checks.

## Limits to carry into the plan

- Device: first-generation Scribe firmware 5.19.6, ARM hard-float, glibc 2.35.
- Request MTU 500, await matching successful callback; observed negotiated MTU 300.
- Serialize descriptor/write completion; one outstanding operation and finite queues.
- Incomplete sensor packets are diagnostic evidence, not puzzle state.
- Motor command is one full 35-byte Normal/WithoutResponse packet, with no replay.
- Position reporting pauses during motion. Human `done`, STOP, realtime re-enable
  and three matching target reports worked for final verification.
- **STOP/Ctrl+C did not reliably halt physical movement in human tests.** The user
  accepts this limitation. Ending the process/connection is not an emergency stop.
- Connection can fail from stale firmware session mapping despite a lit board LED.
- General captures, promotion, castling, setup/parking and automatic completion need
  scoped integration tests. SDK encoding is not physical feasibility validation.

Private device identities belong only in the ignored local file described in the
handoff, never this package. Raw logs and SDK/device library copies were deleted.

Extraction checks passed: 20 Rust tests (including fake-worker IPC and preflight),
11 Python/native-build test cases, and the actual pinned-header adapter contract.
The original controller CLI tests remain only in the archived full probe, because
this reference has no production controller binary.
