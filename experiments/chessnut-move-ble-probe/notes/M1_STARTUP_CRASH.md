# H1 native startup crash: offline diagnosis and candidate fix

Date: 2026-11-04. Branch: experiment/chessnut-move-kindlebt-probe.
Status: defective binary layout identified and corrected in the host build;
**device confirmation pending**. The Kindle was stopped by the user, and no SSH
or device commands were attempted during this investigation.

## Evidence from the exact failed artifact

Installed build dc8d7bc used worker SHA-256
`8ac86445f56fcc2e43f6a1cd21b3b31a0aff8e81cce04978e619d5a38f758290`.
The Rust controller ran, but this worker died with SIGSEGV before emitting any
preflight checks. Checksums passed. No crash core/backtrace was collected.

Offline ELF inspection of that saved worker found a non-PIE ET_EXEC executable
with an R_ARM_COPY dynamic relocation for stdout. Its exported dynamic symbols
incorrectly assigned all these unrelated globals the same address:

| Symbol | Address | Size | Defined in executable |
| --- | --- | --- | --- |
| stdout | 0x4b454 | 8 | yes |
| stderr | 0x4b454 | 8 | yes |
| stdin | 0x4b454 | 8 | yes |
| environ | 0x4b454 | 8 | yes |
| __environ | 0x4b454 | 8 | yes |

`environ` and `__environ` may legitimately alias each other; neither may alias
stdio streams. ARM disassembly shows that main loads stdout from this address
and immediately calls setvbuf, before its first preflight report.

The pinned [Zig 0.13 glibc stub generator](https://github.com/ziglang/zig/blob/0.13.0/src/glibc.zig#L900-L1038)
emits data-symbol labels and sizes without separate runtime storage. The
non-PIC executable link promoted the stub's aliases through the COPY relocation
into the executable's own definitions. These are link-time stubs, not a libc
that can supply runtime object contents. By comparison, the existing Rust
controller and shared KindleBT library import their libc globals indirectly
and do not exhibit this exported alias group.

[Glibc 2.35 initialization](https://github.com/bminor/glibc/blob/glibc-2.35/csu/init-first.c)
assigns the environment pointer to __environ. With the defective exported
aliases, an environment assignment can overwrite stdout. A subsequent setvbuf
would then receive an invalid FILE pointer. This is a concrete binary defect
and a strong explanation for the observed early SIGSEGV. Without a device
backtrace or successful retest, the exact faulting instruction and complete
resolution remain unconfirmed.

## Correction and regression gates

Compile the C worker with **-fPIC -pie**, retaining the pinned compiler,
architecture and glibc target. No Kindle system, libc or vendor library change
is needed. Native ELF validation now requires executables to be PIE and
inspects actual DT_REL/DT_RELA relocation tables to reject **R_ARM_COPY**.
The check also applies to the staged payload and is included in build receipts.

Two new host regressions first failed against the old validator: an ET_EXEC
worker and a PIE containing R_ARM_COPY were accepted. Both are now rejected.
The saved failed worker is also rejected by the new validation. Building the
same C source with the corrected flags produces ET_DYN/PIE, no COPY
relocations, and undefined/imported stdout/stderr rather than the incorrect
executable-owned aliases. Its dynamic interpreter remains
/lib/ld-linux-armhf.so.3 and its highest GLIBC requirement remains <=2.35.

The full independent gate contains 18 Rust integration tests and 13 Python
delivery tests. Final build/stage receipts contain the corrected flags, PIE
status, copy-relocation count, exact new hashes and source digest. The candidate
must pass build.sh, stage.sh and verify-stage.sh before deployment. No binary
or SDK artifact is committed.

## Next device boundary

The currently installed dc8d7bc worker is still the failed version. A host
candidate is not proof of H1 success. When the user makes the Kindle available
again, deploy the corrected bundle and rerun the same harmless preflight over
SSH. Inspect its output/exit status and cleanup; keep H1 failed until it passes
and the human confirms the original chess app still displays/operates normally.
No board address, BLE session, motor or M2 work is required or permitted for
this check. The assistant can perform the SSH deployment/preflight directly;
the user need only make the Kindle available and observe the physical app.

## First corrected-device run and revised identity verification

The user made the Kindle available again and authorized deployment. Build
3851829 was installed and ran past startup: armv7l, kernel 4.9.77-lab126,
glibc 2.35 and firmware 5.19.6 were reported. The SIGSEGV did not recur.
Preflight then rejected its own assumption that the bluetooth passwd primary
GID must equal the named bluetooth group GID. Read-only observations showed:

- bluetooth account UID 1003, passwd primary GID 150;
- named bluetooth group GID 1003;
- running btmanagerd real/effective/saved/filesystem UID and GID all 1003;
  its supplementary groups are 150 and 1003.

Updated preflight plan before changing identity code: resolve the named account
and named group, reject zero target IDs, and verify all four daemon UID/GID
fields against those IDs using /proc status. The account primary GID is
reported but need not equal the daemon's effective group. Only after this proof
may the worker clear supplementary groups, setgid/setuid to the verified target,
and verify its resulting IDs. Never edit passwd/group or system services.
This replaces the invalid equality assumption with evidence from the actual
Bluetooth process. Host fixtures must cover the observed mapping, mismatched
UID/GID rows, partial/malformed rows and repeated identity records before code.
