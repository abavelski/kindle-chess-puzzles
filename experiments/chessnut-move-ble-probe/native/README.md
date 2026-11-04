# Native boundary research and M1 build — 2026-11-04

M1 adds a one-shot native preflight worker; it never calls Bluetooth APIs.
The M0 findings below still constrain M2. No runtime compatibility on the target
Scribe is established. SDK, vendor headers, libraries and binaries are ignored
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
  same commit; never pair those headers with the v0.1.0 binary. No native
  compatibility or binary hash is established yet. Building from a pinned
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
calls and resulting IDs, before openSession. The example's 1003 is not a
verified value on this target. Never change the Rust controller identity.

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
M2 must exercise those behind H1 acceptance.

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
path. Firmware/account/library checks and the original app check are NOT TESTED
on-device. The worker deliberately requires root SSH so it can prove that only
the child changes identity. Its read-only probes never restart btmanagerd or
change stock radio/UI/power state.

## H1 startup correction (offline, 2026-11-04)

The first device run exposed an early native-worker SIGSEGV. The saved worker
contained a COPY relocation that promoted aliased Zig libc-stub globals into
the executable. stdout, stderr and the environment incorrectly shared storage.
The worker now compiles with -fPIC -pie, and the ELF validator rejects ET_EXEC
executables and R_ARM_COPY entries in REL/RELA tables. New receipts include
PIE and COPY-relocation checks. The C source and vendor/KindleBT code are
unchanged. See [the evidence and confidence limits](../notes/M1_STARTUP_CRASH.md).
The user stopped the Kindle; no redeployment or SSH retest occurred during
this investigation. H1 remains failed until the corrected candidate is tested.

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
