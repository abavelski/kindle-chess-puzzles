# Device results

M0 and M1 are Implemented. H1 preflight and human original-app verification
passed on 2026-11-04. The assistant ran SSH deployment/preflight at the user's
request; the user supplied the running Kindle and confirmed physical app behavior.
No Bluetooth session or board communication was attempted. H2–H5 are NOT TESTED.

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
