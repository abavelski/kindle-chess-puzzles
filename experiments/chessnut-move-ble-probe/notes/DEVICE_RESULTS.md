# Device results

M0–M2 are Implemented. H1 and H2 passed with human acceptance on 2026-11-04.
The assistant ran SSH deployment and probe commands at the user’s request; the
user supplied the Kindle, board address and fresh BLE-button activation, and
confirmed normal behavior. H3–H5 remain NOT TESTED. No board commands were sent.

## H1

```text
Checkpoint: H1
Date/time (local): 2026-11-04 21:08:45 CET preflight; subsequent human confirmation
Result: PASS
Kindle model, firmware, uname -m, libc: first-generation Scribe; 5.19.6; armv7l; kernel 4.9.77-lab126; glibc 2.35
Loader: /lib/ld-linux-armhf.so.3; direct dependencies and actual transitive loading PASS
KindleBT version/commit + library hash: 59e1d8e69a52e3cd58cee6fcc416d1774ad55791; sha256 8b5d3bbe417831e9040b922d6db2969e0cfa9ecc5d8790f1af826bb85f96b485; load and required symbols PASS
Chessnut protocol crate version: 1.0.2; controller reports 1.0.2; no live protocol exchange
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: source commit 138cc25; source_sha256 37feec3a8bbd19e51d94a97262713f792af63fa41dfb82a82bb2d8175dc5c986; host ELF PASS; installed SHA256SUMS PASS
Worker SHA256: 804cf2f81dbc64b65df4a9fd79c9f5490333b39cffbbf231ed0042759644f542; PIE; zero COPY relocations
Bluetooth address: not supplied or used
BT privileges and session status: named UID/GID 1003 verified against all btmanagerd kernel identity fields; passwd primary GID 150 reported; worker group clearing and privilege drop PASS; session NOT_OPENED; connection NOT_ATTEMPTED
Discovered UUIDs + write properties: NOT TESTED
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: result=PASS; preflight_exit_status=0; operation_lock=cleared; private raw logs retained only in ignored target directory
Independent main-app check: original binary and 120 puzzle/game/state files retain identical before/after hashes; human display, response and exit check PASS
Human signature / confirmation: user replied "Yes, it works normally" to original-app display/response/exit verification
Follow-up action and next permitted milestone: M1 complete; stop here. M2 is the next implementation milestone and has not been started; no board tests performed.
```

## H2

```text
Checkpoint: H2
Date/time (local): 2026-11-04 22:04:41–22:04:43 CET; subsequent human acceptance
Result: PASS
Kindle model, firmware, uname -m, libc: first-generation Scribe; 5.19.6; armv7l; kernel 4.9.77-lab126; glibc 2.35
KindleBT version/commit + library hash: 59e1d8e69a52e3cd58cee6fcc416d1774ad55791; sha256 8b5d3bbe417831e9040b922d6db2969e0cfa9ecc5d8790f1af826bb85f96b485
Chessnut protocol crate version: 1.0.2; no command protocol exchange in M2
Chessnut Move firmware (if known): NOT QUERIED
CLI/C-worker build receipt and ELF checks: installed source f19ecfb; source_sha256 7ae950d1cd7c12eda9619082ca3ea82584616b08c7f5cdff6bcdd63db12c626c; host ELF PASS; installed SHA256SUMS PASS; preflight PASS
Bluetooth address: [REDACTED]; public address supplied by user; cache UUID matched the Mac-discovered board
BT privileges and session status: worker UID/GID verified; session/BLE/GATT/connect returned 0 on two fresh workers; both disconnected, deregistered and closed with status 0
Discovered UUIDs + write properties: PASS; six services / 11 characteristics; consistent reverse 128-bit byte order; required parent services and all three characteristics matched; position Notify=0x10, command write=0x0c, response Notify=0x10
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED; advertised write properties are not proof of payload delivery
Notifications and decoded position: NOT TESTED; no subscription
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED; zero command writes
Timeouts/errors, redacted log path: earlier isolated status -2 connection timeout with icon subsequently off; final timed test PASS, two connects/four validations/two cleanups, session_exit_status=0, operation_lock=cleared; raw logs private under ignored target
Independent main-app check: original binary plus 120 puzzle/game/state files have unchanged hashes after final H2; no probe processes remain; H1 visual acceptance retained, no additional visual check claimed
Human signature / confirmation: user pressed Bluetooth button when listener/Kindle ready and replied "Yes, normal behavior; H2 accepted"
Follow-up action and next permitted milestone: M2 Implemented. Stop here; M3 is next and has not started.
```

