# Native boundary research and M1 build — 2026-11-04

M1 adds a one-shot native preflight worker; it never calls Bluetooth APIs.
H1 verified loading and worker privileges on the target Scribe. H2 subsequently
verified session setup, GATT discovery and clean reconnect. Notification/write
behavior remains unverified; earlier M0/M1 research is retained below. SDK, vendor headers, libraries and binaries are ignored
build inputs, never committed or bundled as device dependencies.

Machine-readable commits and SHA-256 are in
[notes/SOURCE_PINS.json](../notes/SOURCE_PINS.json). Git source hashes refer to
`git archive --format=tar <commit> | shasum -a 256`, not GitHub's generated
tar.gz. Cargo.lock independently records the published crate archive checksum.
Release asset digests are GitHub's advertised digests, not local verification.
M1 must download, check the digest, and inspect ELF before use.

## Releases, pins, and licensing

- [chessnut-move v1.0.2](https://github.com/daymxn/chessnut-move-rs/releases/tag/v1.0.2),
  commit `4f0468097b48c35151beb8d0743b169c3ae7bce3`, Apache-2.0.
  Latest release checked through GitHub's releases API; no newer release was
  advertised. Cargo uses registry **=1.0.2**, default-features=false; thiserror
  is its only resolved dependency. No async runtime/BLE transport is enabled.
  `protocol/command.rs` contains the H8/G8 byte fixture and the query fixtures;
  `protocol/wire/pieces.rs` defines all piece codes; packed squares run from
  H8 down to A1, with the first square in the low nibble. FEN maps into SDK
  canonical A1..H8 storage. The published API's `Position::squares` is private;
  the implementation uses public Position::new and piece_at.
- [KindleBT v0.1.0](https://github.com/Sighery/kindlebt/releases/tag/v0.1.0),
  commit `f50665b177941c450bfe6b5b72a23f8fe32069db`, is still the latest release
  (2025-10-08). Its release uses SDK build-20250905. The tag has no root LICENSE;
  do not infer licensing of that binary from a later file without review.
- **Proposed native build pin**: KindleBT current source commit
  [`59e1d8e69a52e3cd58cee6fcc416d1774ad55791`](https://github.com/Sighery/kindlebt/tree/59e1d8e69a52e3cd58cee6fcc416d1774ad55791),
  which contains an explicit MIT LICENSE. It also contains the public headers,
  docs and example referenced by the plan. Build source and worker against this
  same commit; never pair those headers with the v0.1.0 binary. The final library hash and H1 loading result are recorded in
  notes/DEVICE_RESULTS.md; session/discovery passed H2, while read/write behavior
  remains unverified. Building from a pinned
  post-release commit is an ABI correction, not an assumption of a newer release.
- [Sighery SDK build-20250905](https://github.com/Sighery/kindle-sdk/releases/tag/build-20250905),
  commit `f02be98a95d428196af6c3617cac178d85e5ed69`, is the KindleBT release's
  historical build reference. The newer SDK release
  [build-20260325](https://github.com/Sighery/kindle-sdk/releases/tag/build-20260325)
  still offers `scribe1-5.17.3.tar.gz`; its source pin and digest are recorded
  for M1. SDK scripts have a GPL-3.0 license file. Embedded toolchain/vendor
  files have their own licenses; SDK script licensing grants no permission to
  redistribute Amazon libraries. Use proprietary objects only as build/link
  references and the device's existing runtime dependencies.
- KindleBT's pinned Meson wraps pin log.c to
  `f9ea34994bd58ed342d2245cd4110bb5c6790153` (MIT), and obtain Darkroot/ACE headers
  from Espressif `acs_rtos_reference_ffs_1.15.0.tar.gz`, SHA-256
  `e70a82ab242ce9d1412949bb035cd972b785b2627179aa934912fcd751b3b545`.
  Those vendor headers are not project-authored or included here.

## Material API mismatch found

At the **release tag**, `src/include/kindlebt.h` declares:

```c
status_t bleGetDatabase(bleConnHandle, bleGattsService_t *);
```

At the **proposed current source pin**, `include/kindlebt/kindlebt.h` declares:

```c
status_t bleGetDatabase(bleConnHandle, bleGattsService_t **, uint32_t *);
status_t bleCleanupGattService(bleGattsService_t *, int);
```

The old tag also lacks the current `examples/basic_usage.c` layout. Source
version strings alone are insufficient: current Meson still labels itself
v0.1.0. M1 must record the exact source commit and resulting library hash.
This is the reason to build the pinned current source rather than silently use
an HF release with incompatible headers. Do not use guessed FFI layouts.

## Expected C ABI and ownership

The worker will include `<kindlebt/kindlebt.h>` from that source pin. Its types
are typedefs of ACE structures from the matching Darkroot headers, rather than
opaque Rust redefinitions. Compile with the SDK's ARM hard-float C toolchain;
Rust exchanges only framed JSON through pipes and never shares C structs.
Required declarations at this pin:

```c
status_t openSession(sessionType_t, sessionHandle *);
status_t bleRegister(sessionHandle);
status_t bleRegisterGattClient(sessionHandle, bleGattClientCallbacks_t *);
status_t bleConnect(sessionHandle, bleConnHandle *, bdAddr_t *,
                   bleConnParam_t, bleConnRole_t, bleConnPriority_t);
status_t bleDiscoverAllServices(sessionHandle, bleConnHandle);
status_t bleWriteCharacteristic(sessionHandle, bleConnHandle,
                               bleGattCharacteristicsValue_t *, responseType_t);
status_t bleSetNotification(sessionHandle, bleConnHandle,
                           bleGattCharacteristicsValue_t, bool);
status_t bleDisconnect(bleConnHandle);
status_t bleDeregisterGattClient(sessionHandle);
status_t bleDeregister(sessionHandle);
status_t closeSession(sessionHandle);
```

`bleGattClientCallbacks_t.size = sizeof(bleGattClientCallbacks_t)`. The example
uses `notify_characteristics_cb(conn_handle, value)`,
`on_ble_gattc_read_characteristics_cb(conn_handle, value, status)` and
`on_ble_gattc_write_characteristics_cb(conn_handle, value, status)`; the value
is passed **by value**, not pointer. Copy bounded notification payloads out of
callbacks. `bleGattCharacteristicsValue_t` includes a `gattRecord.uuid` and
`blobValue` with pointer `data`, `size`, `offset`; write values use
`format = BLE_FORMAT_BLOB`. Never retain borrowed callback data or free it as
if owned. Native build-time contracts must check actual typedefs/widths against
these exact headers; numeric enum values, padding and offsets are not assumed
from the development host.

UUID matching needs all 16 bytes and the correct UUID type. Check actual
KindleBT/ACE byte order at M2 using the known Chessnut UUIDs. The example's
`print_uuid` repeats `uu[5]` where index 6 belongs: do not copy that diagnostic
bug. Its simple callback booleans and early-error returns are examples, not
production timeout/cleanup handling.

Keep database memory until finished with characteristic records, then call
bleCleanupGattService with the returned count. Unsubscribe/disconnect,
deregister GATT/BLE, close the session, free owned buffers on every exit;
which operations actually succeeded must control cleanup. Worker-only
setgid/setuid must use the **verified** Bluetooth account/group, check both
calls and resulting IDs, before openSession. The example's 1003 must not be assumed; H1 independently verified
the named account/group and running daemon use these IDs on this target. Never change the Rust controller identity.

## Build and hardware gates

The documented path is Sighery's koxtoolchain + SDK + Meson cross file,
`arm-kindlehf-linux-gnueabihf`. Rust remains
`armv7-unknown-linux-gnueabihf`; target GLIBC must be <=2.35. The main app's
Zig target `arm-linux-gnueabihf.2.35` is a reference, not a replacement for
vendor headers/link references. Use an experiment-local toolchain cache and
library path. Pin/verify additional build tools when M1 installs them.

KindleBT uses pkg-config `ace_bt`, dynamically links the vendor `libace_bt.so`
and libdl, and patches absolute SDK NEEDED paths back to `libace_bt.so`. M1
must inspect the final worker and every staged shared object for ELF32 ARM,
hard-float ABI, expected loader (`/lib/ld-linux-armhf.so.3`), NEEDED dependencies
and GLIBC symbols <=2.35, then preflight their existence/loadability on Kindle.
The vendor's transitive NEEDED list cannot be guessed from source; record it
from the actual artifact. Never ship vendor libraries or write `/usr/lib`.

Current docs report Scribe testing on **5.17.2**, not this target's **5.19.6**.
The public API has no scan or MTU-request call; an internal MTU callback does
not prove any negotiated value. Full notifications, write-without-response
35-byte delivery, and STOP delivery remain **UNVERIFIED**. Requests and
callbacks must report their actual status; successful encoding/IPC is not BLE
acceptance or motor completion. No Bluetooth daemon restart, radio changes,
stock UI changes, or device commands were performed during M0.

## M1 implementation and artifact verification

The builder now uses the proposed current KindleBT source pin and matching
headers. Archive hashes are independently pinned in scripts/delivery.py; the
SDK and Espressif downloads were locally checksum-verified. Zig 0.13.0 replaces
the SDK's Linux-hosted compiler on macOS while reproducing Meson's eight source
files, log.c, includes and dynamic ACE dependencies. There is no firmware SDK
installation on the Kindle. The SDK archive is only a host link/inspection
reference. Its sysroot mixes toolchain and firmware library copies: extraction
resolves links as data and prefers firmware usr/lib over the newer GCC lib
runtime where basenames overlap. No extracted symlink or vendor binary is staged.

ACE lacks SONAME, so Zig records its absolute SDK path in DT_NEEDED. The narrow
localize_needed helper adjusts that entry's string offset to the existing
libace_bt.so suffix, performing the same correction as upstream's patchelf
step without changing any vendor file. A test rejects any other absolute input
and verifies the resulting dynamic dependency. The final ELF verifier rejects
all remaining absolute dependencies and RPATH/RUNPATH.

kindlebt_worker.c compiles against the exact public header and asserts 32-bit
pointers, all 16 UUID bytes, and the current bleGetDatabase signature. It does
not link KindleBT directly; preflight loads the sibling shared library with
RTLD_NOW, looks up the required public symbols and closes it. No function
pointer is invoked. The Rust controller supervises the child with a 15-second
deadline and SIGINT/SIGTERM/SIGHUP cancellation. Only the worker clears groups
and drops to verified Bluetooth IDs; failure aborts. Runtime symbol loading
cannot establish that callback layouts or vendor ABI behavior work on 5.19.6;
H2 subsequently exercised session/discovery callbacks; notification/write
callback behavior remains untested.

ELF inspection reads program headers, PT_INTERP, DT_NEEDED and the real
DT_VERNEED records. All three staged native artifacts are ELF32 ARM EABI5
hard-float; both executables name /lib/ld-linux-armhf.so.3. Their maximum GLIBC
requirement is 2.34, below the measured device's 2.35. The complete SDK-derived
reference dependency closure (24 library names, including the loader) is
stored in build.json as sdk_reference_libraries. required-libs.txt lists only
the four direct device imports of the shipped binaries. Preflight loads these
and the local KindleBT library with RTLD_NOW, allowing the loader to validate
the actual firmware's transitive chain. No device libraries are bundled. The exact final hashes, input hashes, flags,
source digest and validation reports are generated in target/kindle/build.json
and copied into the staged runtime. Stage creation refuses a stale source or
binary receipt; deployment verifies the stage before opening SSH.

M1 uses fixed line-sized text reports, not the future session IPC. Production
arguments accept only --preflight plus the controller's sibling library/list
paths. There is no scan, address, registration, session, subscription or write
path. Firmware/account/library checks passed H1 on-device, and the user
confirmed normal original-app display, response and exit. The worker deliberately requires root SSH so it can prove that only
the child changes identity. Its read-only probes never restart btmanagerd or
change stock radio/UI/power state.

## H1 startup correction and device verification (2026-11-04)

The first device run exposed an early native-worker SIGSEGV. The saved worker
contained a COPY relocation that promoted aliased Zig libc-stub globals into
the executable. stdout, stderr and the environment incorrectly shared storage.
The worker now compiles with -fPIC -pie, and the ELF validator rejects ET_EXEC
executables and R_ARM_COPY entries in REL/RELA tables. New receipts include
PIE and COPY-relocation checks. The C source and vendor/KindleBT code are
unchanged. See [the evidence and confidence limits](../notes/M1_STARTUP_CRASH.md).
Initial analysis was offline while the Kindle was stopped. The user later
made it available and authorized deployment; corrected worker startup passed.

Preflight identity verification was also corrected after observing that the
passwd primary GID is 150 while btmanagerd runs as UID/GID 1003. It now validates
all four kernel process UID/GID fields against the named account/group instead
of assuming primary-GID equality; host C fixtures cover malformed and mismatched
records. No system accounts or services are changed.

The device's 5.19.6 libace_bt has a different NEEDED list than the 5.17.3 SDK
reference. Requiring the SDK-only libcrypto.so.1.0.0 produced a false failure
after the real libace_bt had already loaded successfully. The preflight list
now comes only from shipped artifacts; the device loader validates current
transitives. No obsolete SDK library is installed to satisfy a reference name.

Final build 138cc25 passes H1: daemon identity, worker privilege drop, direct
and transitive dependency loading, local KindleBT loading and required symbol
resolution all passed; preflight exited 0 and cleared its lock. The user confirmed
the original app works normally, with all 121 binary/data hashes unchanged.
M1 is Implemented. No Bluetooth API was called or board connection attempted.

## M2 discovery-only native adapter (H2 accepted 2026-11-04)

kindlebt_session.h adapts the pinned public C types to the portable, host-tested
session_core.h lifecycle. Only open/close, registration/deregistration,
connect/disconnect, discovery/database retrieval/free and connection-state
observation function pointers exist in the M2 table. No command, characteristic,
descriptor write or subscription API can be dispatched by session IPC.
The worker repeats preflight and drops privileges before loading this table.

Discovery and database callbacks use C atomics. KindleBT clones the database
before notifying the application; we keep only its owned clone. Callback payloads
are never retained. The database traversal checks type, full UUID width, counts,
linked-list bounds, record count and final frame size. It rejects unknown UUID
types and never silently truncates a list. Native UUID bytes are returned without
assuming an order; Rust validates all required service/characteristic pairs in a
consistent forward or reverse order and checks property masks. H2 verified reverse 128-bit byte order, discovery callbacks, database ownership
and cleanup on firmware 5.19.6. Notification/write callbacks remain untested.

KindleBT exposes no application connection-state callback at this pin. The
adapter resolves exported devices_context/dca_find_by_handle and observes
callback-updated state under its mutex while idle. This depends on the pinned
internal context contract, so the build and preflight pin must remain aligned.
There is one API-owning thread, one connection and no concurrent context-array
mutation. The CLI starts a fresh worker for each explicit reconnect.

The library's own callback waits are normally 10 seconds. IPC uses a 45-second
deadline and 55-second cleanup grace; a stuck vendor call ends in a reported
forced reap, not claimed successful cleanup. Unknown handles on failed/timed-out
connect cannot be explicitly disconnected: acquired registrations/session are
closed, and H2 must verify actual link release. Failed deregistration is reported;
no daemon restart or radio-control workaround is attempted. M2 does not call
explicit enableRadio/disableRadio APIs; openSession/closeSession retain their
upstream lifecycle behavior, which still needs physical interference checks.

The build runs native_adapter_test.c against the real downloaded, checksummed
KindleBT and Darkroot headers on the host, then compiles the actual ARM worker
against the same headers. The host test renames only the typedef token uuid_t
to avoid Darwin's unrelated system typedef; the vendor structure stays intact.
It exercises full-byte UUID formatting, database serialization/count/cycle
rejection and callback handling without opening BLE. Neither that host executable
nor any SDK/vendor object is staged. Portable C lifecycle/parser fixtures also
run in the ordinary independent host gate, without SDK downloads.

H2 passed on installed source f19ecfb with two fresh-worker connections and
clean disconnect/reconnect. The user accepted normal physical behavior. No
subscription or board write ran. BusyBox ash required the launcher to save input
on fd 3 before spawning in the background; a regression and on-device input
probe verify that boundary. The board’s BLE availability is short, with no exact
window measured; prepare the command before human button activation. See
DEVICE_RESULTS.md for final evidence. M2 is Implemented; stop before M3.


## M3 notification, write and MTU boundary (host-ready, H3 NOT TESTED)

The worker now resolves bleSetNotification/bleWriteCharacteristic against the
same real pinned headers, with an independent strict parser and non-motor packet
allowlist in m3_wire.h. kindlebt_m3.h handles both notification subscriptions,
bounded callback copies, callback completion, LED cleanup and optional MTU evidence.
The old M2 behavior above describes the accepted installed build; M3 is not yet
deployed. Native tests cover partial subscription unwind, exact forward/reversed
UUID and parent matching, copied payload lifetime, invalid blob format/size,
write API/callback failures, callback timeout and fixed all-off cleanup.

MTU uses exported log_add_callback from the checksummed log.c build. The observer
matches precisely the pinned bleMtuUpdatedCallback format and function name, then
copies numeric status, MTU and connection from va_list. Only status 0 for the
current connection and MTU 23..517 becomes known. No raw vendor log is emitted;
missing exports/callback or unexpected formats yield unknown. This is a private,
version-specific source contract, not a public getter or a negotiated MTU request.
The library is unchanged, and its mapping remains live until worker exit.

Both write modes require API and matching callback success within five seconds;
WithoutResponse completion is local stack evidence, not a board ACK. SDK LED
packets are 34 bytes and require observed MTU >=37. Writes use persistent worker
buffers even after timeouts. Normal cleanup sends all-off before unsubscribing;
after an ambiguous timeout, a separate persistent all-off buffer is used for one
best-effort cleanup write and cleanup remains failed. The database is freed only
after subscription cleanup. No motor/STOP packet is permitted, no payload is
split or truncated, and no command is replayed on reconnect. Hardware notification
sizes, actual MTU, callback availability and physical LED behavior need H3.
