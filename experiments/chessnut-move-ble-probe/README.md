# Chessnut Move / KindleBT probe

An isolated CLI experiment for a first-generation Kindle Scribe, ARMv7
hard-float, firmware 5.19.6, glibc 2.35. The existing chess application and
parent workspace are unchanged. **M1 is deployed; H1 preflight failed. An offline native-link fix is ready
for a device retest.** No board connection or command is
implemented. Deployment and installed-payload checksums passed on 2026-11-04. SSH preflight
launched the Rust controller, then the native worker crashed before reporting
checks (exit status 1). Offline analysis found aliased libc globals in the
non-PIC worker; the new -fPIC -pie build removes them and rejects COPY
relocations. See [the diagnosis](notes/M1_STARTUP_CRASH.md). The installed build
has not been updated; H1 remains failed pending a device retest. The existing chess binary and 120 data files retain their
pre-deployment hashes.

## Progress

The scope is [the probe plan](../../docs/CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md).
All changes and completion records stay in this experiment.

- [x] M0 — Implemented: pinned independent Rust workspace, FEN/SDK encoding,
  expected packet fixtures, bounded/versioned IPC and mocked failures.
- [ ] M1 — Host-ready, deployed; **H1 failed** before marking Implemented.
  Host checks, native preflight, ARM build, ELF/dependency validation, stage,
  and initial scoped deployment are complete. Human preflight, main-app launch
  and device uninstall checks remain pending.
- [ ] M2 — GATT connection/discovery; H2 discovery checkpoint.
- [ ] M3 — Telemetry/LED/MTU evidence; H3 read-only checkpoint.
- [ ] M4 — Guarded motor control; H4 motors checkpoint.
- [ ] M5 — Reliability/rollback; H5 independence checkpoint.

H1 preflight: **FAIL**. Physical board checkpoints H2–H5: **NOT TESTED**. A cross-build cannot establish
KindleBT compatibility, library loading, privilege success, or board behavior.

## Host build and inspect

Requirements: rustup with **Rust 1.85.1**, rustfmt, clippy and target
`armv7-unknown-linux-gnueabihf`; Python **3.12+**, a host C compiler (Xcode command-line tools or cc), POSIX
shell, and SSH/scp for
later deployment. macOS ARM64 is the validated build host; checksummed Zig
archives also cover macOS x86-64 and Linux x86-64/ARM64. Initial downloads need
network access (SDK ~159 MiB, vendor headers ~88 MiB, Zig ~44 MiB). Allow a few
GiB of experiment-local disk space. No Docker or system toolchain changes.

From the repository root:

```sh
experiments/chessnut-move-ble-probe/scripts/check.sh
experiments/chessnut-move-ble-probe/scripts/build.sh
experiments/chessnut-move-ble-probe/scripts/stage.sh
experiments/chessnut-move-ble-probe/scripts/verify-stage.sh
```

Scripts also work from another directory. The build runs the complete host
gate before compiling, removes an old receipt on failure, checksum-verifies
all downloaded archives and extracts fresh sources. Rust dependencies are
locked; `chessnut-move =1.0.2` has default features disabled (no Tokio or
btleplug). Signal handling uses pinned `signal-hook =0.3.18`. Native compilation
uses Zig **0.13.0**, targeting `arm-linux-gnueabihf.2.35`, with the Sighery HF
SDK's vendor link reference and the matching pinned KindleBT/Darkroot headers.

The SDK's packaged compiler is a Linux executable. For macOS, the independent
builder reproduces KindleBT's documented Meson source list, includes, and link
dependencies using Zig. No SDK compiler is executed, no vendor library is
modified, and no existing project build scripts are imported or changed.
See [native details](native/README.md) and [source pins](notes/SOURCE_PINS.json).

Outputs (ignored, never committed):

```text
target/kindle/build.json                source, inputs, flags, binary hashes, ELF reports
target/kindle/required-libs.txt         direct device dependencies of shipped binaries
target/kindle/stage/                    verified upload directory
target/kindle/chessnut-ble-probe.tar     inspectable convenience archive
```

The stage contains a Rust executable, the C `kindlebt-worker`, `libkindlebt.so`,
launcher, uninstaller, build receipt, dependency list, checksums and notices.
Only those three native artifacts are included. Vendor/SDK libraries, SDK tools,
headers, mock-worker, Cargo build products and private logs are excluded.
Every staged native artifact is checked as ELF32 ARM EABI5 hard-float; executable
interpreters must be `/lib/ld-linux-armhf.so.3`; actual GLIBC version needs must
be <=2.35. The current artifacts' highest requirement is **GLIBC_2.34**.
Absolute NEEDED entries and RPATH/RUNPATH are rejected. No writable global
library directories are used. Notices include upstream and Rust dependency
license texts. This remains a local-install experiment; project code has no
declared public distribution license.

## Installed bundle and human H1 operation

**Initial deployment completed on 2026-11-04. Subsequent SSH preflight failed
with SIGSEGV in the native worker; do not proceed to M2.**
Keep the board disconnected; no board address is needed. The installed build is
from commit `dc8d7bc`. To rebuild/redeploy later, run build.sh and stage.sh first,
then substitute the private SSH hostname. Do not rerun the failed installed
worker before updating to the corrected build:

```sh
experiments/chessnut-move-ble-probe/scripts/deploy.sh --host root@DEVICE_HOST --port 2222
```