Required UUIDs after normalization:

| Parent service | Characteristic | Properties |
| --- | --- | --- |
| 1b7e8261-2877-41c3-b46e-cf057c562023 | 1b7e8262-2877-41c3-b46e-cf057c562023 | 0x10 Notify |
| 1b7e8271-2877-41c3-b46e-cf057c562023 | 1b7e8272-2877-41c3-b46e-cf057c562023 | 0x0c Write / WithoutResponse |
| 1b7e8271-2877-41c3-b46e-cf057c562023 | 1b7e8273-2877-41c3-b46e-cf057c562023 | 0x10 Notify |

## H3

```text
Checkpoint: H3
Date/time (local): NOT TESTED
Result: NOT TESTED
Kindle model, firmware, uname -m, libc: NOT TESTED
KindleBT version/commit + library hash: NOT TESTED
Chessnut protocol crate version: host pin 1.0.2; device NOT TESTED
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: NOT TESTED
Bluetooth address: [REDACTED; identify only whether fixed/random]
BT privileges and session status: NOT TESTED
Discovered UUIDs + write properties: NOT TESTED
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: NOT TESTED
Independent main-app check: NOT TESTED
Human signature / confirmation: NOT TESTED
Follow-up action and next permitted milestone: human deployment and H1 preflight; no live board work
```

## H4

```text
Checkpoint: H4
Date/time (local): NOT TESTED
Result: NOT TESTED
Kindle model, firmware, uname -m, libc: NOT TESTED
KindleBT version/commit + library hash: NOT TESTED
Chessnut protocol crate version: host pin 1.0.2; device NOT TESTED
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: NOT TESTED
Bluetooth address: [REDACTED; identify only whether fixed/random]
BT privileges and session status: NOT TESTED
Discovered UUIDs + write properties: NOT TESTED
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: NOT TESTED
Independent main-app check: NOT TESTED
Human signature / confirmation: NOT TESTED
Follow-up action and next permitted milestone: human deployment and H1 preflight; no live board work
```

## H5

```text
Checkpoint: H5
Date/time (local): NOT TESTED
Result: NOT TESTED
Kindle model, firmware, uname -m, libc: NOT TESTED
KindleBT version/commit + library hash: NOT TESTED
Chessnut protocol crate version: host pin 1.0.2; device NOT TESTED
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: NOT TESTED
Bluetooth address: [REDACTED; identify only whether fixed/random]
BT privileges and session status: NOT TESTED
Discovered UUIDs + write properties: NOT TESTED
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: NOT TESTED
Independent main-app check: NOT TESTED
Human signature / confirmation: NOT TESTED
Follow-up action and next permitted milestone: human deployment and H1 preflight; no live board work
```

## Deployment observations — 2026-11-04

Installed source commit dc8d7bc through the scoped SSH/scp workflow. Named
payload entries are required by this Kindle's legacy SCP server: `/.` is
rejected. Remote SHA256SUMS verification, initial directory-swap installation,
and upload cleanup succeeded. No operation lock remained. The launcher was not
run. The original chess binary and all 120 files under puzzles/games/state
were hashed before and after and remained identical. Private addresses and raw
logs remain in ignored target files, outside this record. H1 remains NOT TESTED
until the human operates preflight and verifies the original app launch.

## H1 preflight failure — 2026-11-04

At the user's request, executed the installed launcher over SSH. Every runtime
SHA256SUMS entry passed. The Rust controller printed its M1 preflight timestamp,
version 0.1.0 and protocol version 1.0.2. Its native worker then terminated with
SIGSEGV before emitting architecture, libc, firmware, privilege or library-load
reports. The overall exit status was 1. No pass is claimed for those checks.

Read-only follow-up confirmed the launcher removed .operation-lock and all
121 original chess binary/data hashes still matched the pre-deployment snapshot.
Stopped at the plan's H1 failure gate; no system service, library, configuration,
or board action was changed. The native startup crash needs investigation
before H1 can pass. Human original-app visual acceptance remains unverified.

## Offline follow-up — 2026-11-04

