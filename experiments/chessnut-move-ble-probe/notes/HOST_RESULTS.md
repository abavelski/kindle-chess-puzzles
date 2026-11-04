# M0 automated evidence — 2026-11-04

Status: Implemented. No physical checkpoint required for M0. H1–H5 remain
NOT TESTED in DEVICE_RESULTS.md. Source and dependency pins are recorded in
SOURCE_PINS.json, rust-toolchain.toml, Cargo.toml and Cargo.lock.

Red/green observations during implementation:

- FEN tests: four positive conversion/packet tests failed with
  `FEN conversion not implemented`; implemented conversion, all five passed.
- IPC tests: subprocess success/cleanup/error tests failed with Protocol while
  the report reader was a stub; bounded versioned framing made all seven pass.
  Initial 250 ms process-start deadline was too short on macOS; the request
  test deadline is two seconds. The timeout mock waits ten seconds and is
  killed/reaped at the deadline; tests never depend on a real BLE link.
- CLI tests: packet-fen initially exited UNSUPPORTED; after implementing the
  offline path, both CLI tests passed. Unsupported live commands still fail.

Final isolated gate: `scripts/check.sh` PASS — Rust 1.85.1, fmt, clippy with
warnings denied, and 14 integration tests including the feature-gated mock
subprocess suite. Cargo metadata confirms this directory is its own workspace
and uses its own ignored target directory. The mock worker has no BLE/native
implementation and is not a deployment artifact.

Parent regression gate: formatting and clippy PASS. The first workspace-test
attempt failed because the documented Python `chess==1.11.2` host dependency
was missing. Installed requirements-tools.txt into this experiment's ignored
`target/host-check-venv/` and reran the full parent suite. See the completion
record below. No parent source, configuration, fixtures or dependencies changed.

Cross-compilation/ELF/runtime checks: NOT TESTED, belong to M1; no platform or
native code was changed. No SSH deployment or device/board communication was
attempted.

Completion record: full parent `cargo test --workspace` PASS after providing
the documented Python dependency (237 tests passed, zero failed).
