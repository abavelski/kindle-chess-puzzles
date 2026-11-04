# Chessnut Move / KindleBT probe

An isolated CLI experiment for a first-generation Kindle Scribe, ARMv7
hard-float, firmware 5.19.6, glibc 2.35. The existing chess application and
parent workspace are unchanged. **M1 is Implemented; H1 passed on 2026-11-04.**
The corrected build is deployed and preflight exits 0. The user confirmed the
original chess app displays, responds and exits normally; its binary and 120
data files retain their pre-update hashes. **M2 is deployed and passes Kindle preflight; H2 is pending.**
Mac-side discovery matched the expected board UUIDs/properties without writes.
The Kindle session is untested pending the board’s physical BLE address. See [device results](notes/DEVICE_RESULTS.md) and
[the startup diagnosis and fixes](notes/M1_STARTUP_CRASH.md).

## Progress

The scope is [the probe plan](../../docs/CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md).
All changes and completion records stay in this experiment.

- [x] M0 — Implemented: pinned independent Rust workspace, FEN/SDK encoding,
  expected packet fixtures, bounded/versioned IPC and mocked failures.
- [x] M1 — Implemented: host gates, ARM build, ELF/dependency checks, scoped
  deployment, H1 preflight and human original-app verification passed.
- [ ] M2 — Host-ready: discovery-only session, connection diagnostics, cleanup
  and host tests implemented; deployed preflight passed. Human H2 remains pending.
- [ ] M3 — Telemetry/LED/MTU evidence; H3 read-only checkpoint.
- [ ] M4 — Guarded motor control; H4 motors checkpoint.
- [ ] M5 — Reliability/rollback; H5 independence checkpoint.

H1 preflight: **PASS**. Physical board checkpoints H2–H5: **NOT TESTED**.
Library loading and worker privilege transition are verified; Bluetooth API
behavior and board communication remain unverified.

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

**The currently installed M1 build passed H1 on 2026-11-04.**
The installed M2 build is from commit `a6005cb`; its preflight passed on
2026-11-04. M1 build `138cc25` previously passed H1. Preflight alone needs no
board address. H2 needs the current physical BLE address.
For a future deployment, run build.sh and stage.sh first, then substitute the
private SSH hostname:

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

The M2 launcher accepts **preflight** or **session --address <address>**. It checks checksums and the loader,
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

## M2 discovery session and H2 preparation

M2 is deployed; Kindle preflight passed. H2 is **BLOCKED** pending a current
physical BLE address and confirmation that the board is awake in BLE mode.
Mac-side discovery succeeded, but a subsequent Mac reconnect timed out.
After resolving those prerequisites,
run over interactive SSH (keep the original chess app closed):

```sh
# Read the address privately; avoid putting its literal value in shell history.
printf 'Current board BLE address: '
read -r board_address
sh /mnt/us/chessnut-ble-probe/runtime/launch.sh session --address "$board_address"
unset board_address
```

The board must be in BLE mode, with the official app and scanner disconnected.
Use the current address from a trusted scanner; a randomized address may change.
There is no scan, pairing, address fallback or automatic reconnect. Address
arguments remain visible to local process inspection, but probe diagnostics
redact them and suppress upstream stdout/stderr, which may contain addresses.
No address is saved in configuration. Keep any captures private and uncommitted.

The controller connects and discovers immediately. It prints service/characteristic
UUIDs, handles and raw property bits, with WithResponse, WithoutResponse and
Notify indicators. Both services and all three characteristic UUIDs from the
pinned SDK must match, under their correct parent services. Required notification
properties and both write modes must be advertised. These are property checks;
no subscription, read, descriptor write or command write is performed. Full-byte
forward and reverse 128-bit UUID ordering are checked as consistent alternatives;
the selected `uuid_byte_order` is reported. Mixed order, ambiguous duplicates,
missing UUIDs/properties or malformed/oversized databases abort the session.

```text
probe> services       # show/validate the cached discovery database again
probe> disconnect     # release database, disconnect, deregister and close session
probe> connect        # explicit reconnect using the supplied address and a fresh worker
probe> services
probe> quit
```

A clean reconnect must again print `discovery=PASS`. Any other command prints
UNSUPPORTED and sends no operation to the board. EOF, quit, Ctrl+C, HUP and TERM
request cleanup. Unexpected link loss is reported even while waiting at the prompt.
No motor, LED, battery, piece-query, STOP or notification commands are implemented
in M2; no write is sent on interruption because this milestone never starts motion.

Each IPC request has a 45-second deadline. Pinned KindleBT generally waits up to
10 seconds for its internal callbacks, but vendor calls may still stall. The
controller allows up to 55 seconds for a terminated worker to unwind, then forces
reaping and reports failed cleanup. Failed setup unwinds only acquired resources;
there are no command retries. If cleanup fails, stop and verify disconnection before
retrying. Do not change stock services, radio settings or accounts to bypass errors.
The worker's library loading/privilege checks precede any session API call. See
[the native boundary](native/README.md), [IPC](IPC.md), and
[M2 host results and limitations](notes/M2_HOST_RESULTS.md).

**Human checkpoint H2 — Discovery:** Human places the Move in BLE mode, disconnects its official app, obtains its current BLE address using a trusted scanner (phone/PC), passes it privately to the CLI, and confirms the expected Chessnut service and characteristic UUIDs and a clean reconnect after disconnect. Record connection outcome and redacted logs. **STOP** if discovery/permissions fail or the UUIDs do not match.

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
are part of this experiment. Host tests cover rollback; real power-loss recovery
and device uninstall remain untested. Library loadability and original-app
operation passed H1; scoped removal is covered by host tests.

## Offline command and host tests

```sh
cargo run --locked --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml --bin chessnut-ble-probe -- packet-fen '6kq/8/8/8/8/8/8/8 b - - 17 42'
cargo test --locked --all-features --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml
```

`packet-fen` retains six-field metadata and prints all 35 bytes, Normal mode,
WithoutResponse and OFFLINE ONLY. It validates syntax and placement, not legal
chess, physical inventory, current position or motor feasibility. It never
starts a worker. `motion-enable`, `set-fen`, STOP, telemetry and LED commands remain
UNSUPPORTED. M2 session permits connection/discovery only. The SDK also encodes STOP as 35 bytes: delivery of both STOP
and auto-move remains UNVERIFIED until the MTU and board checkpoints.

The full gate includes the feature-gated mock-worker subprocess tests, Rust
supervision/signal tests, ELF/manifest checks, and shell install/rollback/remove
contracts in temporary directories. Plain cargo test omits the feature-gated IPC
suite; scripts/check.sh always includes it. [IPC.md](IPC.md) describes the M0
transport scaffold; native preflight uses a one-shot worker with fixed bounded
reports, not the future live session protocol.

## H1 instructions (verbatim from the plan)

**Human checkpoint H1 — Kindle-only, no board:** Human supplies SSH access and runs preflight. Record uname -m, firmware, GLIBC, loader/NEEDED check, BLE stack process presence, BT user/group mapping, whether libkindlebt.so can load, and app exit status. Check that the original Kindle Chess Puzzles still launches unchanged. **STOP** if binary fails to load, a privilege assumption fails, or system changes appear necessary.