The failed worker has a COPY relocation and incorrectly co-located libc globals
(stdout/stderr/environment). A -fPIC -pie rebuild removes the defect; new host
regressions reject both non-PIE executables and ARM COPY relocations. See
M1_STARTUP_CRASH.md for evidence and limits. The user stopped the Kindle, so
no SSH connection or corrected-candidate deployment was attempted. H1 stays
FAIL pending deployment/preflight and human original-app acceptance.

## Corrected deployment and H1 acceptance — 2026-11-04

The earlier failure and offline records above are historical. After the user
made the Kindle available again, the assistant deployed and tested the fixes
in 3851829, 9429c3e and finally 138cc25. The worker now starts successfully,
validates actual Bluetooth process identities and loads the current firmware's
dependencies rather than requiring the old SDK's transitive filenames.
All installed checksums passed; final preflight exited 0 and cleared its lock.
No services, system libraries, radio settings or original-app files were changed.
The user confirmed normal original-app operation, and a subsequent checksum
comparison found all 121 original binary/data hashes unchanged. H1 is accepted.
The deployed receipt identifies 138cc25; the subsequent acceptance commit only
updates documentation. No additional device communication is needed for M1.

## M2 host preparation — 2026-11-04

Discovery-only session implementation and automated tests are ready for H2.
No M2 device deployment or board connection has been attempted. The installed
M1 build and accepted H1 record above remain unchanged. M2 is not Implemented
until the human H2 checks pass; see M2_HOST_RESULTS.md and README.md. The next
boundary requires the Kindle plus the Move in BLE mode, with its official app
and scanner disconnected, and a current address supplied privately.

## M2 deployment and Mac discovery — 2026-11-04

At the user’s request, deployed source a6005cb and ran Kindle preflight over SSH.
Installed checksums passed, preflight exited 0, and the operation lock cleared.
Architecture/firmware/libc, daemon identity, worker privilege transition and all
library/symbol checks passed. All 121 original app/data hashes match the fresh
pre-deployment baseline. No Kindle Bluetooth session has been opened.

An isolated Mac diagnostic connected once to the sole advertised Chessnut Move,
discovered its GATT database, then disconnected. The required services and
characteristics match the pinned SDK exactly, with Notify / both write modes /
Notify properties respectively. Four services/eight characteristics were observed.
No read, notification subscription, descriptor write or command write was sent.
A later scan found no board and one explicit reconnect attempt timed out; further
board connections stopped. The board must be confirmed awake in BLE mode again.

The physical BLE address is unavailable through the CoreBluetooth UUID alone.
Direct and administrator-helper reads of the macOS Bluetooth cache were denied.
The user reported enabling Full Disk Access; logs identify Visual Studio Code
as the shell’s responsible app and Apple authtrampoline as the privileged reader.
No ACL, ownership or service changes were made. Manual Terminal export or another
scanner is needed to supply a current address. H2 stays BLOCKED; Mac discovery
alone does not establish Kindle session compatibility or a clean reconnect.

## First Kindle connection — 2026-11-04

The user supplied a public board address from the Mac cache, whose peripheral
UUID matches the previously discovered board, then confirmed Kindle and Move
ready with the official app closed. The installed a6005cb worker connected and
discovered the correct required UUIDs/properties, using reverse 128-bit ordering.
Every setup/discovery and cleanup stage returned status 0; session exit was 0
and the operation lock cleared. No notification subscription or board write ran.
The controller saw early EOF from BusyBox ash’s background-input behavior, so
queued reconnect commands were not executed. A separate read-only shell probe
confirmed the old redirect loses input and a saved descriptor preserves it.
The host-tested launcher correction needs deployment and retest before H2 can
pass. Raw address and logs remain private and uncommitted.

## H2 reconnect and acceptance — 2026-11-04

After the launcher input correction, deployed f19ecfb and verified installed
receipt/checksums and preflight. One attempt timed out with status -2 before a
connection handle was returned; all acquired registration/session cleanup calls
returned 0, exit was 1 and the operation lock cleared. The user reported the icon
was off and that the board exits BLE availability quickly. No precise window or
definitive cause of the timeout was established, and no automatic retry occurred.

Prepared one new attempt after reporting the failure: SSH waited for a start
marker while a temporary Mac listener watched only the already-identified board’s
advertisement UUID. The user pressed the Bluetooth button when both were ready.
The listener stopped scanning on a match, without connecting, and triggered the
prepared Kindle command. At 22:04:41–22:04:43 CET, connect/discover/services,
disconnect, explicit reconnect/discover/services, then quit all succeeded.
Two connections, four successful validations and two cleanups were recorded;
every native stage returned status 0, the final session exit was 0 and its lock
cleared. This checkpoint helper is ignored diagnostic code; no runtime scanning
or automatic retry feature was added.

