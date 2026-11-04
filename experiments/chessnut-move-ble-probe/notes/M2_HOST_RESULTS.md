# Milestone 2 preparation — 2026-11-04

Status: **Host-ready; awaiting deployment and H2. Not Implemented.**
H1 remains accepted for the installed M1 source build 138cc25. No M2 SSH,
deployment, Bluetooth session or board operation has been performed. All changes
remain under this experiment on experiment/chessnut-move-kindlebt-probe.
M3–M5 remain untouched.

## Scope and implementation

- Discovery-only interactive session: supplied private address, services,
  disconnect, explicit connect using a fresh worker, quit and graceful EOF.
  Every board-writing or telemetry command is UNSUPPORTED and sends no IPC request.
- Native worker repeats verified M1 identity/library checks before session APIs;
  only the worker changes UID/GID. It opens a session, registers BLE/GATT,
  connects to exactly the supplied address, discovers and clones the database.
- Exact pinned headers define callbacks and types. Callback data stays borrowed;
  owned clones are freed. Partial setup unwinds acquired resources in order.
  Cleanup errors remain errors; forced cleanup cannot acknowledge success.
- Bounded versioned canonical JSON subset, one outstanding request, stage/numeric
  errors, cancellation and idle disconnect reporting. No automatic retries,
  scanning, address fallback, subscription or write API dispatch exists.
- GATT snapshots retain parent services, full UUIDs, handles and raw properties.
  Consistent forward/reverse 128-bit byte ordering is detected using all required
  service/characteristic pairs. Ambiguous/missing records, wrong parent services,
  missing properties and malformed/oversized databases abort the session.
- Raw vendor stdout/stderr are suppressed while executing session APIs to avoid
  protocol pollution and MAC disclosure; our stderr reports timestamps/stages
  and numeric status only. Captures remain private and ignored.

## Red-to-green and automated gates

Discovery tests first failed because the module was missing; native lifecycle
fixtures failed for the missing production header; interactive tests failed
for the missing session controller. Implementations made these green.
A later test showed forced cleanup incorrectly acknowledged success; the client
now reports cleanup_failed_or_forced. Another demonstrated lost startup stage
codes from ID-0 errors; those diagnostics are now retained.

Final gates: independent fmt/clippy, 28 Rust integration tests, 17 Python tests,
including compiled portable C identity and lifecycle/parser fixtures. The ARM
build also compiles/runs a host adapter contract against the real pinned headers,
then verifies ARMv7 hard-float PIE executables, zero COPY relocations, the loader,
GLIBC <=2.35 and local runtime imports. Stage/manifest checks exclude mocks,
private logs, host test executables and vendor libraries. Parent fmt/clippy and
all 237 workspace tests passed separately. The ARM build and native header
adapter contracts passed. No parent application source or dependency changed.

## Hardware limits and next checkpoint

No native session compatibility, real byte order, actual characteristic
properties, physical link release/reconnect or stock-Bluetooth coexistence is
claimed from host tests. Pinned KindleBT does not forward connection-state events
to application callbacks; the adapter observes its exported internal context
under its mutex, requiring H2 validation. A failed connect may leave no public
handle to disconnect; acquired session/registrations are cleaned up and actual
link release must be verified on hardware. A vendor call may outlast its normal
10-second callback wait: requests time out after 45 seconds, cleanup is allowed
55 seconds, then forced reaping is reported as failure.

The next boundary is deploying the verified bundle and performing H2 with the
human's current board address. The board must be in BLE mode and the official
app/scanner disconnected. Stop for any permission/discovery/UUID mismatch.
No board command, LED, telemetry or motor testing belongs to M2.

## Completion checklist

- [x] Session, BLE/GATT registration and address-based connect/disconnect.
- [x] Discovery, database ownership and required UUID/property validation.
- [x] Callback handling, bounded IPC, cancellation, timeouts and partial cleanup.
- [x] Interactive services/connection diagnostics with writes disabled.
- [x] Host regression gates and ARM cross-build/ELF gates.
- [ ] Deploy M2 to the Kindle and repeat preflight.
- [ ] Human H2: board connection, matching UUIDs/properties, clean reconnect.

M2 remains host-ready until the remaining hardware checks pass. The assistant
stops before deployment or board communication, as requested. The generated
build receipt and checksummed local stage identify the prepared payload;
neither generated artifacts nor raw logs are committed.
