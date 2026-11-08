# Chessnut Move MVP verification

Status: Task 59 physical seating deployed on 2026-11-07 after explicit user
authorization. Tested-source/package/stage validation, installed hashes, user-data
preservation, secure config, loader and native preflight PASS. Physical seating
acceptance remains pending.
The general user report does not certify every original checklist item. No board connection or motor command was attempted
while implementing this change. Work is on `feature/chessnut-move-mvp`, based
on `main`; the probe branch was read for its pinned build recipe, never merged.

## Implementation and host boundaries

The Puzzles Settings row is transient session status. It does not change
`settings.json`, puzzle collections, progress, or review resume. The optional
send icon follows Analysis in STANDARD and SMALL. Review has no board controls.
The target is a canonical placement captured from `AppState::board()` at the tap;
Flip does not rotate it. Pending promotion blocks sending. Send immediately
submits that snapshot without confirmation. It stays selected/disabled only until
transport submission finishes; the snapshot is then discarded. There is no DONE,
post-motion verification, source-refresh wait or automatic retry/replay.

`kindle-platform::chessnut` owns one bounded service thread and one separately
privileged native worker. Connection still checks callback ownership, GATT
parents/properties, MTU and CCCD completion, capabilities, UID/GID, then enables
realtime and requires one complete position report before declaring ready.
Vendor output is discarded instead of exposing addresses in routine logs.

Every explicit tap authorizes one Arm/35-byte Normal WithoutResponse pair. A
later explicit tap can arm another target without STOP or position reports.
Native one-shot authorization, serialization, callback/MTU checks and error
cleanup remain. Its conservative motion_maybe flag still triggers best-effort
cleanup on session exit; it no longer blocks another explicitly armed target.
A successful transport callback is not a motion-completion acknowledgement.

Settings displays DISCONNECT whenever connected. Idle telemetry gaps, navigation
and sleep/wake do not deliberately close the link. Startup, wake, reconnect and
navigation never initiate a target. Actual worker/link errors require explicit
reconnection; failed commands are never retried. The UI polls asynchronous input
and service state without waiting on BLE. Exit/signal cleanup retains its existing
55-second worker / 65-second wrapper grace periods.

The user explicitly chose send-and-forget on 2026-11-06, superseding the DONE
workflow recorded below. There is still no verified automatic motion-finished
event, and the app makes no synchronization/completion claim. STOP, disconnect
and app exit do not guarantee physical interruption of an already-issued move.

## Build and local package checks

Run:

```sh
scripts/check.sh
sh support/chessnut-move/scripts/check.sh
scripts/build-kindle.sh
scripts/stage-kindle.sh
```

The app build now also builds the minimal C worker and patched pinned KindleBT
DSO, checks the real pinned native headers, ELF32 ARM EABI5 hard-float, glibc ≤2.35,
PIE/COPY relocations, exact NEEDED/dependency lists, and bundle hashes. Only
`runtime/lib/libkindlebt.so` is bundled; Amazon ACE/SDK libraries are build-only
references. Native/protocol notices and corresponding Rust/project source are
included. Build receipts bind the worker bundle to the exact app source. Staging
rejects missing, changed or extra payloads. Host install/update/rollback tests
operate in temporary directories and preserve user data.

Renderer checks cover every session state, direct Send/selected Send/error, both board
sizes, toolbar adjacency and minimum hit sizes, landscape Settings fit and
unchanged Review. Inspectable Gray8 examples are emitted under
`target/chessnut-send-forget-snapshots/`; existing settings snapshot changes are confined to
the added row. Device acceptance is separate from those checks.

## Private config provisioning

The Kindle controller reads only
`/var/local/kindle-chess/chessnut-move.private.json`, a regular file at most
1024 bytes with mode 600, containing one `address` string. It is outside ordinary
settings and packages. Do not put a real identifier into tracked docs or logs.

Prepare from the retained, ignored local probe record (does not contact a device):

```sh
python3 tools/prepare_chessnut_config.py
```

This extracts only `ble.address` from
`local-kindle/chessnut-move/devices.private.json` into
`local-kindle/chessnut-move/runtime.private.json` with mode 600. Both paths are
ignored. For supervised installation, close the app, back up the current runtime,
install the staged package using the usual documented procedure, and privately
copy this file to the Kindle path above, then set mode 600 there in a root-owned
mode-700 `/var/local/kindle-chess` directory. Do not stage it under `/mnt/us`: the
verified `fuse.fsp` mount reports mode 777 even after chmod. Do not print the
file or pass its address as a shell argument. A missing/malformed/public-readable
config is an actionable connection error. There is no address picker or scan UI.