The Kindle returned six services and 11 characteristics, with 128-bit UUID bytes
reversed relative to canonical text. All three required characteristics belonged
to their expected services and advertised the expected properties. No telemetry,
notification subscription, characteristic read, LED, STOP or motor command ran.
The final process check found no probe left running, and all 121 original app/data
hashes remained unchanged. The user explicitly confirmed normal behavior and
accepted H2. M2 is Implemented. H3–H5 remain NOT TESTED; stop before M3.


## M3 preparation — 2026-11-04

Checkpoint: H3 — **NOT TESTED**. Host implementation is ready; no deployment,
SSH, Mac BLE scan/connection or board command was performed during M3 preparation.
The installed source remains f19ecfb (M2). Notification delivery/lengths, physical
placement and hand-move comparison, battery/piece status, actual MTU, write
callbacks, LED on/off and cleanup, stock Bluetooth interference and human
acceptance are all NOT TESTED for M3. 35-byte delivery remains UNVERIFIED and all
motor/STOP packets are blocked. The next boundary is deployment and preflight,
then a human-timed board connection and H3. See M3_HOST_RESULTS.md for the host
evidence and README for the exact checkpoint and CLI commands. Do not mark M3
Implemented before actual H3 results and human acceptance are recorded.


## M3 deployment and stopped connection attempt — 2026-11-04

Installed source b6ca863, source_sha256
b0b38215dcce4a5cd2a0c36d1467fe7cd7989f49b2f5e6eb42147b42ab0330bb.
Deployment/runtime checksums, installed receipt equality and preflight PASS;
exit 0 and operation lock cleared. All 121 original app/data hashes match the
fresh pre-deployment baseline. No stock services or Bluetooth configuration changed.

After the user enabled BLE, an advertisement-triggered, single Kindle attempt
began at 22:31:28 CET. The Mac listener stopped scanning without connecting.
Session/BLE/GATT registration returned 0, but connect returned -2 after ten
seconds. No command subscription or board write ran (write_api_calls=0), and no
MTU value was obtained. GATT/BLE/session cleanup APIs returned 0; overall session
exit was 1, lock cleared and a read-only process check found no probe worker or
controller. There was no automatic retry.

The user reported a steady/on board icon after the timeout. Read-only Kindle
system logs show controller link and GATT connection success, followed by
get_client_session_from_gatt_uuid / gatt_client_conn_ind failing to find the
client session for the connection UUID. Thus this attempt reached a physical
link but failed to deliver connection confirmation to the probe; it cannot be
attributed simply to a missed advertising window. Root cause of the daemon's
session/UUID mapping mismatch is not yet established. Session close attempted
native disconnection, but physical link release is not proved by API return
codes. No daemon restart, cache deletion, radio toggle, alternate address or
unverified ABI change was attempted. Raw system logs remain private/ignored.

H3 is **BLOCKED before telemetry**. Requested that the human power the board off
using its normal control to release a possible lingering link, then confirm its
icon is off. Further connections are stopped. Next work is offline investigation
of the connection/session mapping evidence and a reviewed recovery/retest plan;
no telemetry/LED test or later milestone may proceed on this failed connection.


The user subsequently confirmed powering the board off and back on; its icon
was blinking again. Physical power-cycle confirmation is recorded, and no
further icon-off confirmation is needed. No new connection was attempted.
Read-only SSH copied the device's libace_bt privately under ignored target for
offline inspection; no firmware binary or proprietary code is committed.
Inspection of the device's connection and GATT-registration entry points is
consistent with the argument placement used by pinned KindleBT; this limited
check does not establish complete ABI compatibility or explain the daemon's
session/UUID mapping failure. No speculative ABI change is justified yet.


## Reviewed discovery-only reference test plan — 2026-11-04

Offline comparison confirms M3's libkindlebt.so is byte-identical to the accepted
H2 library (SHA256 8b5d3bbe417831e9040b922d6db2969e0cfa9ecc5d8790f1af826bb85f96b485).
Session opening, BLE/GATT registration and connection arguments also match the
accepted M2 source. These checks do not exclude a worker regression or prove
stale daemon state. Neither possibility justifies changing stock services yet.

