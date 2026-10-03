# Phase-one measured results

These results summarize the first Kindle milestone on the measured target: **Kindle Scribe
1st generation (Barolo), firmware 5.19.6, ARMv7 hard-float, glibc 2.35, 1860x2480
Gray8 framebuffer path**. Detailed raw observations and historical regressions are retained
in [device/ks1-barolo.md](device/ks1-barolo.md).

The goal is an evidence baseline for later optimization and real-user-feedback work, not
laboratory benchmarking.

## Build/runtime footprint

Task 09 added release-size reporting to the existing pinned build workflow. CI run
`37113721806` at commit `34be5551d592ad61afe0d7ae62b7580efb30f4fc` passed the
full host gate and Kindle release build and measured:

| Artifact | Observed size |
| --- | ---: |
| optimized `armv7-unknown-linux-gnueabihf` `kindle-chess` executable | **1,563,364 bytes** (about 1.49 MiB) |
| uploaded CI artifact bundle (`kindle-chess.tar`, KPM package, build receipt; Actions-compressed) | **16,056,380 bytes** (about 15.31 MiB) |

The larger distribution bundle intentionally includes install/package material and
corresponding-source payloads; it is not the resident application binary size. Task 09
does not change production Rust behavior, so this is the phase-one executable footprint
for the same runtime code exercised in the final Task 08 device checks.

The final physically installed Task 08 executable recorded in the device log has SHA-256
`cc6b60a4dbd71c1fc3417959410775f3e7e087421b1e90e9693a91205cd15e52`.

## Responsiveness

| Observation | Phase-one result | Notes |
| --- | --- | --- |
| process entry -> first usable frame | **437 ms** in the optimized Task 07 retest; **440 ms** through the installed Task 08 package entrypoint | FBInk initialization was 10 ms in the 437 ms run; the packaged smoke submitted its first full update in 35 ms and completed it in 397 ms |
| touch -> final update submission, puzzle navigation | **48-51 ms** | final batched release; navigation submits three damage regions |
| touch -> navigation update completion | about **0.43 s** | final batched release; measured completion about 390 ms after submission |
| touch -> final update submission, NOTE toggle | **33-35 ms** | two damage regions |
| touch -> NOTE update completion | **214-226 ms** | final batched release |
| touch -> visible square selection | no isolated final-build stopwatch value was retained | selection is constrained by host damage tests to one compacted region; the pre-batching build's 1.17-1.62 s selection result is obsolete and is not presented as the final result |
| app Exit -> usable stock UI | repeatably observed as normal/usable, but no standalone stopwatch value was retained | final library regression verified Xorg/awesome resume, native repaint, no remaining app/launcher, and working native UI |

The missing isolated selection and exit stopwatch numbers are measurement gaps, not
claims of zero latency. Real-device feedback after the batching/display-ownership fixes
described the final interaction as responsive.

## Refresh and ghosting observations

The phase-one refresh path changed materially during device testing:

1. Task 04 showed short residual traces on a moved piece before a later full flash cleared
   them.
2. An early Task 07 implementation submitted as many as 38 rectangles serially; navigation
   could take about 10.5 seconds. That implementation failed responsiveness validation.
3. Damage compaction and batched presentation reduced normal navigation to three regions
   and the timings above. The user reported responsive interaction and clean piece redraws.
4. A later white/native patch was traced to stock winmgr/Xorg drawing over direct FBInk
   output, not to missing application damage. Exclusive finger input plus the scoped
   awesome/Xorg display handoff fixed the observed contention. Consecutive framebuffer
   probes remained stable beyond the native overwrite window.
5. The user accepted the remaining non-sleep ghosting/modal observations when Task 07 was
   closed.

The final phase-one assessment for ordinary awake puzzle use is therefore **acceptable
ghosting with no known persistent stale application pixels**. A full redraw remains
available for recovery.

## Reliability summary

The physical checkpoints cumulatively exceed 100 touch interactions. Task 04 alone logged
53 accepted taps; Task 05's nine scripted parity groups require more than 50 additional
touches, and Tasks 06-08 add further collection, promotion, navigation, and launch/exit
use.

Verified final-path observations include:

- repeated board/control coordinate mapping without drift or frequent missed touches;
- wrong/correct/complete solving and repeated promotion flows;
- multi-collection browsing and progress persistence across normal restart;
- source puzzle collections unchanged while progress is stored separately;
- normal exit, SIGTERM, killed-child cleanup, and exclusive-input release;
- library launch with stable framebuffer ownership and top-right X exit;
- update and uninstall/reinstall without losing puzzle/progress data.

See [PARITY.md](PARITY.md) for the behavioral matrix.

## Known limitations at phase-one close

- **Natural idle suspend/resume is unverified.** No claim is made about framebuffer
  invalidation, input descriptors, or refresh reinitialization after sleep.
- **Library launch after a full device reboot is unverified.** Ordinary library relaunch
  and recovery are verified.
- Stylus input is filtered from the puzzle interaction path; stylus-specific product
  interaction is not implemented.
- Version-1 puzzle collections retain the 256 KiB compatibility limit.
- Unsolved-only navigation and version-2 branching/rich solutions are deliberately
  deferred.
- The application intentionally does not implement general chess legality, engine
  analysis, PGN, castling validation, or en-passant behavior.
- The first milestone is validated on the first-generation Scribe/firmware listed above;
  other Kindle models/firmware are not claimed supported by this record.

These limitations define the phase-one baseline. Improvements prompted by ongoing real
puzzle use should be planned as the next product phase rather than silently changing the
parity contract.
