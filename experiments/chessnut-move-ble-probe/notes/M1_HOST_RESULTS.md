# Milestone 1 host preparation — 2026-11-04

Status: **Host-ready; awaiting H1**, not Implemented. No SSH connection,
deployment, Kindle preflight, board operation or stock-system change occurred.
All changes remain on experiment/chessnut-move-kindlebt-probe under this
experiment directory. H1–H5 remain NOT TESTED.

## Automated acceptance

- Independent gate: fmt, clippy with warnings denied, **18 Rust integration
  tests** and **10 Python delivery contracts**. The full gate is scripts/check.sh.
- Parent gate: fmt, workspace clippy, and all **237 workspace tests** passed;
  used the existing ignored host-check virtual environment for the documented
  Python chess dependency. No parent source/dependency files changed.
- Rust preflight supervision accepts child success, reports failures, enforces
  timeout/interruption and reaps children. Real SIGINT and SIGTERM sent to a
  copied host controller terminate/reap its mock worker. Native source has no
  board/session calls; the ARM C worker compiles with warnings denied and
  current-header ABI contracts.
- ELF fixture tests reject host/truncated ELF, wrong ARM/float ABI, wrong
  interpreter and GLIBC_2.38. Shared-library handling is tested separately.
- SHA256SUMS checks reject tampering, unexpected files, symlinks and traversal.
- Device shell scripts run only against substituted temporary experiment roots
  in host tests. Initial install, update and scoped uninstall preserve a sibling
  kindle-chess/progress.json. Active locks, symlink targets and bad checksums
  are rejected. A failed runtime.new swap restores the prior runtime and removes
  its lock. No test accesses /mnt/us or SSH.
- SSH argument tests reject shell characters, option-like users and bad ports.
- The final build/stage/verify commands produce a checksummed payload containing
  exactly two executables and one shared library, plus receipts, notices,
  launcher and scoped installer/uninstaller. Stage/deployment reject stale or
  altered sources/artifacts. The generated build receipt contains the final
  source digest, input archive hashes, flags, artifact hashes and ELF reports.

## Red-to-green observations

Added the delivery tests before delivery.py existed. Once implemented, five
ELF/manifest cases passed. Added installer contracts before device scripts
existed; then fixed a failing symlink-refusal assertion by making shell guard
failures explicit (a leading failed command in an && list does not reliably
terminate under set -e). All installer contracts pass. Added worker supervision
tests before the Rust module existed, then implemented the bounded supervisor.
Added an option-like SSH user rejection test, observed failure, and tightened
validation before remote execution.

The first ARM build correctly rejected an absolute SDK path in DT_NEEDED. Added
a failing test for its narrow correction, then implemented changing that entry
to the existing libace_bt.so suffix. Other absolute paths remain errors. No SDK
or vendor binary is patched. Source downloads are checksum-verified and fresh
source extraction prevents edited-cache builds. SDK extraction streams regular
library data and resolves aliases without creating symlinks; firmware usr/lib
references take precedence over the GCC runtime overlay.

## Native artifacts and remaining limits

Verified on the development host: ELF32 ARM EABI5 hard-float; both interpreters
/lib/ld-linux-armhf.so.3; all staged GLIBC requirements <=2.34 (target <=2.35).
KindleBT is built from 59e1d8e69a52e3cd58cee6fcc416d1774ad55791 with matching
headers and MIT notice. The SDK-derived device dependency closure has 24 names.
No proprietary device library or mock worker is staged. Exact artifact hashes
and inspection reports are in target/kindle/build.json and its staged copy.

The SDK is Scribe 5.17.3, not the target's 5.19.6. Preflight will test library
loading and account permissions on the real device; it cannot establish runtime
BLE compatibility. No numeric Bluetooth UID/GID or MTU is assumed. The launcher
accepts only preflight, which never opens a session. M2 is not authorized by a
host build: stop for human deployment and H1, record results, and leave the
original chess app and Bluetooth services unchanged.

## Deployment transport correction — 2026-11-04

The real Kindle SSH connection confirmed armv7l and firmware 5.19.6. Its legacy
SCP server rejects a recursive source ending in `/.` with `unexpected filename:
.`. The failed upload was cleaned up, leaving only the experiment ownership
marker and no installed runtime. A failing regression test reproduced the
restriction before changing the uploader to name install.sh, SHA256SUMS and
runtime explicitly. The delivery suite now has 11 Python tests. Retry uses the
same checksums, scoped installer and cleanup; no Bluetooth preflight/session
was run as part of this transport correction.
