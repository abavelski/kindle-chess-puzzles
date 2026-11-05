# Chessnut Move integration handoff

Recorded 2026-11-05. This is the result of the standalone Kindle probe and the
input to a future integration plan; it is not an implementation task or a claim
that the puzzle app already supports a physical board.

## Decision and acceptance

There is enough evidence to implement an opt-in board integration. Further generic
BLE experiments are not a prerequisite. Preserve the proven transport and plan the
app-specific state transitions and acceptance tests next.

| Capability | Evidence / result |
| --- | --- |
| ARM deployment, permissions, library loading | PASS on the target Scribe |
| Connect, discover, subscribe and clean disconnect | PASS; intermittent stale daemon session mappings also observed |
| Realtime positions and manual moves | PASS: complete 38-byte packets, manual e6→e7 detected |
| MTU negotiation | Requested 500, negotiated **300**, matching successful connection callback |
| Battery | PASS: decoded 65%, not charging in the full telemetry test |
| Tracked-piece status | PASS: complete 139-byte response, 34 identities with coordinates and battery |
| Square LEDs | Human confirmed green e4 and successful all-off clearing |
| 35-byte Normal motor path | API/callback success and human-confirmed e2→e4 and e4→e2 movements |
| Final-position verification | PASS after human `done`, STOP and realtime re-enable; three target reports matched e4 |
| Longer STOP probe | e2↔e7 path exercised after the short moves; physical interruption was not demonstrated reliably |
| STOP / Ctrl+C | **Ineffective in the reported physical tests**, despite successful local STOP writes; user accepts this limitation |
| Captures, promotion, castling, multi-piece puzzle setup | Not validated as integrated physical flows |
| Sleep/wake and long-run reconnect reliability | Not established |
| Main-app coexistence | Probe remained isolated; original binary/data hashes unchanged during deployments and removal |
| Probe removal | Completed; no probe process/lock remained; all 121 original app/data files unchanged |

H1–H3 passed. H4 movement/return and final sensing passed, but reliable physical
interruption did not. The user explicitly accepted continuing with that limitation.
Do not relabel STOP as PASS or market cancellation as an emergency stop. Full H5
sleep/wake, stock-Bluetooth and post-removal visual app acceptance were not performed.

## Target and private device identities

Verified hardware: first-generation Kindle Scribe (Barolo), firmware 5.19.6,
`armv7l`, kernel 4.9.77-lab126, glibc 2.35, loader
`/lib/ld-linux-armhf.so.3`. Board advertised as `Chessnut Move`; board firmware
version was not established. Do not claim these findings cover another firmware.

Board BLE address, observed address type/source, macOS CoreBluetooth identifier,
and Kindle SSH endpoint are saved locally in
`local-kindle/chessnut-move/devices.private.json`, mode 0600, outside Git. The
parent `local-kindle/` path is ignored. No password is stored. The CoreBluetooth
identifier is a macOS identity, not a BLE MAC. Neither private identifier appeared
in Git history when checked. Do not copy either into tasks, examples, fixtures,
settings committed to Git, command histories or published reports. An observed
address may need rediscovery after a change; do not silently choose another board.
A new checkout does not contain this local file.

## Proven architecture and reusable material

Keep the current architecture: Rust owns application state and typed protocol
operations; a small, separate C worker owns KindleBT/ACE calls, callbacks and
Bluetooth privileges. This avoids using desktop BlueZ/btleplug on Kindle and
keeps vendor ABI/unsafe operations out of the puzzle core and renderer.

`support/chessnut-move/` contains a standalone, host-tested reference package:

- Rust FEN/SDK conversion, telemetry decoding, strict GATT discovery and bounded
  process IPC, with exact Cargo dependency pins and lockfile.
- Native worker, identity checks, lifecycle/parser, descriptor callback bridge,
  MTU observation, notification copying, write policy and cleanup ownership.
- Portable native tests and the optional adapter test against the real pinned headers.
- Tested ELF/GLIBC/COPY-relocation checks, narrow NEEDED localization, descriptor
  source patch and manifest helpers. These have no downloader or deployment entry point.