Deploy verifies the stage and current source receipt before opening SSH. It
uses the existing scp legacy transport (`-O`) and normal SSH host-key checking.
It creates only `/mnt/us/chessnut-ble-probe/`, uploads into a unique incoming
directory, verifies remote checksums and installs a new runtime via directory
renames. A failed update restores the previous runtime; preflight/install/remove
share a lock. Upload leftovers are removed best-effort after success/failure.
**Deployment does not launch anything.** It installs no scriptlet, service,
startup hook, Bluetooth configuration, or files under `/usr/lib` or the chess app.

After deployment, log in separately and run:

```sh
sh /mnt/us/chessnut-ble-probe/runtime/launch.sh preflight
echo "exit_status=$?"
```

The M1 launcher accepts **preflight only**. It checks checksums and the loader,
sets LD_LIBRARY_PATH to the runtime directory only, and holds an operation lock.
The Rust controller prints a timestamp and build/protocol versions, spawns its
sibling C worker and limits execution to 15 seconds. Ctrl+C, HUP, TERM or timeout
kill/reap the worker; the launcher releases its lock on exit. The worker:

1. Reads architecture/kernel, firmware, glibc and loader presence.
2. Resolves the named `bluetooth` account/group and verifies all real/effective/
   saved/filesystem IDs of btmanagerd against those nonzero target IDs, without
   altering the process. The passwd primary GID may differ (150 on this device;
   the verified daemon and named Bluetooth group use GID 1003).
3. Drops the worker's supplementary groups and root privileges to those verified
   IDs, then checks its resulting real/effective UID/GID. The controller and SSH
   stay root.
4. Loads each direct device dependency and the local KindleBT library with
   RTLD_NOW, checks required symbols, and closes handles. The loader resolves
   the current firmware's transitive dependencies; the old SDK's transitive
   library names are reference metadata, not mandatory device filenames. It does not call
   openSession, registration, radio, connection or write APIs.

A final `result=PASS session=NOT_OPENED connection=NOT_ATTEMPTED` and exit 0
mean preflight succeeded only. Firmware/library/privilege failures print a
specific reason and return nonzero. Do not change system services or permissions
to work around a failed H1; record the result first. Output contains no Bluetooth
address. Logs are not saved automatically; optional private captures belong only
under `/mnt/us/chessnut-ble-probe/logs/` and must never be committed.

Verify the original Kindle Chess Puzzles still launches and exits normally,
then fill [DEVICE_RESULTS.md](notes/DEVICE_RESULTS.md) with the actual results.
Do not proceed to M2 until the human confirms H1 acceptance.

## Uninstall and interrupted-update recovery

After preflight has exited:

```sh
experiments/chessnut-move-ble-probe/scripts/uninstall.sh --host root@DEVICE_HOST --port 2222
```

Or, on the Kindle:

```sh
sh /mnt/us/chessnut-ble-probe/runtime/uninstall-device.sh
```

Uninstall requires the experiment ownership marker and an idle operation lock.
It removes only `/mnt/us/chessnut-ble-probe/`, including its optional logs.
It never removes chess data or system libraries. Copy any wanted private logs
before uninstalling.

If power/SSH loss leaves `runtime.old`, `runtime.new`, or `.operation-lock`,
inspect experiment processes through SSH first. Do not remove a live lock.
With no experiment process running, restore `runtime.old` as `runtime` if needed,
remove an abandoned `runtime.new`/lock/incoming directory **inside the experiment
root only**, and redeploy. If runtime is missing, the old uninstall script is
also available in `runtime.old`. No stock UI/Bluetooth service recovery commands
are part of this experiment. Host tests cover rollback; real power-loss recovery,
loadability and uninstall remain human checks.

## Offline command and host tests

```sh
cargo run --locked --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml --bin chessnut-ble-probe -- packet-fen '6kq/8/8/8/8/8/8/8 b - - 17 42'
cargo test --locked --all-features --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml
```

`packet-fen` retains six-field metadata and prints all 35 bytes, Normal mode,
WithoutResponse and OFFLINE ONLY. It validates syntax and placement, not legal
chess, physical inventory, current position or motor feasibility. It never
starts a worker. `session`, `motion-enable`, `set-fen`, telemetry and LED commands
remain UNSUPPORTED. The SDK also encodes STOP as 35 bytes: delivery of both STOP
and auto-move remains UNVERIFIED until the MTU and board checkpoints.

The full gate includes the feature-gated mock-worker subprocess tests, Rust
supervision/signal tests, ELF/manifest checks, and shell install/rollback/remove
contracts in temporary directories. Plain cargo test omits the feature-gated IPC
suite; scripts/check.sh always includes it. [IPC.md](IPC.md) describes the M0
transport scaffold; native preflight uses a one-shot worker with fixed bounded
reports, not the future live session protocol.

## H1 instructions (verbatim from the plan)

**Human checkpoint H1 — Kindle-only, no board:** Human supplies SSH access and runs preflight. Record uname -m, firmware, GLIBC, loader/NEEDED check, BLE stack process presence, BT user/group mapping, whether libkindlebt.so can load, and app exit status. Check that the original Kindle Chess Puzzles still launches unchanged. **STOP** if binary fails to load, a privilege assumption fails, or system changes appear necessary.
