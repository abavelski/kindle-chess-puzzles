# Chessnut Move / KindleBT probe

An independent **host-only Milestone 0** experiment. No Kindle deployment,
Bluetooth connection, native worker, privilege change, or motor command is
available. The main chess application and parent Cargo workspace are unchanged.

## Progress

The scope is [the probe plan](../../docs/CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md).
Its final instruction limits implementation to one milestone per request.
Completion is tracked here to keep every change inside the experiment.

- [x] M0 — **Implemented** (2026-11-04): inspect/pin upstream sources and ABI;
  isolate the workspace; validate FEN conversion and protocol byte fixtures;
  test versioned bounded IPC with a subprocess mock; pin compiler/dependencies.
- [ ] M1 — Ready: ARMv7/glibc <=2.35 build, native preflight worker, stage,
  verify, recoverable deploy/uninstall. H1 must pass before M2.
- [ ] M2 — GATT connection/discovery; H2 discovery checkpoint.
- [ ] M3 — Telemetry/LED/MTU evidence; H3 read-only checkpoint.
- [ ] M4 — Guarded motor control; H4 motors checkpoint.
- [ ] M5 — Reliability/rollback; H5 independence checkpoint.

M0 automated gate: **PASS**. All physical checkpoints H1–H5: **NOT TESTED**.
No end-to-end or Kindle compatibility claim is made.

## Host prerequisites and commands

Use rustup with **Rust 1.85.1**, rustfmt, and clippy (local rust-toolchain.toml).
The dependency graph is checksum-pinned in Cargo.lock. chessnut-move **=1.0.2**
has default features disabled: no Tokio or btleplug. It uses edition 2024 and
compiles on this compiler. Development/validation host: macOS ARM64.

From the repository root:

```sh
experiments/chessnut-move-ble-probe/scripts/check.sh
cargo test --locked --all-features --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml
cargo run --locked --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml --bin chessnut-ble-probe -- packet-fen '6kq/8/8/8/8/8/8/8 b - - 17 42'
```

The full test command requires `--all-features` to build the test-only
`mock-worker` subprocess. Plain `cargo test --manifest-path ...` runs the FEN
and CLI tests; it skips the IPC suite. The check script always runs all tests
and selects the experiment's compiler even when invoked from another directory.
Tests need a POSIX host (`kill -TERM` verifies an actual child-process signal).

`packet-fen` prints the original six-field FEN, all 35 bytes, Normal mode,
WithoutResponse, and **OFFLINE ONLY**. It never starts a worker or writes BLE.
It validates eight ranks/eight files, empty runs, all 12 piece identities,
active color, castling syntax, en passant syntax, and counters. It does not
check legal chess, physical inventory, known current position, or safe motor
feasibility. Empty boards and sparse protocol fixtures are intentionally allowed
for offline encoding. Full metadata stays on the host; only placement enters
the SDK Position/Command. Live movement needs the later M4 checks.

`preflight`, `session`, `motion-enable`, `set-fen`, and other live commands exit
with **UNSUPPORTED**. There is nothing to install or uninstall at M0; remove
this experiment's ignored `target/` to clean local build outputs.

## IPC scaffold

See [IPC.md](IPC.md). The synchronous Rust client starts an explicitly supplied
worker with stdin/stdout pipes and separate inherited diagnostic stderr. Only
tests select a worker at present. The mock binary is feature-gated, contains no
BLE code, and must never be staged to the Kindle. One request is outstanding;
any error closes/reaps the worker. No command is automatically retried.

Tests cover success, required UUIDs, query write status, short/malformed
notification, wrong source, timeout, wrong ID/version, oversized frame,
disconnect, worker crash, real worker SIGTERM, and explicit interruption sending
DISCONNECT then SHUTDOWN. Drop kills/reaps a surviving child and joins its reader.
These are host failure contracts, not proof of native cleanup or motor STOP.
Controller SIGINT/SIGTERM handlers, native teardown/STOP, physical disconnect,
and sleep/wake remain later milestones. No live session exists yet.

## Source findings and next boundary

[Native/source notes](native/README.md) record release pins, licensing, SHA-256,
ABI mismatch, SDK/toolchain expectations, and MTU limitations. The old KindleBT
release library must not be paired with current public headers. M1 should build
the pinned current MIT source with matching headers (or deliberately use the
old API throughout), inspect all native ELF/GLIBC/NEEDED dependencies, and add a
harmless preflight before any device connection. Proprietary vendor libraries
are build references/device dependencies; never bundle them.

MTU remains unknown. A 35-byte conventional ATT write without response needs
MTU >=38. STOP also encodes to 35 bytes, so its delivery has the same unresolved
constraint. Nothing here proves that KindleBT can deliver either command.

## Next human checkpoint (after M1, verbatim from the plan)

**Human checkpoint H1 — Kindle-only, no board:** Human supplies SSH access and runs preflight. Record uname -m, firmware, GLIBC, loader/NEEDED check, BLE stack process presence, BT user/group mapping, whether libkindlebt.so can load, and app exit status. Check that the original Kindle Chess Puzzles still launches unchanged. **STOP** if binary fails to load, a privilege assumption fails, or system changes appear necessary.

Do not run this checkpoint yet: M1's binaries and preflight are not implemented.
Device records are [notes/DEVICE_RESULTS.md](notes/DEVICE_RESULTS.md).