- Source/archive checksums, successful build metadata and third-party notices.

It is deliberately **outside the main Cargo workspace**. Nothing links it into,
launches from, or changes the current app. The probe-specific CLI, motion shell,
operator scripts and installer stay on the experiment branch; they are not the
production app API. No generated binary, vendor library, SDK archive or raw log
is needed in Git. The final archived experiment commit is linked in the support
README; use it for the complete historical build recipe if necessary.

The helper FEN parser validates syntax, not chess legality or motor feasibility.
The native `arm` permission is only a transport gate. The future app must supply
source matching, allowed targets, per-operation authorization and in-flight state.
Do not treat copied reference code as a completed production board feature.

## Versions, build and licensing

| Component | Verified pin |
| --- | --- |
| Rust | 1.85.1, target `armv7-unknown-linux-gnueabihf` |
| Zig | 0.13.0, C target `arm-linux-gnueabihf.2.35` |
| chessnut-move | **1.0.2**, `default-features = false` |
| KindleBT | `59e1d8e69a52e3cd58cee6fcc416d1774ad55791` |
| log.c | `f9ea34994bd58ed342d2245cd4110bb5c6790153` |
| Header reference | Amazon ACS RTOS reference FFS 1.15.0, pinned archive |
| Kindle SDK reference | Scribe1 5.17.3, release `build-20260325`, build-only library reference |

Exact URLs, archive hashes, toolchain variants and artifact receipts are in
`support/chessnut-move/SOURCE_PINS.json` and `BUILD_PINS.json`. C uses actual pinned
headers and `__typeof__` for function-pointer declarations, not reconstructed
opaque structures. Native preflight checks the real target's libraries, firmware,
architecture and identity before opening a session.

KindleBT and log.c are MIT; chessnut-move is Apache-2.0. Preserve notices when bundling.
Amazon headers/libraries are build/runtime references, not redistributable probe
payloads. The actual device supplies `libace_bt.so`; never replace `/usr/lib`.
Project-owned reference code has no newly invented distribution license.

Build facts that must survive integration:

1. Use ARM hard-float/glibc <=2.35, and inspect executable/shared-object ELF tables.
2. The Zig libc stub globals can cause dangerous ARM COPY relocations, even for a
   PIE. `-fPIC -pie` plus an actual relocation-table rejection fixed the worker
   startup crash. PIE alone was insufficient.
3. Some ACE SDK DSOs lack SONAME. Narrowly replace the known absolute SDK
   `DT_NEEDED` with its basename; reject arbitrary dependency rewriting and
   absolute paths/RPATH. Do not alter vendor binaries.
4. Validate dependencies actually imported by the built payload and supplied by
   the running firmware. Old SDK transitive names are not current device facts.
5. Preserve compiler/source pins and checksum-staged updates. Do not follow
   floating upstream branches or run the SDK's Linux compiler on macOS.

## BLE and protocol facts

| Role | UUID |
| --- | --- |
| Position service | `1b7e8261-2877-41c3-b46e-cf057c562023` |
| Position notification | `1b7e8262-2877-41c3-b46e-cf057c562023` |
| Command service | `1b7e8271-2877-41c3-b46e-cf057c562023` |
| Command write | `1b7e8272-2877-41c3-b46e-cf057c562023` |
| Command-response notification | `1b7e8273-2877-41c3-b46e-cf057c562023` |

The device's GATT table exposed these 128-bit UUID bytes in reversed order.
Compare all 16 bytes and parent services; validate write/notify properties and
resolve handles by discovery. Observed position/command/response handles were
258/514/516, but **do not hardcode them**. No public scanning API was available
in this KindleBT pin; address-based connection was used. A trusted external scan
was used earlier to obtain the address; no scanning daemon belongs in the app
without a separate decision.

Use the pinned SDK encoder/decoder rather than maintaining another piece map:

- Realtime enable: `21 01 00`, three bytes, WithResponse.
- Position notification: 38 bytes; packed board is bytes 2–33. Wire squares go
  h8, g8 … a1, low nibble before high nibble. Sensor data contains placement only:
  no side-to-move, castling, en-passant or move counters. Never invent that metadata.
