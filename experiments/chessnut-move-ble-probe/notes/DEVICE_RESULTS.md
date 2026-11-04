# Device results

M0 host gate PASS on 2026-11-04. M1 installed on the Kindle; H1 preflight pending.
No probe launch, Bluetooth session or board communication has occurred.
Do not mark M1 Implemented until H1 passes.

## H1

```text
Checkpoint: H1
Date/time (local): 2026-11-04 20:51 CET deployment record; preflight NOT TESTED
Result: NOT TESTED
Kindle model, firmware, uname -m, libc: existing Scribe target; SSH reports firmware 5.19.6 and armv7l; libc preflight NOT TESTED
KindleBT version/commit + library hash: 59e1d8e69a52e3cd58cee6fcc416d1774ad55791; sha256 8b5d3bbe417831e9040b922d6db2969e0cfa9ecc5d8790f1af826bb85f96b485; load NOT TESTED
Chessnut protocol crate version: host pin 1.0.2; device NOT TESTED
Chessnut Move firmware (if known): NOT TESTED
CLI/C-worker build receipt and ELF checks: build dc8d7bc, source_sha256 b75b4995e309e18ab4fde1ab64c274241fe224a509de44d2f9ea1b0506a30ea2; host ELF PASS; installed SHA256SUMS PASS; launch NOT TESTED
Bluetooth address: [REDACTED; identify only whether fixed/random]
BT privileges and session status: NOT TESTED
Discovered UUIDs + write properties: NOT TESTED
ATT MTU (measured / unknown), max tested write length, write kind: unknown; NOT TESTED
Notifications and decoded position: NOT TESTED
Auto-move source FEN -> target FEN (H4 only): NOT TESTED
Physical behavior, STOP, final observed position (H4 only): NOT TESTED
Timeouts/errors, redacted log path: NOT TESTED
Independent main-app check: main binary plus 120 puzzle/game/state files retain identical before/after hashes; human launch NOT TESTED
Human signature / confirmation: NOT TESTED
Follow-up action and next permitted milestone: human H1 preflight and original-app launch; no live board work
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