## Human checkpoint — send-and-forget pending

1. Connect, check DISCONNECT, disconnect/reconnect without motion.
2. Choose a known visible target and tap Send: no confirmation or DONE. Send is
   selected/disabled during submission and becomes available after submission.
3. After inspecting the board, submit another explicit target on the same link.
   No STOP/realtime verification or new source report should be required.
4. Check idle persistence, navigation and sleep/wake without automatic commands;
   test STANDARD/SMALL touch targets, analysis/Free Board capture, promotion
   rejection, unchanged Review and puzzle grading/progress.
5. Check actual connection/error recovery and app exit; no automatic retry/replay.
   Compare user-data hashes and retain the previous runtime for rollback.

The original MVP was generally reported working by the user. New send-and-forget
behavior awaits human acceptance. Earlier DONE-era records below are historical.

## Initial automated result — 2026-11-05 (superseded artifact)

- `scripts/check.sh`: PASS, including fmt, clippy, 252 Rust tests,
  renderer snapshots, Python conversion/contracts, lifecycle and packaging tests.
  Used `target/chessnut-tools-venv` with the pinned `requirements-tools.txt` dependency.
- `sh support/chessnut-move/scripts/check.sh`: PASS (20 Rust and 11 Python/native
  contract tests). The ARM build also passed the actual pinned-header native test.
- `scripts/build-kindle.sh`: PASS; app, worker and KindleBT ELF checks passed.
  Independently inspected the app's dynamic tables: PIE, no COPY relocation,
  maximum required glibc 2.34, imports loader/libc/libm/libpthread only.
- `scripts/stage-kindle.sh`: PASS. Actual staged payload installed, uninstalled and
  rolled back in a host temporary directory with all user-data hashes unchanged.
  Package/source scan found no private address/config or bundled Amazon library.
- Initial package (superseded at the same output path after the deployment correction):
  `target/kindle/kindle_chess_0.1.0_kindlehf.kpkg`.
  SHA-256: `227a3ed22c10a27f71d3fb41eecf301d338dd578df21a739c3861b57fbbddca3`.
- Exact code/source digest: `321167eb75dd21b2b85c42313f5867edb19aa562c013feb5ab6597ce02d4d580`.
  Receipts: `target/kindle/build.json` and `target/kindle/mvp-verification.json`.
- At the initial software handoff no deployment, Bluetooth connection, motor write,
  commit or push had been performed. Subsequent deployment is recorded below.
  HEAD and `main` remain at `3bab075`; all implementation changes are uncommitted
  on `feature/chessnut-move-mvp`.

These are software/build results only. At that initial handoff every item in the original human checkpoint
remained pending. Retain the pre-install device runtime for physical rollback.

## Deployment — 2026-11-05

Installed through the verified SSH/scp workflow on Barolo firmware 5.19.6,
ARMv7 hard-float, glibc 2.35. The original runtime and matching library scriptlet
were backed up before installation; the exact backup path and receipt are in
ignored `local-kindle/chessnut-move/mvp-deployment.private.json`.

Deployment found that `/mnt/us` (`fuse.fsp`) reports config mode 777 after chmod.
The plan was updated before changing the adapter. Its default now uses
`/var/local/kindle-chess/chessnut-move.private.json`; device stat verified owner 0,
mode 600 and root-owned parent mode 700. The temporary USB-storage copy was
removed. A new host regression test protects this default location. The app was
rebuilt, restaged and redeployed with that correction.

Original deployment verification (before the interaction improvements):

- Complete host gate PASS (253 Rust tests plus Python/native contracts), pinned
  ARM build and stage PASS.
- Installed app/worker/KindleBT/dependency-file hashes match the tested stage.
- All original puzzle/game/state file hashes remain unchanged. Private address
  provisioning is separate from ordinary settings and progress.
- App dynamic-loader check PASS without running main or taking over the display.
- Native preflight PASS: Bluetooth UID/GID 1003/1003, daemon identity, privilege
  drop, actual library loads and required symbols verified. Session NOT_OPENED;
  connection NOT_ATTEMPTED. No app or board worker remains running.
- Latest package SHA-256: `b2178130b23ca0653499732b742902d8cfe173359aae7e6400bba76076f6cfe6`.
- Latest code/source digest: `b8d96780caca51dc8858ad0fb5df3b82c6de4f1e0358ee21747da79fe2cf6e99`.

At deployment handoff no Chessnut connection, Normal/STOP write or motion test
had been attempted by the agent. The subsequent user report confirms general
operation of that version, without item-by-item results.
The original backup is retained; no commit or push was made. Uninstalling the
runtime preserves the separate private config under `/var/local/kindle-chess`.