- Normal auto-move: **35 bytes**, `42 21`, 32 packed board bytes, final flag **01**,
  WithoutResponse. Byte 00 as a force flag must not be substituted. Send the
  whole packet as one write; no truncation, ad-hoc fragmentation or write-mode change.
- SDK STOP: 35 bytes, `42 21` then a zero body. It is a separate command, not a
  guarantee that ending a BLE connection cancels the previous motor command.
- LED: 34 bytes, `43 20` plus packed colors, WithoutResponse. Off/red/green/blue
  are 0/1/2/3. One-square green and all-off were physically checked.
- Battery query: `41 01 0c`, WithResponse; decoded response was five bytes.
- Piece-status query: `41 01 0b`, WithResponse; complete response was 139 bytes,
  34 tracked identities. Coordinates are normalized 0–255 values, not square
  indexes. A 34-entry inventory does not mean 34 pieces occupy the playing area.
  Its identity encoding is different from the packed-position piece encoding.

Short `23 01 00` responses occurred alongside successful writes. Keep bounded raw
content and mark it undecoded; do not infer its acknowledgement semantics. It
must not satisfy a battery/status query or count as a valid position.

SDK position decoding needs at least 34 bytes. Unknown responses and incomplete
positions can be preserved diagnostically, but incomplete/invalid sensor input
must never grade a puzzle, authorize a move or satisfy final verification.

## MTU: the actual fix

Without negotiation, native and daemon callbacks both delivered 20-byte position
packets; Rust was not truncating them. Explicit negotiation changed them to complete
38-byte reports. Requested MTU 500; observed successful negotiated MTU **300**.
The resulting conventional ATT payload capacity is 297 bytes.

KindleBT's public header has no MTU-request wrapper. The actual firmware exports
`aceBT_bleRequestMtu(conn_handle, int mtu)`, with signature matching the pinned ACE
header; inspection of the firmware library's implementation confirmed the two
arguments and forwarding. The worker resolves that exact symbol with `dlsym` and
waits for the pinned KindleBT MTU callback evidence, matching the active connection.
No guessed vendor IPC structure is constructed. The private firmware-library copy
used for inspection has been deleted with build artifacts.

Required order: connect → discover/resolve → MTU request → matched successful MTU
completion → subscribe, completing each CCCD write → enable realtime/read/write.
API request success alone is not negotiation success. The probe required MTU >=41
for the complete 38-byte notification; a conventional 35-byte write itself needs
MTU >=38. Larger status responses need sufficient notification capacity too.
Negotiate again for each fresh connection; do not persist an assumed MTU.

The MTU observer currently depends on an exact pinned log.c callback format and
argument types, protected by a mutex. Keep the pin, or replace it with a tested
callback bridge during integration. An upstream update requires ABI/contract tests.

## Descriptor and asynchronous-operation fixes

The ACE database stores one descriptor in the embedded `gattDescriptor`. Multiple
descriptors use the counted `descList`. Searching only the list caused subscriptions
to fail before an API write. Support both layouts, validate CCCD UUID 2902, reject
missing/ambiguous descriptors, and never guess characteristic handle + 1.

The pinned KindleBT callback table omitted forwarding the descriptor-write completion
to its public application callback. The small bridge and anchored source patch in
the reference package fix that omission. Both subscribing and unsubscribing must
wait for the matching descriptor handle/connection and successful callback before
starting another operation. Overlap previously produced pending-GATT status -15.

One owner thread calls vendor APIs and writes protocol output. Callbacks copy
bounded data before returning; they must not retain vendor-borrowed notification
buffers. Retain write buffers until completion, including timeout/error paths.
Emergency cleanup STOP has separate persistent storage, so it cannot overwrite a
buffer still borrowed by an outstanding API operation.

IPC is versioned JSON, bounded to 4096-byte frames and 512-byte notification data,
with bounded queues (16), explicit write kind and monotonically increasing IDs.
One request is outstanding. Errors, queue overflow, disconnects and malformed
frames are terminal. Do not automatically replay a motor command after any error,
worker replacement, reconnection or wake.

