# Chessnut Move — manual position-push MVP

**Status:** Task 59 physical seating is deployed on the Kindle after explicit user authorization on 2026-11-07. Host/native/ARM/package checks and installed hashes, user-data preservation, loader and native preflight passed. Physical seating acceptance is pending. All changes remain uncommitted on `feature/chessnut-move-mvp`. See [verification record](CHESSNUT_MOVE_MVP_VERIFICATION.md) and [Task 59](../tasks/improvements/59-physical-board-seat.md).
**Target:** Existing Kindle Chess Puzzles on the verified first-generation Kindle Scribe (Barolo), firmware 5.19.6.
**Implementation branch:** Continue on `feature/chessnut-move-mvp`; keep all changes uncommitted/unpushed as requested; **do not merge or cherry-pick the probe branch**.

## Goal and UX contract

Add an opt-in, one-way action to set the Chessnut Move to the position currently **visible** in the Kindle **Puzzles** workspace.

1. **Settings row:** Show DISCONNECTED initially, CONNECTING while opening, and DISCONNECT whenever connected, including during submission. Keep the connection across idle telemetry gaps, navigation and sleep/wake. Actual worker/link failure or app exit closes it.
2. **Toolbar button:** In Puzzles show Send immediately after Analysis; keep it selected/disabled only during transport submission. There is no DONE control or post-motion lock. Preserve Review and STANDARD/SMALL portrait/landscape touch geometry.
3. **Tap to send:** Capture the canonical visible `AppState::board()` placement, including analysis preview or Free Board edits, rejecting pending promotion. Submit one explicitly armed Normal command without confirmation or a fresh-source wait. Do not alter grading or progress.
4. **Return:** After transport submission, discard the target and return to connected/idle. Do not infer physical completion, issue STOP/realtime verification, queue a replay, or retry failures. Another explicit tap can send another target even without new telemetry. Keep requests serialized; app sleep/wake never initiates a target.

**Motor-safety wording:** STOP, disconnect, exit, and Ctrl+C are **not verified physical emergency stops**. The board may complete an already-issued move. Keep the board's travel area clear and retain its physical power-off/recovery route. No auto-send on startup, collection change, analysis selection, reset, navigation, reconnect, or wake.

## Implementation steps (small, test-first)

### 1. Adapt the proven board service

- Reuse the code and pins in [`support/chessnut-move/`](../support/chessnut-move/README.md) and the [integration handoff](CHESSNUT_MOVE_INTEGRATION_HANDOFF.md); the reference is **not** a ready-to-run app daemon. Add a production adapter in the Kindle/platform/runtime layer, not BLE imports into `chess-core` or `chess-render`.
- Preserve the Rust controller + separate minimal C KindleBT/ACE worker and verified Bluetooth UID/GID. Use versioned, bounded IPC, finite timeouts, one outstanding operation, callback-matched connection ownership, strict GATT discovery, and cleanup on exit/error. Do **not** assume BlueZ/btleplug or add a scanner.
- For each connection: connect → discover → request MTU 500 and await matched success (probe observed 300; require enough for complete 38-byte position reports) → subscribe with completed CCCD writes → enable realtime. Only mark `CONNECTED` once the session is genuinely ready. Require an initial complete sensor report before declaring connection ready. Subsequent sends do not require fresh source telemetry.
- Use the pinned `chessnut-move` 1.0.2 protocol encoder (`default-features = false`) to convert the captured placement. Transmit exactly one **35-byte Normal** (`42 21 … 01`) write **WithoutResponse**; no Force, truncation, ad-hoc splitting, automatic retry, or concurrent transport requests. Handle undecoded responses without treating them as movement proof.
- **Verified deployment constraint (2026-11-05):** `/mnt/us` is `fuse.fsp` and reports file mode 777 after `chmod 600`. Store the private config at `/var/local/kindle-chess/chessnut-move.private.json` instead (verified root-owned mode 600, directory 700); keep it out of `/mnt/us` packages and ordinary state.
- Configure the board address via a **private, locally provisioned Kindle-side config** (outside committed sources/ordinary `settings.json`); document provisioning from the ignored `local-kindle/chessnut-move/devices.private.json` when available. Keep addresses out of Git and routine logs. No new address-picker/scanning UI in this MVP.

### 2. Wire the existing UI without coupling it to BLE

- `crates/chess-core/src/app.rs`: expose only platform-neutral, transient board-session state and intent/effect(s) for connect, disconnect, direct send and transport result. Keep existing puzzle grading and `progress.json` untouched. Treat this row as **live status**, not a persisted workspace preference; keep `settings.json` version 1 compatible.
- `crates/chess-render/src/layout.rs`, `button.rs`, `render.rs`: add the sixth settings row and its hit target; fit it above Close without overlap. Insert optional Send immediately after Analysis in Puzzles; extend icon drawing, selected/disabled state, visibility, hit testing and status notices. Review toolbar remains as before.
- `app/kindle-chess/src/main.rs` + Kindle platform: wire effects to the board adapter. The current `input.next_event` loop blocks; provide an event/poll/wake path for asynchronous worker state updates so connection, timeouts, telemetry and transport results never freeze touch UI. Keep the connection across sleep/wake and clean up on exit with **no automatic replay**.
- Update the scoped build, staging and packaging scripts to include the native worker and redistributable KindleBT dependencies/notices. Preserve pinned ARMv7 hard-float/glibc ≤2.35 builds, MTU/descriptor fixes, ELF/COPY-relocation/NEEDED checks, and existing app paths/data. Do not install/replace system Bluetooth libraries.