## Interaction improvements — 2026-11-06

The user requested persistent connection with DISCONNECT, immediate send without
confirmation, and a selected/disabled Send while arranging. They accepted a
separate DONE fallback. Implemented on the existing branch without commits/pushes.

- Host gate PASS: fmt, clippy with warnings denied, 258 Rust tests and all Python/
  lifecycle/packaging contracts. The focused platform tests also passed after a
  regression fix for late source reports racing an explicit retry.
- Preserved support/native gate PASS: 20 Rust tests and 11 Python/native contracts.
- Visually reviewed 40 monochrome state snapshots across STANDARD/SMALL and
  board/Settings; hit tests also cover portrait/landscape, minimum touch sizes,
  selected/inert Send, separate DONE, DISCONNECT and unchanged Review controls.
- Pinned ARM build PASS, including actual native-header, ELF and dependency checks.
- Stage/manifest PASS. The actual staged payload passed host install, uninstall
  and backup restoration with user-data hashes preserved. Package/source scans
  found no private identifier/config. Build and package verification receipts:
  `target/kindle/build.json`, `target/kindle/improvements-verification.json`.
- SSH attempts timed out before installation. No device app launch, connection
  or motor command was performed for this update. The previous deployed version
  remains the version generally verified by the user.

Human acceptance for these new interactions remains pending.

## Interaction-update deployment — 2026-11-06

After the user made the Kindle reachable and closed the app, the validated update
was installed through the existing SSH/scp workflow. A fresh backup preserves the
previous runtime and matching library scriptlet; a new pre-update baseline covers
all puzzle, game and state files. The original 2026-11-05 backup is also retained.
The exact backup path is in ignored
`local-kindle/chessnut-move/improvements-deployment.private.json`.

- Installed app, worker, KindleBT DSO and dependency-file hashes match the validated stage.
- Every pre-update puzzle/game/state hash remains unchanged.
- Private config still has root ownership/mode 600, parent root/mode 700; no
  insecure ordinary-state copy exists.
- App loader PASS; native preflight PASS for identity/privilege transition, actual
  dependencies and required symbols. Session NOT_OPENED; connection NOT_ATTEMPTED.
- No app or board worker remains running. No board connection or motor command
  was performed by the agent. Open the library app for the interaction checklist.
- No commits or pushes. Human acceptance of the new interactions remains pending.

The tested build/package receipt is `target/kindle/improvements-verification.json`.

## Send-and-forget change — 2026-11-06

The user requested removal of DONE and accepted immediate return after transport
submission. Removed DONE/verification states and controls, stale-source waiting
and automatic post-send commands. Each later explicit send is separately armed
and serialized. Persistent connection and cleanup behavior remain. Focused core,
platform, native and renderer checks cover the new flow, including service
acknowledgement after each submission and native rearming without STOP.

Validation: full host gate PASS (fmt, clippy with warnings denied, 256 Rust
tests and Python/lifecycle/packaging contracts); preserved support gate PASS
(20 Rust and 11 Python/native tests). Pinned ARM build/header/ELF checks PASS.
Reviewed 20 new state snapshots across STANDARD/SMALL and board/Settings;
hit tests cover both orientations and unchanged Review.

Package/deployment results follow below. No commits or pushes. No
agent-triggered board connection or motor commands.

### Send-and-forget deployment results

Installed on 2026-11-06 after confirming the app and worker were closed. A fresh
backup preserves the previous DONE-era runtime and matching scriptlet; previous
backups are retained. The exact backup path is in ignored
`local-kindle/chessnut-move/send-forget-deployment.private.json`.

- Actual staged manifest/private-source scan and host install, uninstall and
  backup restoration PASS with fixture data hashes unchanged.
- Installed app, native worker, KindleBT DSO and dependency-file hashes match stage.
- All fresh pre-update puzzle/game/state hashes remain unchanged.
- Private config remains root/mode 600 with root/mode-700 parent; no insecure
  ordinary-state copy exists.
- App dynamic loader and native preflight PASS, including UID/GID, dependencies
  and required symbols. Session NOT_OPENED; connection NOT_ATTEMPTED.
- No app or board worker remains running; no agent-triggered board connection or
  motor command. Open the library app for human interaction acceptance.

Receipts: `target/kindle/build.json` and
`target/kindle/send-forget-verification.json`. No commits or pushes.

## Task 57 — inbound board input (2026-11-06)