BusyBox redirects stdin for background jobs: explicitly preserve the input descriptor
before launching a background controller, or interactive commands silently disappear.
Continuous 10 Hz position output also obscured typing. Keep routine telemetry in
bounded/private diagnostics; screen output should show state changes and deliberate
observations, not every packet. Production logs need rotation and address redaction.

## Motion, completion and the accepted STOP limitation

Position reports pause during automatic movement. Both initial motor scripts moved
the pawn successfully but then timed out at 45 seconds waiting for unsolicited
position resumption. That was a verification-flow error, not proof of failed motion.

The physically successful completion flow was:

1. Read and match the actual source, print the target, mode and exact bytes.
2. Obtain explicit per-target confirmation; send one Normal command.
3. After the human sees completion, `done` sends STOP, re-enables realtime once,
   and requires three consecutive exact target placements.
4. Disconnect and clear authorization. A return is a new target and confirmation.

The app must plan a usable completion UI around this fact. Do not claim that
local write completion or a fixed sleep proves target arrival. The 45-second probe
observation limit is an operational fail-safe, not measured board latency. The
reported 8771 ms until final verification included human input and telemetry;
it was not motor travel time. Automated completion without a human `done` remains
an integration design/firmware investigation, not an established capability.

STOP and Ctrl+C both reached native 35-byte writes with successful local API and
callback status, including a STOP about one whole timestamp-second after a return
command. The human reported that movement continued. Timing/firmware/transport
cause remains unresolved; the manufacturer documents STOP, so do not generalize
that already-sent BLE commands are inherently unstoppable. Do not invent a different
STOP packet to hide the limitation.

**Accepted operating assumption: a confirmed motor command may finish despite
STOP, Ctrl+C, app exit or disconnect.** Keep the physical travel area clear, user
confirmation, Normal mode, a single active target, no automatic retries, and a
known physical power-off/recovery route. Cancellation can cancel pending software
work; it is not a verified physical emergency stop. Surface that distinction in
the integration UI. Do not use speculative Stop success as permission for another
motor command or for moving the stock UI into an incompatible state.

## Connection reliability and lifecycle

Bluetooth privileges were verified against the actual `btmanagerd` process:
worker UID/GID 1003/1003, even though the account's primary group was 150. Verify
both account and daemon identity, then drop only the worker. Do not hardcode the
account's primary GID or drop the app/UI controller to that identity.

The board LED can be steady blue while the probe is unconnected. In one repeated
failure, the daemon established the BLE ACL/GATT link but routed its connection
callback to an old session UUID. The new worker saw no callback/context/handle and
timed out (-2) after about 10 seconds. A radio off/on cycle did not clear that
particular stale mapping. A later read-only run with the same executable recovered
and passed; the user's exact intervening recovery actions were not established.
Do not promise a radio toggle or reboot as a proven fix.

Require the current session's matched connection callback and valid context;
never adopt an unknown handle based on the board LED. Isolate SDK errors from
observed callback/context errors in diagnostics. Firmware metrics/session-dispatch
warnings also appeared during otherwise successful cleanup; do not equate every
vendor log error with a failed board operation. Conversely, zero cleanup API returns
after an unconfirmed connection do not prove the connection was owned/disconnected.

Keep one board owner, finite timeouts, TERM/HUP/INT/EOF cleanup, persistent callback
state and a manual recovery route. The CLI used a 55-second worker cleanup grace;
initial native write completion could delay terminal command processing by up to
five seconds. The integrated UI must remain responsive while the worker waits.
Do not restart/replace Bluetooth services, manipulate global radio configuration,
or patch daemon memory as an automatic recovery action.

## What the next integration plan must specify

No numbered integration task exists yet. Create explicit tasks and a dedicated
integration branch from current main; do not start by merging the experiment.

1. **Opt-in UI and ownership.** Connection choice, private address storage,
   disconnected/connecting/ready/moving/verifying/out-of-sync/error states,
   best-effort STOP wording, target preview and per-target/session authorization.
   No board movement on startup, reconnect, collection load or wake.