Prepared the exact native sources from f19ecfb as a diagnostic reference using
the same checked headers, compiler flags, Zig and ARM/glibc target. ARM/PIE/loader/
COPY-relocation/GLIBC inspection passed. Reference worker SHA256:
7c1248212615483d96c6a695d6e1e52283b6390bebbfc2043d4a657d7019516a.
It uses the deployed M3 Rust supervisor for its compatible discovery-only IPC,
and the byte-identical accepted KindleBT library. The reference sources, bundle,
receipt and helpers are ignored host artifacts. Its fixed noninteractive command
sequence is connect/discover -> services -> quit, with no subscriptions, status
queries, LEDs, MTU request or motor packets; it cannot accept M3 write operations.

Installed only in /mnt/us/chessnut-ble-probe/diagnostics/m2-reference-f19ecfb,
with a local checksum manifest and the existing owner-marker/operation-lock,
input-preservation and trap behavior. The M3 runtime was not replaced. Reference
preflight/checksums PASS and the lock cleared. An initial scp directory spelling
was rejected; upload used explicit regular filenames instead and both uniquely
owned temporary upload directories were removed. No stock file/service changed.

Next: prepare SSH and the advertisement trigger, request a fresh human button
press, run exactly one reference connection, and inspect success or failure with
cleanup diagnostics. STOP on failure and record the system evidence before any
retest. If the reference also fails, that is evidence of a problem shared with
M2/the system stack, not proof of its precise cause. If it passes, investigate M3
worker differences before telemetry. A stock radio/service restart or reboot is
not authorized as an automatic workaround by this diagnostic plan.


## M2 reference result and stale-entry evidence — 2026-11-04

At 22:41:28 CET the advertisement-triggered M2-reference attempt opened the
session and registered BLE/GATT successfully, then connect timed out with -2
at 22:41:38. Discovery did not run, native writes=0, cleanup API statuses were 0,
final SSH/session exit was 1, the lock cleared and a read-only /proc check found
no probe process. The user reported the light becoming steady immediately.
The listener made no Mac connection. No further connection was attempted.

System evidence narrows the failure beyond a generic callback/ABI issue. During
the earlier M3 attempt, client interface 5 was registered with session UUID A and
connection-list node X, but its successful connection event was routed to an
older UUID. During the M2 reference, the same interface number was registered
with new UUID B and new node Y; getConnectionByClientIfc returned old node X,
and the callback lookup used UUID A, not B. The old UUID no longer had a client
session. Raw UUIDs and pointer values stay in private ignored logs. This observed
stale connection-list lookup explains the missing probe confirmation despite
the established physical BLE link; it does not establish why the older entry
survived or prove there are no other compatibility problems.

The failure occurs with both native workers and the accepted byte-identical
KindleBT library. Board power cycling alone did not resolve it. Next recovery
is a human-performed normal Kindle restart to reset the daemon's in-memory
state, with the board off and no probe process/lock active. No shell reboot,
Bluetooth daemon restart, cache removal, global configuration or firmware edit
will be performed. After the user restores SSH, repeat preflight and prepare
one fresh discovery test before asking for board BLE activation. Recovery and
post-restart behavior remain NOT TESTED; H3 stays BLOCKED and motor packets
remain prohibited. If discovery succeeds, verify disconnect/cleanup before
continuing the M3 telemetry/LED checkpoint.


## Human Kindle restart and renewed preflight — 2026-11-04

The user restarted the Kindle normally and then started SSH. The initial SSH
check was refused on ports 22/2222; after the launcher was started, port 2222
accepted the existing authentication. Uptime was approximately 625 seconds and
btmanagerd had a new PID (4260 rather than 2205), confirming a fresh daemon
process. Read-only checks found no probe process or operation lock. Installed
M3 source b6ca863 still matches the verified receipt, and preflight/privilege/
loader/dependency checks PASS with exit 0 and lock cleared. No board connection
or command ran during this check. Stale-entry recovery remains unverified until
the next single, human-triggered connection/discovery attempt.


## Post-restart connection failure differs — 2026-11-04

At 22:59:39 CET one advertisement-triggered M3 connection attempt started on the
fresh daemon (PID 4260). Public connect again returned -2 after ten seconds;
no discovery, subscription or board write ran (write_api_calls=0). Cleanup APIs
returned 0, final exit was 1, lock cleared and the subsequent read-only process
check found no probe process. No automatic retry occurred.