This extends the manual-send MVP under
`tasks/improvements/57-chessnut-board-input.md`. After Normal, the service requests
realtime without STOP, retries only that query every second while waiting for the
sent target, and forwards complete post-send positions after 500 ms of no change.
Identical reports do not postpone settlement; partial reports reset the candidate.
Only after the exact sent target is seen does core recognize a subsequent move.

Recognition is a physical placement difference checked against existing Board
semantics: ordinary moves, captures and explicit promotion replacement. Piece
lifts, capture removal, promotion pawns awaiting replacement and ambiguous or
compound changes do not grade. Castling/en passant and castle-shaped intermediate
rook/king moves are excluded; this is not a legality engine. Quiet time is a
heuristic for manual placement settlement, not a motor-finished signal.

Core applies normal grading, wrong rollback, automatic logical opponent replies
and solved progress. Wrong moves and replies wait for physical correction to the
Kindle placement before accepting another solver move; echoes never grade. Motor
replies remain manual via Send. Navigation, analysis, manual input, sleep and
connection errors invalidate listening. Flip and Settings preserve it. Review and
analysis previews never accept board input. Runtime now saves progress effects
from incoming reports and calculates damage using the same path as touch.

The new realtime-only enable behavior during/after auto-motion is not physically
verified. Manufacturer documentation says FEN reports are disabled during
auto-moves; software tests cannot establish resumption without STOP. The app
shows waiting until it actually observes the target, with no DONE fallback or
automatic motor commands. Human verification is required for resumption, move
settlement timing, capture/promotion inventory behavior, corrections and replies.

Automated gates PASS: 265 Rust tests on the final source, fmt/Clippy, all Python/
lifecycle/packaging checks, 20 support Rust and 11 Python/native tests, real pinned
header contracts and ARM ELF checks. Reviewed 32 session/input snapshots. Actual
stage, private-source scan and host install/uninstall/backup restoration PASS
with fixture user data unchanged. Deployment is recorded below. No commits
or pushes, agent board connection or agent motor tests are authorized here.

### Task 57 deployment results — 2026-11-06

Installed through the existing SSH/scp path after confirming the app/worker closed.
A fresh backup preserves the previous send-and-forget runtime and matching
scriptlet. Previous backups remain. The exact path is in ignored
`local-kindle/chessnut-move/board-input-deployment.private.json`.

- Installed app, worker, KindleBT DSO and dependency-file hashes match stage.
- All fresh pre-update puzzle/game/state hashes remain unchanged.
- Private config remains root/mode 600, parent root/mode 700, with no insecure
  ordinary-state copy.
- App loader and native preflight PASS (UID/GID, dependencies and symbols).
  Session NOT_OPENED; connection NOT_ATTEMPTED. No app/board worker remains.
- No agent-triggered board connection or motor command. Physical resumption,
  settlement timing and move handling remain the Task 57 human checkpoint.

Receipts: `target/kindle/build.json`,
`target/kindle/board-input-verification.json`. No commits or pushes.

## Task 58 — two-way replies and restored puzzle text (2026-11-06)

The user reports Task 57 works great. This is general board-input acceptance, not
an itemized physical-inventory/timing checklist. The new scope removes every board
communication overlay and its text-area hit mask, preserving normal notes/topic/
analysis, correct/wrong/complete feedback and Settings connection controls.

An accepted physical solver move that advances through an authored opponent reply
authorizes exactly one Normal target of Kindle's resulting position. Core enters
Sending and waits for the exact target before accepting another solver move.
Wrong moves, Free Board, terminal solver plies without replies, previews and touch
actions do not authorize automatic targets. Progress/grade state remains logical
truth on transport failure; no automatic motor retry or replay occurs.

The binary now routes Board effects from asynchronous positions as well as touch
through the same helper, preserving progress effects even on send failure. Tests
cover reply/echo/next-move round trips, promotion and invalid replies, completed
progress with an automatic reply, transport errors, unobstructed text pixels and
analysis links. Reviewed 32 updated state snapshots; the input-state set explicitly
shows puzzle descriptions throughout waiting/listening/correction.

Full host gate PASS (271 Rust tests plus fmt/Clippy/Python/lifecycle/packaging),
support gate PASS (20 Rust and 11 Python/native tests), pinned ARM/header/ELF
checks PASS. Package and deployment results follow after validation. Agent tools
will not connect to or move the board. Human verifies the automatic physical reply
and next-move reconciliation; no commits or pushes.

### Task 58 package handoff — deployment pending