2. **State adapter, outside core.** Keep BLE, handles, worker processes and device
   paths in the platform/service layer. Feed ordinary logical actions into existing
   deterministic grading. Core/render must not import KindleBT or shell concepts.
3. **Physical move recognition.** Compare complete stable placements and recognize
   finished transactions; piece lifts, capture removal and transient positions are
   not completed moves. Use existing authored solutions/precomputed positions and
   existing UCI behavior; no Kindle runtime engine or general legality subsystem.
4. **Automatic opponent reply boundary.** The app already applies stored opponent
   replies logically. Represent the difference between that logical target and the
   physical board still showing the solver move. Prevent telemetry echo from grading
   a reply or solving another puzzle. Verify physical sync before another transaction.
5. **Wrong moves and ambiguity.** Define feedback and physical correction without
   silently overwriting grading/progress, sending speculative moves or treating an
   ambiguous capture/promotion/castling transition as an ordinary two-square move.
6. **Setup and inventory.** Plan and test full puzzle setup, captures/parking,
   promotion pieces and castling separately. The proven short pawn path is not
   proof that arbitrary midgame rearrangements or captures are physically supported.
7. **Navigation/preview semantics.** Define reset, next/previous, flip, solution,
   Free Board and analysis preview behavior while connected/moving. Orientation
   changes display only; they do not rotate sensor coordinates. Preview must never
   mutate grading, durable progress or move the physical board unexpectedly.
8. **Accepted non-interruptibility.** Decide what navigation/exit/sleep controls do
   while motors may continue; no queued new target or false stopped state. Re-read
   and reconcile after errors, sleep/wake or a lost worker. Never replay automatically.
9. **Persistence and packaging.** Preserve immutable collections, separate progress
   and protected malformed/future files. Add optional dependencies, worker/library
   packaging and private rotating logs only through scoped tasks and pinned builds.
10. **Tests and device acceptance.** Red/green host tests with recorded synthetic
    board events, fake transport, delayed/corrupt/truncated packets, disconnects,
    concurrent input and progress invariants; snapshots for visible states. Follow
    with supervised physical checks of the actual integrated puzzle flows.

Known acceptance limits must be explicit in those tasks. A user-accepted limitation
can change a requirement, but must not turn an unverified capability into PASS.

## Cleanup and retained evidence

The owner-checked installed uninstaller removed `/mnt/us/chessnut-ble-probe/`,
including binaries, private runtime address, incoming bundles and all probe-owned
logs. No probe process/lock remained and no Chessnut-named path remained under
`/mnt/us` (depth 3) or `/tmp` (depth 2). All 121 original app/data files had identical
hashes before and after removal. Shared OS logs were not erased; system services,
configuration and the existing app install were not modified. A small private
cleanup receipt remains beside the private identifiers.

Raw manual reports, build/device logs, captures, private library copies, downloaded
SDKs and generated artifacts are deleted locally. Only source, tests, pins,
license notices and curated findings belong in Git. Private identifiers remain
outside the probe tree so they survive removal and branch switching.

The final probe source passed 54 Rust tests, 26 Python tests, the pinned-header
native adapter contracts, ARM build, staging and remote preflight. These establish
code/build contracts; physical results in the matrix remain separate evidence.
The clean support package passed 20 Rust tests, 11 Python/native-build test
cases and the actual pinned-header adapter contract. It is not wired into the
app or parent workspace. Neither branch should commit raw output or identities.

## Primary references

- [Manufacturer Move protocol](https://github.com/chessnutech/chess_move_api)
- [Manufacturer quick guide](https://chessnut.us-east-1.linodeobjects.com/Chessnut_Move_Robotic_Chess_Set_Quick_Guide.pdf)
- [Pinned Rust SDK](https://github.com/daymxn/chessnut-move-rs/tree/4f0468097b48c35151beb8d0743b169c3ae7bce3)
- [Pinned KindleBT](https://github.com/Sighery/kindlebt/tree/59e1d8e69a52e3cd58cee6fcc416d1774ad55791)
- [Original probe plan](CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md)
- [Reusable reference package](../support/chessnut-move/README.md)