### 3. Acceptance checks

- **Host (red → green):** Connect/disconnect/error/lost-worker and initial readiness tests; capture displayed normal/analysis/Free Board targets; exactly one Arm/Normal pair per tap; repeated explicit sends without STOP or fresh telemetry; failed write never retries; actor acknowledges each submission even when connected status is unchanged; no progress/settings mutation.
- **Render:** All current states, Settings DISCONNECT, selected/inert Send only during submission, no DONE slot, toolbar adjacency/minimum sizes, both sizes/orientations and unchanged Review.
- **Build:** Full host/support gates, pinned ARM/header/ELF checks, stage and install/rollback preservation checks.
- **Supervised device:** Connect and send a known target without confirmation or DONE. Check that Send unlocks after submission, a later explicit target sends on the same connection, idle/sleep does not disconnect or replay, and puzzle state remains unchanged. Do not treat unlocking as proof of motion completion.

## Deliberately out of scope

Board-driven taps/move recognition or grading; automatic opponent replies; background synchronization; general puzzle setup, captures, castling, promotion, parking or multi-piece motion guarantees; autonomous motion-completion detection; board scanning/pairing UI; general Game Review integration; and claims that STOP reliably interrupts motors. Those need separate tasks and physical acceptance.

**References:** [Probe plan](CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md) · [Validated handoff](CHESSNUT_MOVE_INTEGRATION_HANDOFF.md) · [Reference sources](../support/chessnut-move/README.md) · [Settings behavior](SETTINGS.md).

## Earlier accepted DONE workflow — superseded on 2026-11-06

The user reports the deployed MVP was verified and works. This is a general user
acceptance report, not an item-by-item assertion for every original checklist.
The user now authorizes immediate send without confirmation and accepts a
separate DONE fallback because autonomous completion has not been established.

Replace preview/Confirm/Cancel with one explicit Send of the visible placement.
Keep the Send icon visible, selected and disabled through arranging/verification;
show a separate DONE control after a command is issued. DONE runs the proven
STOP/realtime/three-report verification. No local write acknowledgement or timer
counts as completion. Keep the connection across idle telemetry gaps, normal
navigation and sleep/wake; Settings shows DISCONNECT while connected. Actual link
loss, worker/protocol failure and app exit still require cleanup. Stale source
telemetry must be refreshed before sending. Logical verification/mismatch errors
retain the connection and block new targets until reconciled or disconnected.
No automatic reconnect/replay, no motor commands from the agent, and no commits
or pushes. Implement these changes with host tests, snapshots and ARM validation.

## Superseding request — send and forget (2026-11-06)

The user explicitly removes DONE and post-motion verification. Each tap submits
one Normal target, then returns to connected/idle after transport submission.
There is no motion-completion claim, STOP/realtime verification sequence, stale
source wait, or automatic retry. A later explicit tap can submit another target
without telemetry or a disconnect. Serialize transport requests and reject repeat
taps while submission is in flight. Retain initial connection readiness, native
one-shot Arm permission per tap, MTU/callback checks, error/exit cleanup, private
config and persistent connection. Native motion_maybe remains a conservative
cleanup flag, not a lock requiring STOP before another explicitly armed target.
Remove obsolete DONE UI/state/tests, verify red-to-green host/native/snapshot
checks and ARM packaging, then deploy without triggering board motion. No commits
or pushes. Earlier DONE workflow sections are historical and superseded.

## Board-input extension — 2026-11-06

The user now authorizes inbound physical move recognition after explicit Send.
Task 57 (`tasks/improvements/57-chessnut-board-input.md`) is the scope boundary.
This supersedes the earlier exclusion of board-driven input, but retains manual
outbound motor commands, send-and-forget, no DONE, no runtime legality engine and
no commits/pushes. Realtime-only re-enable after Normal is an unverified hardware
assumption to expose at the human checkpoint, never hide with automatic STOP.

## Task 58 — automatic replies and puzzle text (2026-11-06)

The user reports Task 57 works great, requests no board communication in the text
panel, and explicitly authorizes sending the authored opponent reply after an
accepted physical solver move. Task 58 is the scope boundary. One reply target is
submitted without a sync tap; input waits for observed target agreement before
the next solver transaction. No motor retries, automatic wrong-move correction,
engine or DONE. The text panel retains ordinary puzzle/topic/description/analysis
behavior and hit areas. Runtime routes both touch and incoming-position Board
effects through one tested path, keeping progress effects on transport errors.

## Task 59 — physical seating (2026-11-07)

The user requests WHITE SIDE / BLACK SIDE / CHANGING SIDE seating and explicitly
prohibits deployment. Task 59 is the scope boundary. Fixed-seat mismatches rotate
target positions 180° without swapping piece colors; CHANGING SIDE preserves
existing coordinates. Rotation is captured from the puzzle's original solver
color, applied consistently to initial targets, raw input and automatic replies,
and independent of Kindle Flip. Settings changes pause input and persist only a
preference; the next explicit Send applies it. Host/ARM/package validation only,
with no Kindle connection, deployment, motor test, commit or push.
