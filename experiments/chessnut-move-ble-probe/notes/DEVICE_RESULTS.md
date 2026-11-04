# Device results

M0 and M1 are Implemented. H1 preflight and human original-app verification
passed on 2026-11-04. The assistant ran SSH deployment/preflight at the user's
request; the user supplied the running Kindle and confirmed physical app behavior.
No Bluetooth session or board communication was attempted. H2–H5 are NOT TESTED.
M2 source a6005cb is deployed and passes Kindle preflight. H2 is blocked
pending the board’s physical BLE address; Mac-side discovery passed without writes.

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
Date/time (local): 2026-11-04; deployed preflight 21:37:55 CET
Result: NOT TESTED to completion — first Kindle connection/discovery PASS; reconnect and human acceptance pending
Kindle model, firmware, uname -m, libc: NOT TESTED
KindleBT version/commit + library hash: NOT TESTED
Chessnut protocol crate version: host pin 1.0.2; device NOT TESTED
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: a6005cb; source_sha256 6a5975b690ca3210a32bc94f06dc3829cf12eda5289fc76723653abc229ba7a5; deployed SHA256SUMS and Kindle preflight PASS; exit 0; lock cleared
Bluetooth address: [REDACTED]; public address supplied by user; cache UUID matched the Mac-discovered board
BT privileges and session status: verified worker identity; session open, BLE/GATT registration and first connection status 0; cleanup status 0
Discovered UUIDs + write properties: Kindle PASS, reverse 128-bit byte order; required service/characteristic pairs and 0x10 / 0x0c / 0x10 matched; 11 characteristics / six services; zero writes
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: first session exited 0 and cleared lock; BusyBox background stdin caused early EOF before queued reconnect commands; launcher correction pending retest
Independent main-app check: original binary plus 120 puzzle/game/state hashes unchanged after M2 deployment/preflight; no new visual app check
Human signature / confirmation: user confirmed board powered with blinking Bluetooth icon; H2 matching UUID/reconnect acceptance NOT CONFIRMED
Follow-up action and next permitted milestone: deploy launcher stdin correction, verify services/disconnect/connect/services/quit, obtain human H2 acceptance; no command writes
```

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
