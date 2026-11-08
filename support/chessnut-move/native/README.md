# Native Kindle transport reference

These files are copied from the verified probe without behavioral changes. Keep
all headers together: `kindlebt_worker.c` includes the small session, identity,
notification/write, diagnostics and parser boundaries. The worker compiles only
for the pinned Kindle environment (32-bit ARM); it is not a desktop BlueZ backend.

The pinned KindleBT translation unit must be generated with
`scripts/build_tools.py::descriptor_bridge_source` before building its bundled
library. That adds `kindlebt_descriptor_bridge.h` and exactly one missing callback
forwarding table entry. Link against the device's existing `libace_bt.so`; build
references and exact hashes are in `BUILD_PINS.json` and `SOURCE_PINS.json`.
Do not distribute Amazon SDK/header/library payloads in this repository.

Exact dynamic imports are declared using actual pinned header types. MTU negotiation
uses the verified exported two-argument `aceBT_bleRequestMtu`; callbacks are matched
to the active connection. Embedded and counted-list CCCDs are handled, with one
outstanding descriptor/write and matched completion. Borrowed write storage stays
alive through timeout, and cleanup STOP uses separate persistent storage.

The native parser accepts safe SDK telemetry/one-square LED packets, fixed STOP,
and a one-shot explicitly armed Normal 35-byte packet. The future app must enforce
its own source/target validation and authorization. Native acceptance and successful
callbacks do not prove physical movement completion or a physical stop.

Use the full archived probe build recipe for cross-link details; the support
package preserves only non-deployment primitives. The actual-header host contract
test is available via `scripts/check-native-adapter.sh`; portable native contracts
run as part of `scripts/check.sh`. Neither check connects to a board.

The production send-and-forget workflow (2026-11-06) allows a new explicit Arm
after a previous Normal write callback, without STOP or fresh telemetry. Arm
remains one-shot and writes remain serialized. motion_maybe is retained only as a
conservative cleanup flag; transport callbacks do not prove motion completion.

Task 57 additionally permits the exact realtime query (21 01 00) while
motion_maybe, so the listener can re-enable reports after Normal without STOP.
Other non-motor probes remain blocked during possible motion; no completion is
inferred, and conservative exit cleanup is retained. Physical verification of
report resumption remains required.
