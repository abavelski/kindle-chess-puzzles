# Chessnut Move on Kindle Scribe: standalone KindleBT proof-of-concept plan

**Status:** Historical probe plan. See [the integration handoff](CHESSNUT_MOVE_INTEGRATION_HANDOFF.md) for verified outcomes, cleanup and the user-accepted physical STOP limitation. This original plan is not a claim that every checkpoint passed.
**Target:** First-generation Kindle Scribe (Barolo), jailbroken, firmware 5.19.6, ARMv7 hard-float, glibc 2.35.
**Goal:** Establish whether the Kindle can control and read a Chessnut Move directly over BLE using KindleBT, **before** changing Kindle Chess Puzzles.
**Audience:** Codex implementing one milestone at a time, and a human operating the actual Kindle and board.

## 0. Rules and boundaries (read first)

1. This is a **separate, deployable test application**, not a feature of Kindle Chess Puzzles. Do **not** modify the existing app, its event loop, chess-core, renderer, input, FBInk, lifecycle, settings, puzzle data, packaging, existing scripts, or existing install.
2. All new implementation belongs under **experiments/chessnut-move-ble-probe/**. Keep its Rust crate/workspace, native C source, build scripts, tests, staging, and installer independent. Cargo should treat it as a separate workspace (an explicit local [workspace] table) rather than automatically adding it to the parent workspace. Keep outputs in its own ignored build directory.
3. Deploy only to **/mnt/us/chessnut-ble-probe/**; never overwrite **/mnt/us/kindle-chess/**, its document/scriptlet, its state, or its launch process. Do not copy libraries into /usr/lib, replace stock system files, modify Bluetooth daemons, or change the Kindle's global Bluetooth configuration to make the test pass.
4. The initial user interface is an **interactive SSH terminal CLI**, not a framebuffer or touch UI. It must be independently installable, runnable, stoppable, and uninstallable. Do not start or pause awesome/Xorg, grab /dev/input, use FBInk, install a library scriptlet, or depend on the main app being installed. Packaging as KPM is **out of scope** until the CLI experiment has passed.
5. Follow the repository's AGENTS.md guidance on tests, device recovery, safe privileges, and human checkpoints. Existing phase-2 and game-review branch rules do not authorize unrelated edits. Scope work to this experiment.
6. **Do not claim hardware success from a cross-compile or mocks.** At each human checkpoint, stop implementing the next risky stage until the human records PASS/FAIL with relevant logs.
7. Do not secretly enable motor motion. Only permit auto-move after a human has enabled motion for that session and explicitly confirmed each target. Default to Normal mode, never Force mode. Provide a STOP action available while connected.
8. Do not assume the Kindle's BLE user/group IDs, that a Bluetooth MAC address remains stable, or that MTU negotiation works; test and record each fact. Use timeouts for all external calls and restore/close resources on error, SIGINT, SIGTERM, and normal exit.
9. This plan is the only deliverable of the planning task. Implementation is a **separate future task**; do not start implementing it just because this file was added.

## 1. Known facts and unverified assumptions

**From this repository:** README.md, docs/KINDLE_SCRIBE.md, and docs/BUILD_DEPLOY.md establish a physically validated Barolo Scribe on firmware 5.19.6, ARMv7 hard-float with glibc 2.35, plus an SSH deployment workflow. The main app's Zig build uses target **arm-linux-gnueabihf.2.35**; its Rust target is **armv7-unknown-linux-gnueabihf**. Those are useful build constraints, not permission to reuse the main app's runtime.

**KindleBT:** https://github.com/Sighery/kindlebt (docs: https://sighery.github.io/kindlebt/ ). It wraps Amazon's ace_bt / btmanagerd stack rather than desktop Linux BlueZ. Its documentation reports BLE/GATT tests on a first-generation Scribe on firmware **5.17.2**. Its public header exposes session creation, BLE registration, GATT-client registration, connect by Bluetooth address, service discovery, characteristic read/write, notifications, and disconnect. It does **not** visibly expose a general-purpose BLE scanning API or an explicit MTU-request API. Compatibility on firmware **5.19.6** and with the Chessnut Move is unverified. The supplied C example drops to Bluetooth UID/GID **1003**; first verify the actual account/permissions on this Scribe. Example: https://github.com/Sighery/kindlebt/blob/main/examples/basic_usage.c .

**Chessnut protocol:** https://github.com/daymxn/chessnut-move-rs (crate **chessnut-move**, version **1.0.2** as inspected). The protocol is independent of its optional btleplug transport, and **Command::auto_move(position, AutoMoveMode::Normal)** constructs a complete **35-byte** board-position command. Its native btleplug backend is **not** expected to work unmodified on the Kindle's Amazon Bluetooth stack. Use the crate's transport-independent types/encoder, with **default-features = false**; pin exact resolved versions in the experiment's Cargo.lock. The SDK describes position notifications as unavailable during automatic motion. See its README and src/protocol/command.rs.

**BLE characteristics (from the Chessnut SDK/Move API):**

| Role | UUID |
| --- | --- |
| Position notification service | 1b7e8261-2877-41c3-b46e-cf057c562023 |
| Position notification | 1b7e8262-2877-41c3-b46e-cf057c562023 |
| Command/response service | 1b7e8271-2877-41c3-b46e-cf057c562023 |
| Command write | 1b7e8272-2877-41c3-b46e-cf057c562023 |
| Response notification | 1b7e8273-2877-41c3-b46e-cf057c562023 |

The 35-byte auto-move command uses the command-write characteristic and is marked **WriteKind::WithoutResponse** in the Rust SDK. Enabling position reports, querying battery/piece status, LED control, STOP, and notification decoding must use the corresponding SDK protocol operations, not arbitrary invented packets.

**MTU is a critical gate.** A conventional 23-byte ATT MTU permits only a 20-byte write-without-response payload. Sending a 35-byte command conventionally needs an effective ATT MTU of at least **38**; full position notifications may need a larger MTU too. Chessnut documentation recommends requesting an MTU of **500** if full position data is not received. KindleBT has an internal MTU-update callback but no clearly documented public MTU negotiation call. **Do not assume an automatic increase, silently truncate, split the 35 bytes into unrelated writes, or change the required write mode to hide the problem.** Measure it and treat lack of support as a blocker or documented research task.

## 2. Deliverable and design contract

Implement a **standalone native CLI bundle** deployable over SSH. A thin C BLE worker uses KindleBT, while a Rust controller uses the chessnut-move protocol crate and supplies a simple interactive command shell. Separate processes are preferred for failure isolation and to avoid dropping the controller's privileges; only the BLE worker should assume the verified Bluetooth identity.

Proposed tree (keep all files inside this directory):

~~~text
experiments/chessnut-move-ble-probe/
  README.md                    # prerequisites, build, commands, risks, cleanup
  Cargo.toml                   # independent Rust workspace; no parent workspace changes
  Cargo.lock                   # pinned dependency resolution
  src/main.rs                  # Rust CLI, FEN -> Position -> Command, prompt/state
  src/protocol.rs              # line-oriented IPC structs/codec (as needed)
  native/kindlebt_worker.c     # small KindleBT-based BLE subprocess
  native/README.md             # build/link/ABI details for C worker
  scripts/build.sh             # host build, fail-fast
  scripts/stage.sh             # self-contained stage directory and manifest
  scripts/deploy.sh            # copy only experiment files over SSH
  scripts/uninstall.sh         # remove only the experiment install
  scripts/verify-stage.sh      # ELF/GLIBC/libs/manifest checks
  tests/                       # host-only FEN, byte, IPC and failure tests
  notes/DEVICE_RESULTS.md      # human hardware findings, initially NOT TESTED
~~~

The structure is a proposal, not a mandate to add files before needed. Use the minimal number required by the active milestone. Generated archives, downloaded SDKs, binaries, secrets, private addresses and device logs should be gitignored.

**Execution/privilege model:**
- Launch the Rust controller over the existing SSH connection; it forks/spawns the C BLE worker with stdin/stdout pipes (stderr reserved for diagnostics).
- The worker validates and changes to the actual Bluetooth UID/GID **before opening an ace_bt session**; reject failed privilege transitions. Never drop the whole SSH session or the Kindle chess app to that identity. Preserve clean process termination and pipe closure.
- The worker opens a KindleBT session, registers BLE and the GATT client, connects to the supplied board address, discovers services, resolves characteristics, subscribes to position and response notifications, writes commands, and closes all resources on shutdown.
- Maintain a single connection and a single outstanding command as appropriate. Do not equate write acceptance with successful physical movement; inspect acknowledgements/position state as supported.
- IPC must be **versioned and bounded** (newline-delimited JSON or a simple documented framed protocol). Minimum requests: CONNECT(address), DISCOVER, SUBSCRIBE, WRITE(bytes, write_kind), DISCONNECT, SHUTDOWN. Minimum asynchronous reports: connection state, operation status, notifications(source, bytes), errors. Enforce length limits, explicit statuses/timeouts, graceful EOF and worker-crash reporting. Do not parse diagnostic stderr as protocol data.
- The Rust side calls **chessnut-move** only for typed protocol encoding/decoding; initially avoid Tokio and btleplug. Test all encoders with mocks on the development machine. A future SDK AsyncTransport implementation is optional, not required to prove hardware support.
- Native KindleBT may need the Sighery Kindle SDK and proprietary header/library references at **build time**. Obtain build dependencies from their documented sources, pin release/commit + checksums, and document actual versions. Link against the Kindle's **existing** ace_bt runtime libraries; do not redistribute proprietary Kindle libraries in the bundle.

**Runtime interface (minimum; exact syntax to document in README):**

~~~text
chessnut-ble-probe preflight           # hardware/ABI/permission/library checks, no connection
chessnut-ble-probe session --address XX:XX:XX:XX:XX:XX
probe> services                        # show required UUID matches/properties
probe> battery                         # query board, display parsed response
probe> pieces                          # query tracked-piece status if supported
probe> watch 20                        # observe physical board changes for 20 seconds
probe> leds e4 green                   # visually identify one square, with clear option
probe> leds-clear
probe> packet-fen "rnbqkbnr/..."       # offline dry run: parsed target, 35 bytes, no write
probe> motion-enable                   # prominent explicit human opt-in
probe> set-fen "rnbqkbnr/..."          # print target, require y/N, Normal mode only
probe> stop                             # send stop command in an active session
probe> disconnect
probe> quit
~~~

If a helper cannot provide a command, print **UNSUPPORTED** or a specific error; do not fake successful behavior. STOP should be best-effort on Ctrl+C/interruption while a session remains connected; document how to stop the board physically if the BLE link dies.

Store optional logs in **/mnt/us/chessnut-ble-probe/logs/**. Redact Bluetooth MAC addresses by default (or explicitly mark local device logs as private); do not commit addresses, credentials, or full raw identity records. Include timestamps, build/firmware versions, phase, result, write byte length/type, notification length, and the reason for errors.

## 3. Codex execution milestones, with stop gates

### Milestone 0 — Ground truth and host-only scaffold (no device action)

Codex:
- Re-read AGENTS.md, README.md, docs/KINDLE_SCRIBE.md, docs/BUILD_DEPLOY.md, docs/TESTING.md and this plan. Inspect upstream KindleBT headers, example, release notes, and chessnut-move v1.0.2 code. Record source pins, licensing, and exact ABI expectations. Verify no relevant newer release invalidates these assumptions.
- Add the isolated experiment workspace and basic README; make no edits outside experiments/chessnut-move-ble-probe/.
- Implement host tests that convert legal FEN **piece placement** (including empty squares and color) to the SDK Position, enforce 8 ranks x 8 files, reject bad characters and invalid input, and compare a few expected command bytes and length with upstream protocol fixtures. Keep full 6-field FEN metadata on the Rust side; the BLE command encodes placement only.
- Add a mocked IPC worker for deterministic tests: success, wrong UUID, malformed notification, write failure, timeout, disconnect, SIGTERM, and interrupted session.
- Provide independent host commands, e.g. cargo test --manifest-path experiments/chessnut-move-ble-probe/Cargo.toml, without changing the parent workspace.

**Automated gate M0:** Host tests pass, protocol writes distinguish WithResponse/WithoutResponse, no BLE access needed, and git diff contains only experiment files. Do not proceed if baseline compiler/dependency versions cannot be pinned.

### Milestone 1 — Build, stage and install a harmless standalone binary

Codex:
- Build ARMv7 hard-float artifacts for glibc **<=2.35**, using the documented Kindle build toolchain/SDK as a reference; do not modify the existing chess app's build scripts. For KindleBT linking, follow Sighery's documented HF release/SDK instructions.
- Verify all executable/shared-object ELF headers, architecture, interpreter, NEEDED libraries, and GLIBC versions before deployment. Include an explicit check for required Kindle-side vendor libraries. Avoid binaries linked against newer host glibc.
- Stage Rust executable, C worker, libkindlebt.so (and any **redistributable** required dependencies), launcher, checksums, notices, and an exact build/source receipt in the experiment's stage folder. Keep dynamic-library paths local (e.g. LD_LIBRARY_PATH relative to the launcher), not /usr/lib.
- Implement scoped deploy and uninstall scripts. Installs/updates must be atomic or recoverable, verify checksums, and never alter /mnt/us/kindle-chess. Support SSH host/port arguments; do not embed personal addresses or credentials. The first installed command is **preflight** only.

**Human checkpoint H1 — Kindle-only, no board:** Human supplies SSH access and runs preflight. Record uname -m, firmware, GLIBC, loader/NEEDED check, BLE stack process presence, BT user/group mapping, whether libkindlebt.so can load, and app exit status. Check that the original Kindle Chess Puzzles still launches unchanged. **STOP** if binary fails to load, a privilege assumption fails, or system changes appear necessary.

### Milestone 2 — Establish GATT connection; no motors

Codex:
- Implement KindleBT session, BLE registration, GATT-client registration, address-based connect/disconnect, service discovery + database retrieval, required characteristic lookup, callback handling, and timeout/error handling. Follow the public KindleBT C example for callback layout and cleanup. Check UUID byte ordering/length.
- Use an externally supplied address; **no scanning requirement** for this milestone because KindleBT's public header does not expose scanning. Accept that randomized BLE addresses may invalidate a saved address, and report a clear failure rather than silently connecting to a different device.
- Implement services and connection diagnostics, but do not write any motor, LED, or other command in this milestone.

**Human checkpoint H2 — Discovery:** Human places the Move in BLE mode, disconnects its official app, obtains its current BLE address using a trusted scanner (phone/PC), passes it privately to the CLI, and confirms the expected Chessnut service and characteristic UUIDs and a clean reconnect after disconnect. Record connection outcome and redacted logs. **STOP** if discovery/permissions fail or the UUIDs do not match.

### Milestone 3 — Read-only telemetry, LED test, and MTU/write proof

Codex:
- Subscribe to **both** position and command-response characteristics. Enable realtime updates using the Rust SDK command. Decode or display bounded raw notifications and compare an observed position to a known physical arrangement.
- Add battery and tracked-piece status commands using SDK constructors and responses; test that normal status queries complete with correct timeouts. Add controlled LED on/off test with cleanup.
- Instrument the actual MTU when available from the KindleBT/ace_bt stack, and log **actual** write acceptance, byte length, write kind, callback/status, and notification lengths. Determine whether the full 35-byte **without-response** command can be written without truncation; examine KindleBT's limited public MTU support. A 35-byte command requires an effective ATT MTU of at least 38 for a conventional ATT write; the official Chessnut documentation suggests 500 for complete position reports.
- **Do not attempt motor motion yet.** The 35-byte write must not be sent as a live auto-move test at this milestone. Demonstrate the exact payload length/type using offline encoding and any safe lower-level capability check. If no non-motor mechanism can prove the path, explicitly mark 35-byte write delivery **UNVERIFIED** and defer its proof to H4, rather than sending an unsafe packet or pretending it is tested.

**Human checkpoint H3 — Read-only:** Human moves one physical piece by hand, checks real position notification and decoded FEN, records battery/status and LED behavior, and turns LEDs off. Human and Codex review the MTU evidence. **STOP** for missing/truncated notifications, unsafe errors, or unexpected stock-Bluetooth interference. An unresolved MTU question can continue to a carefully controlled H4 only if all other safety requirements pass and the human explicitly agrees.

### Milestone 4 — Controlled robotic movement (highest-risk checkpoint)

Codex:
- Implement a dry-run **packet-fen** path before any write. Compare the target position with a known board state. Validate target piece counts, colors and representation; refuse malformed FEN, unknown present position, or impossible/obviously unsupported requests.
- Implement motion-enable, one-command confirmation, NORMAL-only auto-move, STOP, movement timeout, and post-motion verification. Avoid automatically sending moves in response to startup, retries, reconnect, scripts, or tests. Do not queue a second move while the first is active.
- Start with a **small reversible position change** from an accurately recognized arrangement, such as one pawn from e2 to e4 and back, **not** a major midgame rearrangement or a capture. Preserve any necessary physical piece setup/parking instructions. Do not infer motor completion from a successful GATT write.
- Report failure with exact stage and preserve sufficient diagnostics; suppress automatic retries of motor commands. If notifications are paused during motion, poll/wait for settled position appropriately without flooding the board.

**Human checkpoint H4 — Motors:** Human physically clears the board's travel area, confirms piece batteries/registration, checks both the printed source and target FEN, explicitly enables motion, and watches the physical move. Test success, final detected squares, STOP behavior on a safe test, return to original position, and manual recovery. Record observed latency but set **no guessed timing acceptance threshold**. **STOP** immediately for collision, wrong movement, unexpected repeated motion, missing STOP, or excessive interference.

### Milestone 5 — Reliability, delivery and rollback

Codex:
- Exercise sequential connections, disconnect/reconnect, worker crash, stale address, worker timeout, KeyboardInterrupt/SIGINT/SIGTERM cleanup, and repeated short read-only sessions. Do not auto-replay motor commands after reconnection.
- Test a Kindle idle/sleep/wake cycle only with human assistance; surface disconnection explicitly. Check interaction with stock Kindle Bluetooth services and the existing chess app at separate times, not simultaneously owning the board.
- Finish README with exact reproducible install/run/uninstall steps, build pins/checksums/ABI, limitation table, sample redacted outputs, and troubleshooting. Keep all changes isolated to experiment files. Do not add this test to the main app or its installer.
- Collect acceptance evidence in notes/DEVICE_RESULTS.md; classify every gate PASS/FAIL/BLOCKED/NOT TESTED.

**Human checkpoint H5 — Independence:** Human verifies no boot/startup persistence, clean uninstall removing only /mnt/us/chessnut-ble-probe, original Kindle app and puzzle data still work, no third-party app is left connected, and stock Bluetooth still operates.

## 4. Test matrix and exit criteria

| Capability | Automated evidence | Human hardware evidence | Required to claim end-to-end PASS |
| --- | --- | --- | --- |
| Separate build & deployment | Independent manifest, stage checksum, ELF/GLIBC checks | Launch & uninstall on Scribe | Yes |
| KindleBT compatibility on firmware 5.19.6 | ABI/source pin verification | Native session opens | Yes |
| Address-based BLE connection | Mocked connection/disconnect errors | Correct Move connects/reconnects | Yes |
| GATT services/characteristics | UUID and callback tests | Expected UUIDs discovered | Yes |
| Battery and piece status | Decoder tests | Valid response or documented device limitation | Battery yes; piece-status status documented |
| Realtime position notifications | Packed-position fixtures | Hand move causes correct FEN | Yes |
| LED write | Command length/encoding test | Physical square lights and clears | Yes |
| Full 35-byte auto-move write | Correct packet, mode, no truncation in code | Accepted on real board | Yes |
| Physical auto-move | No automatic sends in tests | Correct small move, return and STOP | Yes |
| Recovery and cleanup | Mock timeout/signal/worker-crash tests | Clean exit/restart, no system alterations | Yes |
| Kindle sleep/wake | State-machine tests | Observed reconnect or explicit degraded behavior | Document behavior; do not claim support without test |

**Definition of success:** The standalone app on the Scribe opens its own KindleBT BLE connection, reads physical Move positions, sends the correct complete auto-move target, observes actual robotic movement to that position, can stop motion, and exits/reconnects without affecting the existing puzzle app or Kindle system.

If one requirement is blocked (for example MTU negotiation or randomized-address discovery), **do not change the main app**. Record the exact blocker, upstream/API evidence, a minimal reproduction, and the next experiment. A possible fallback is an external Windows/Linux BLE bridge controlled by Wi-Fi, but that is a separate decision and is **not** the deliverable here.

## 5. Manual checkpoint record template

Append a dated record to **experiments/chessnut-move-ble-probe/notes/DEVICE_RESULTS.md** at each checkpoint, initially using NOT TESTED for every item:

~~~text
Checkpoint: H1 / H2 / H3 / H4 / H5
Date/time (local):
Result: PASS / FAIL / BLOCKED / NOT TESTED
Kindle model, firmware, uname -m, libc:
KindleBT version/commit + library hash:
Chessnut protocol crate version:
Chessnut Move firmware (if known):
CLI/C-worker build receipt and ELF checks:
Bluetooth address: [REDACTED; identify only whether fixed/random]
BT privileges and session status:
Discovered UUIDs + write properties:
ATT MTU (measured / unknown), max tested write length, write kind:
Notifications and decoded position:
Auto-move source FEN -> target FEN (H4 only):
Physical behavior, STOP, final observed position (H4 only):
Timeouts/errors, redacted log path:
Independent main-app check:
Human signature / confirmation:
Follow-up action and next permitted milestone:
~~~

Do not mark a checkpoint PASS based solely on Codex claims or CI. Human operation of the physical Kindle and board is mandatory for H1–H5.

## 6. Reference links and constraints

- Project repo contract: ../README.md, ../AGENTS.md, KINDLE_SCRIBE.md, BUILD_DEPLOY.md, TESTING.md, device/ks1-barolo.md.
- KindleBT repository: https://github.com/Sighery/kindlebt
- KindleBT public API: https://github.com/Sighery/kindlebt/blob/main/include/kindlebt/kindlebt.h
- KindleBT C usage and privilege example: https://github.com/Sighery/kindlebt/blob/main/examples/basic_usage.c
- KindleBT documentation, releases and SDK toolchain: https://sighery.github.io/kindlebt/ and https://github.com/Sighery/kindle-sdk
- Chessnut Rust SDK: https://github.com/daymxn/chessnut-move-rs
- Chessnut protocol/API: https://github.com/chessnutech/chess_move_api and https://github.com/chessnutech/Chessnut_eBoards
- Swift corroboration of auto-move packet and BLE characteristics: https://github.com/NSStudent/EasyLinkSwiftSDK/blob/main/Documentation-en.md

**Codex working instruction:** Implement **one milestone** per request, run the relevant host tests, summarize changed files and concrete risks, and present the corresponding human checkpoint instructions verbatim. Do not proceed beyond a checkpoint unless the human reports real-device results. Preserve this as an isolated experiment until a later, separately approved integration plan.