The current system log had rotated; read-only retrieval of existing compressed
logs preserved the 22:59 evidence privately. Unlike the previous attempts, both
client registration and connection lookup used the same fresh session UUID and
node. The earlier stale-node routing failure did not recur in this attempt.
The controller briefly reported a link, followed at 22:59:40 by GATT disconnect
reason 62 (0x3e, failed establishment in the pinned vendor headers), xStatus 133
and BLE connection failure status 10. No MTU evidence was obtained. This is a
new establishment failure; the restart did not restore successful discovery,
and no specific cause (range, interference, peer state or stack issue) is proved.

A read-only CoreBluetooth retrieveConnectedPeripherals query for the board's
two known services returned zero matches for the known board UUID. It requested
no scan, connection, read, subscription or write. This does not prove absence
of every other possible BLE client; it found no current Mac connection.

H3 remains BLOCKED. Next reviewed diagnostic: a single Mac-only, advertisement-
triggered connection to the exact already-identified board, bounded service/
characteristic discovery, then disconnection, with no subscription or write.
The Kindle must remain idle throughout. The helper selects only the private
known UUID, stops scanning before connecting, logs RSSI and waits for callbacks;
timeout/failure is terminal. This cross-check tests current board availability
on another host rather than blindly repeating the failed Kindle attempt. No
further board button press or connection has occurred yet; human interaction
is needed once the listener is ready. No firmware/service changes are proposed.


## Mac-only current-board cross-check — 2026-11-04

The user enabled BLE after the Mac-only listener was ready. The exact private
known peripheral advertised at RSSI -56 dBm as measured at the Mac. The listener
stopped scanning and connected once, discovered four services/eight
characteristics, validated both required services and three characteristic
UUIDs with Notify / both write modes / Notify properties (16 / 12 / 16), and
received clean disconnection with no error. Diagnostic exit was 0. No read,
subscription, descriptor write or board command ran (writes=0); the Kindle
remained idle throughout. Private raw identities/logs remain ignored.

This demonstrates that the board currently accepts Mac BLE/GATT discovery and
releases the link. It does not prove Kindle radio proximity, connection quality,
MTU or the cause of the Kindle's 0x3e establishment failure. Requested confirmation
that the Kindle is awake beside the board before considering another targeted
Kindle test. No automatic retry or stock-Bluetooth configuration change occurred.
H3 remains BLOCKED before telemetry; Mac discovery is not an H3 pass.


The human confirmed the Kindle was already awake beside the board on the same
table. Thus sleep/proximity cannot be assumed to explain the failed attempt.
Read-only inspection of archived client history found additional client-interface
callbacks, but did not establish a competing application connection; no stock
process was stopped. The probe's session type, registration order, connection
parameters, role and priority match the pinned upstream basic_usage.c example.
Upstream documentation and issue search yielded no verified fix for this exact
failure; no dependency pin or API arguments were changed on that basis.

Physical tests are stopped. No further button press, power cycle or unchanged
connection retry is requested. The next engineering investigation must identify
a supported connection-path correction or establish lower-level controller
failure evidence, with a concrete hypothesis before another hardware test.
Any diagnostic requiring global logging/radio/service/firmware changes falls
outside the current isolation rules and must be reviewed separately. Mac
success, close placement and a restarted Kindle do not make H3 PASS: real
Kindle telemetry, status, MTU, LED on/off and human acceptance remain untested.


## Connection diagnostics prepared and deployed — 2026-11-04

At the user's explicit request to debug, added a bounded exact-pinned connection
callback observer and read-only KindleBT context snapshot. Neutral connect error
labels replace the unjustified presumed-address diagnosis. This is diagnostic
instrumentation, not a claimed BLE fix; API calls, arguments, retry policy and
cleanup behavior are unchanged. See CONNECTION_DEBUG.md for red/green evidence.
Source b4ce485 passes all host/native/ARM/stage gates and is installed, with
source_sha256 c94bb0a216b6dd93e69530ebbf67f088dc4dca0aae8ed80a10278fda325fa23b.
Installed checksums/receipt and preflight PASS, exit 0 and lock cleared; all 121
original app/data hashes match the fresh deployment baseline. No connection,
subscription or board write occurred during this preparation. Next hardware
boundary: one coordinated discovery-only attempt with numeric callback/context
evidence and private read-only system-log capture, then stop on failure.
