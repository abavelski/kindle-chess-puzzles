# Milestone 2 preparation — 2026-11-04

Status: **M2 Implemented; H2 PASS**, accepted by the user on 2026-11-04.
Installed source `f19ecfb` passed preflight and the complete Kindle discovery/
disconnect/reconnect sequence. The original app binary and 120 data files retain
their before/after hashes. All work remains under this experiment on
experiment/chessnut-move-kindlebt-probe. M3–M5 remain unstarted. Earlier preparation,
blocked and failed-attempt sections below are historical; final evidence is last.

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

Final gates: independent fmt/clippy, 28 Rust integration tests, 18 Python tests,
including compiled portable C identity and lifecycle/parser fixtures. The ARM
build also compiles/runs a host adapter contract against the real pinned headers,
then verifies ARMv7 hard-float PIE executables, zero COPY relocations, the loader,
GLIBC <=2.35 and local runtime imports. Stage/manifest checks exclude mocks,
private logs, host test executables and vendor libraries. Parent fmt/clippy and
all 237 workspace tests passed separately. The ARM build and native header
adapter contracts passed. No parent application source or dependency changed.

## Initial hardware limits and next checkpoint (historical)

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
- [x] Deploy M2 to the Kindle and repeat preflight.
- [x] Human H2: board connection, matching UUIDs/properties, clean reconnect.

M2 is complete with human H2 acceptance. The user authorized Mac Bluetooth
discovery, Kindle deployment and the final button-timed test; see DEVICE_RESULTS.md. The generated
build receipt and checksummed local stage identify the prepared payload;
neither generated artifacts nor raw logs are committed.

## Hardware preparation follow-up — 2026-11-04

The user supplied a powered Move with a blinking Bluetooth icon and authorized
Mac Bluetooth access. A temporary, ignored Swift/CoreBluetooth diagnostic found
one Chessnut Move, connected, discovered four services/eight characteristics,
and disconnected without any characteristic read, subscription or write.
The required position and response characteristics advertise Notify (0x10),
and command characteristic advertises both write modes (0x0c). Mac-side
connect/discovery PASS is corroborating evidence, not acceptance of Kindle H2.
A later Mac scan found no candidate and a single explicit reconnect timed out;
further board attempts stopped pending physical BLE-mode confirmation.

The verified a6005cb bundle deployed successfully, passed all installed checksums
and Kindle preflight, exited 0 and cleared its lock. All 121 original chess
binary/data hashes remain unchanged. No Kindle Bluetooth session was opened.
The missing physical BLE address blocks it: CoreBluetooth supplies a UUID,
not a usable six-octet address. macOS denied direct and privileged cache reads,
even after the user reported enabling Full Disk Access for Codex/osascript.
Permission logs attribute the shell to Visual Studio Code and the privileged
reader to Apple authtrampoline. No file permissions, system services or Bluetooth
configuration were changed to bypass this. The next proposed step is a manual
read-only cache export from Terminal with its own Full Disk Access, or a current
address supplied by another scanner. Private identities/raw logs remain ignored.

## First Kindle H2 attempt and BusyBox input correction — 2026-11-04

The user supplied the public BLE address from a macOS OtherDevices row; its
peripheral UUID matches the sole board previously discovered by the Mac.
After waking the Kindle and confirming both devices ready, the assistant ran
the installed a6005cb discovery-only session. Session open, BLE/GATT registration,
connect, discovery and database retrieval all returned status 0. All required
UUIDs and properties matched with consistent reverse 128-bit byte ordering.
The Kindle returned 11 characteristics across six services, including standard
16-bit Generic Access/Attribute entries omitted by the Mac’s high-level listing.
The database, link, registrations and session cleaned up with status 0, the
launcher exited 0 and its lock cleared. No board command was sent.

The intended services/disconnect/connect/services/quit input was not consumed:
the controller saw immediate EOF. A harmless two-case shell probe on the real
Scribe reproduced BusyBox ash replacing a background job’s fd 0 before <&0
could preserve it. The original form read an empty line; saving fd 0 to fd 3
before spawning and restoring from fd 3 read the supplied marker correctly.
The launcher now does that and closes the extra descriptor in child and parent.
A new host contract first failed for the missing saved descriptor, then passed;
the existing host argument/input/lock test also passes. The independent Python
suite now has 18 tests. This fixes the shell boundary without changing native
BLE or Rust behavior. A corrected build/deployment and real reconnect retest
are required before H2 acceptance; no later milestone work is started.

## Final H2 acceptance — 2026-11-04

Corrected source f19ecfb passed the independent gate (28 Rust integration tests,
18 Python tests), native adapter contracts against the real pinned headers,
ARM build, ELF/GLIBC inspection, stage verification and deployment. Installed
source_sha256 is 7ae950d1cd7c12eda9619082ca3ea82584616b08c7f5cdff6bcdd63db12c626c.
The three native artifacts are unchanged from a6005cb; only the launcher,
regression and supporting documentation changed. Repeated preflight passed.

An initial corrected-build connection timed out after the library’s 10-second
wait with status -2; acquired registrations/session cleanup returned 0. The user
reported the board icon was off and explained its short BLE availability window.
No exact duration or definitive cause of that timeout is claimed. After stopping
and reporting it, prepared one new test tied to a real advertisement: an ignored
Mac listener recognized the same cached peripheral UUID, stopped its scan and
triggered the SSH command already waiting on the Kindle. The user pressed the
Bluetooth button only after the listener and Kindle were ready. The Mac did not
connect. This was temporary checkpoint orchestration, not a runtime scan/retry
feature, and no board write was introduced.

At 22:04:41–22:04:43 CET the Kindle completed connect -> discover -> services ->
disconnect -> connect -> discover -> services -> quit. Two connections, four
successful UUID/property validations and two complete cleanups were recorded.
Every native setup/discovery/cleanup stage returned 0; final exit was 0 and the
operation lock cleared. All required UUIDs matched in reverse byte order,
including records mixed with 16-bit services. The user confirmed:
"Yes, normal behavior; H2 accepted". A final check found no probe process and
all 121 original app/data hashes unchanged. No subscription, characteristic read,
telemetry, LED, STOP or motor command ran. MTU and all write delivery remain
unverified. M2 is Implemented; stop before M3.

The acceptance commit updates documentation only. The installed receipt continues
to identify f19ecfb; future source-bound deployments need rebuild/restage.