Actual stage manifest and private-source scan PASS. Host install/uninstall and
backup restoration PASS with fixture data preserved. Source/build/package hashes
are recorded in `target/kindle/two-way-verification.json`. SSH timed out and then
reported No route to host; no deployment or board action was performed. The
previous installed Task 57 version remains unchanged. User readiness was requested.
Deploy only after checking the app/worker are closed, taking a fresh runtime/
scriptlet backup and data baseline, and validating the tested source receipt.
Human automatic-reply acceptance remains pending. No commits or pushes.

### Task 58 deployment — 2026-11-07

The user requested deployment after choosing package-only on 2026-11-06. Validated
the unchanged tested source/build/package receipts and stage manifest. After
confirming the app/worker were closed, preserved the previous runtime and matching
scriptlet in a new backup and recorded fresh puzzle/game/state hashes. The private
backup path is in `local-kindle/chessnut-move/two-way-deployment.private.json`.

- Installed app, worker, KindleBT DSO and dependency-file hashes match stage.
- All pre-update puzzle/game/state hashes remain unchanged.
- Private config root/mode 600 and parent root/mode 700 remain correct; no insecure
  ordinary-state copy exists.
- App loader and native preflight PASS on Barolo 5.19.6, ARMv7/glibc 2.35, including
  UID/GID, actual dependencies and required symbols. Session NOT_OPENED;
  connection NOT_ATTEMPTED. No app or board worker remains running.
- No agent board connection or motor command. Open the library app for the
  automatic-reply physical checklist; this deployment is not physical acceptance.

Updated receipt: `target/kindle/two-way-verification.json`. Previous backups
retained. No commits or pushes.

## Task 59 — physical board seating (2026-11-07)

The user requests a physical seat setting and explicitly says not to deploy.
Added a global optional version-1 `physical_board_seat` field with typed values
white/black/changing; old files default to changing to preserve existing behavior.
Puzzles Settings cycles WHITE SIDE / BLACK SIDE / CHANGING SIDE. A fixed seat
rotates physical square positions 180° if it differs from the original puzzle
solver color; colors and logical boards remain unchanged. Incoming raw reports
are normalized before grading, and automatic replies use the same captured
transform. Preview turn and Kindle Flip/lock never select physical rotation.

Changing seating emits only SettingsChanged, preserves board/progress/orientation
and pauses the listener until a new explicit Send. Review has no seating control
and its pixels/layout remain unchanged. The new seventh Puzzles Settings row
meets minimum touch sizes in both board sizes and display orientations.

Focused tests cover all six seat/color combinations, settings migration/roundtrip/
invalid input, asymmetric rotations, capture, promotion, correction, preview/
review isolation, SDK command/sensor round trips and automatic reply targets.
Reviewed all 12 seat/size/orientation Settings snapshots; other changed goldens
are confined to Puzzles Settings, including connection/input states. Full host gate PASS (278 Rust tests, fmt/Clippy and all Python/contracts),
support gate PASS (20 Rust and 11 Python/native tests), pinned ARM/header/ELF/
dependency checks PASS. Package validation follows below. No Kindle contact or deployment is authorized.

### Task 59 package-only results — 2026-11-07

All 278 Rust tests and host fmt/Clippy/Python/lifecycle/packaging checks PASS.
Support gate PASS (20 Rust, 11 Python/native). Pinned ARM release/header/ELF/
dependency checks PASS. Stage/manifest/private-source scan PASS; actual host
install, uninstall and backup restoration preserve all fixture user-data hashes.
Tested source and staged receipts match.

Package: `target/kindle/kindle_chess_0.1.0_kindlehf.kpkg`; receipt:
`target/kindle/physical-seat-verification.json`. No Kindle contact, deployment or
motor commands. The previously installed Task 58 version is untouched. Task 59
physical acceptance awaits future deployment authorization. No commits or pushes.

### Task 59 authorized deployment — 2026-11-07

The user subsequently requested deployment with the Kindle ready. Validated the
unchanged tested-source/package hashes and stage manifest, confirmed app/worker
were closed, and saved a fresh runtime/scriptlet backup and user-data baseline.
All installed runtime/scriptlet hashes match stage; all pre-update puzzle/game/
state hashes remain unchanged. Root-only configuration permissions, app loader
and native preflight PASS on ARMv7/glibc 2.35. Session NOT_OPENED; connection
NOT_ATTEMPTED. No agent board connection or motor command, and no app or worker
left running. Physical seat/move/reply acceptance remains pending.

Backup and deployment record: ignored
`local-kindle/chessnut-move/physical-seat-deployment.private.json`.
Updated receipt: `target/kindle/physical-seat-verification.json`.
Previous backups retained. No commits or pushes.
