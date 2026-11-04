# Milestone 3 preparation — 2026-11-04

Status: **Host-ready and deployed; H3 BLOCKED before telemetry; M3 not yet Implemented.**
All changes remain in the isolated experiment on
experiment/chessnut-move-kindlebt-probe. Implementation preparation involved no device communication. Subsequently b6ca863
was deployed, passed preflight, then failed connection confirmation after the
system established a link. DEVICE_RESULTS.md records the daemon session/UUID
mapping failure and required stop; no telemetry or LED write ran.

## Scope and evidence

- Both position/response subscriptions use full UUID, parent-service, property
  and native-handle validation. SDK realtime enables streaming explicitly and
  requires an initial position. Battery and all tracked-piece queries use SDK
  constructors with one response deadline, no automatic retries.
- CLI telemetry, battery, pieces, mtu, watch and controlled one-square LED/clear
  commands print bounded raw and decoded observations. Position output is the
  FEN placement field only: sensor data has no turn/castling/en-passant/counters.
  A new operator command ends watch; quiet periods are nonfatal.
- Independent Rust and native packet allowlists permit only three safe query/
  realtime packets WithResponse and 34-byte LED packets WithoutResponse. Unknown
  and all 35-byte motor/STOP frames are rejected before native API dispatch.
- Notification callbacks validate connection, characteristic, blob format,
  offset and size, then copy at most 512 bytes into a 16-slot locked ring. Only
  the owner thread writes IPC. Invalid traffic/overflow is terminal.
- MTU evidence observes the exact pinned KindleBT bleMtuUpdatedCallback log.c
  arguments, checking callback name/format, connection, successful status and
  range 23..517. Optional observer/callback absence yields unknown. This private
  source contract does not add an MTU negotiation API or modify KindleBT.
- A 34-byte LED requires observed MTU >=37. API status, length, write kind and
  callback status are logged separately. Both write modes require callback
  completion; WithoutResponse completion is local stack evidence, not board ACK.
  No splitting, truncation or write-mode substitution is allowed.
- Cleanup tries fixed all-off before unsubscribing both acquired subscriptions
  and the existing database/disconnect/deregistration/session unwind. Callback
  timeout keeps payload memory alive, blocks subsequent commands, and permits
  only one separate best-effort all-off buffer; cleanup remains unsuccessful.
  No guessed realtime-disable command is sent; disconnect ends streaming.

## Red/green and verification

The first telemetry/session tests failed for missing M3 behavior. Portable C
parser/packet tests and pinned-header adapter tests initially failed for missing
native implementation. A later invalid-format callback test demonstrated that
blob format was unchecked; validation now rejects it before copying.

The independent fmt/clippy/Rust and Python gates cover safe SDK encodings,
bounded notifications, sensor placement, status replies, missing/malformed
initial position, battery timeout, callback failure, link loss, unknown/small
MTU LED refusal, invalid arguments and quiet/interruptible watch. Existing signal,
worker supervision, discovery, lifecycle and scoped deployment regressions run
unchanged. The real pinned-header native adapter additionally tests partial
subscription cleanup, forward/reversed UUID matching with correct parents,
payload copying, invalid callback size/format, MTU callback evidence, LED size
and write kind, API/callback errors, timeout-owned payload lifetime, refusal of
later writes, and final best-effort clear/unsubscription.

Final independent gate PASS: fmt/clippy, 36 Rust integration tests and 19
Python tests (including portable C contracts). ARM cross-build and real-header
adapter PASS; stage.sh and verify-stage.sh PASS. Parent fmt/clippy/workspace
tests PASS separately (all 237 tests) as the compatibility gate. Generated source-bound receipt, ELF checksums, build/test logs,
SDK sources, private identities and staged binaries stay ignored under target.
No parent application source/dependency or main branch is changed.

## Limits and next checkpoint

MTU >=38 establishes conventional capacity for a 35-byte ATT write; it cannot
prove full application payload delivery. **35-byte delivery is UNVERIFIED**
until a separately authorized H4 motor test. M3 sends no such packet, including
STOP. Notification reassembly, real callback availability/status, actual MTU,
physical LED operation and cleanup are not established by mocks or compilation.
LEDs are refused if MTU is unknown/insufficient; H3 must record that limitation
and stop for review. Official suggestions of MTU 500 are not a measured value.

Next: deploy the verified bundle, run preflight, then prepare the connection
before asking the user to press Bluetooth. During H3, compare a known physical
arrangement, move one piece by hand, inspect the changed placement, query battery
and pieces, review MTU and test one LED/clear when allowed. Verify disconnection,
LEDs off and normal stock behavior. Record redacted observations and human
acceptance in DEVICE_RESULTS.md. M4/M5 are untouched.
