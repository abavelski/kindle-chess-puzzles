# Chessnut Move — manual position-push MVP

**Status:** Planned; this file is a plan, not an implementation or a device PASS.
**Target:** Existing Kindle Chess Puzzles on the verified first-generation Kindle Scribe (Barolo), firmware 5.19.6.
**Implementation branch:** Create a new `feature/chessnut-move-mvp` branch from current `main` for code and tests. Commit this plan directly to `main`; **do not merge or cherry-pick the probe branch**.

## Goal and UX contract

Add an opt-in, one-way action to set the Chessnut Move to the position currently **visible** in the Kindle **Puzzles** workspace.

1. **Settings row:** Add `CHESSNUT MOVE  DISCONNECTED` to the existing Settings dialog. Tapping when disconnected/error starts connection; tapping when connected and idle disconnects. Show `CONNECTING`, `CONNECTED`, `SENDING / AWAITING DONE`, `VERIFYING`, or `ERROR` as appropriate. Display an actionable error on missing private address, timeout, or dropped worker. While a position command is outstanding, the row must not imply that disconnect/STOP physically cancels movement.
2. **Toolbar button:** Only in Puzzles, while the board session is actually connected, ready, and idle, show an icon-only **Send position** control immediately **to the right of Analysis**. Hide it (and its hit target) otherwise. Keep Review mode unchanged. Respect both STANDARD and SMALL layout sizes and their minimum touch targets.
3. **Tap to send:** Snapshot the **displayed** `AppState::board()` placement at the tap, including any analysis preview or Free Board edits; do not silently use `live_board()`, the puzzle's initial FEN, or a flipped coordinate system. If promotion is pending, reject the request. Present the target and a clear **Confirm / Cancel** step before motor motion; confirmation authorizes exactly that snapshot. A preview is read-only and must not alter grading or progress.
4. **Finish:** Send one Normal-mode command only after preconditions and confirmation. Show pending state until the human explicitly chooses **Done**, then perform the probe-proven STOP (best effort), re-enable realtime telemetry and require **three consecutive complete, exact target placements** before reporting synchronized. On mismatch/error/disconnect, show **UNVERIFIED / OUT OF SYNC**, clear authorization, and never automatically replay the command.

**Motor-safety wording:** STOP, disconnect, exit, and Ctrl+C are **not verified physical emergency stops**. The board may complete an already-issued move. Keep the board's travel area clear and retain its physical power-off/recovery route. No auto-send on startup, collection change, analysis selection, reset, navigation, reconnect, or wake.

## Implementation steps (small, test-first)

### 1. Adapt the proven board service

- Reuse the code and pins in [`support/chessnut-move/`](../support/chessnut-move/README.md) and the [integration handoff](CHESSNUT_MOVE_INTEGRATION_HANDOFF.md); the reference is **not** a ready-to-run app daemon. Add a production adapter in the Kindle/platform/runtime layer, not BLE imports into `chess-core` or `chess-render`.
- Preserve the Rust controller + separate minimal C KindleBT/ACE worker and verified Bluetooth UID/GID. Use versioned, bounded IPC, finite timeouts, one outstanding operation, callback-matched connection ownership, strict GATT discovery, and cleanup on exit/error. Do **not** assume BlueZ/btleplug or add a scanner.
- For each connection: connect → discover → request MTU 500 and await matched success (probe observed 300; require enough for complete 38-byte position reports) → subscribe with completed CCCD writes → enable realtime. Only mark `CONNECTED` once the session is genuinely ready. Use complete fresh sensor placement as the known physical source; fail closed if missing/stale or if the board changes between preview and confirmation.
- Use the pinned `chessnut-move` 1.0.2 protocol encoder (`default-features = false`) to convert the captured placement. Transmit exactly one **35-byte Normal** (`42 21 … 01`) write **WithoutResponse**; no Force, truncation, ad-hoc splitting, automatic retry, or overlapping motor commands. Handle undecoded responses without treating them as movement proof.
- Configure the board address via a **private, locally provisioned Kindle-side config** (outside committed sources/ordinary `settings.json`); document provisioning from the ignored `local-kindle/chessnut-move/devices.private.json` when available. Keep addresses out of Git and routine logs. No new address-picker/scanning UI in this MVP.

### 2. Wire the existing UI without coupling it to BLE

- `crates/chess-core/src/app.rs`: expose only platform-neutral, transient board-session state and intent/effect(s) for connect, disconnect, position preview/confirm/cancel and verification result. Keep existing puzzle grading and `progress.json` untouched. Treat this row as **live status**, not a persisted workspace preference; keep `settings.json` version 1 compatible.
- `crates/chess-render/src/layout.rs`, `button.rs`, `render.rs`: add the sixth settings row and its hit target; fit it above Close without overlap. Insert a seventh optional toolbar slot immediately after Analysis in Puzzles; extend icon drawing, visibility, hit testing and the confirmation/Done UI. Review toolbar remains as before.
- `app/kindle-chess/src/main.rs` + Kindle platform: wire effects to the board adapter. The current `input.next_event` loop blocks; provide an event/poll/wake path for asynchronous worker state updates so connection, timeouts, telemetry and verification never freeze touch UI. Handle sleep/wake/exit with cleanup and **no automatic replay**.
- Update the scoped build, staging and packaging scripts to include the native worker and redistributable KindleBT dependencies/notices. Preserve pinned ARMv7 hard-float/glibc ≤2.35 builds, MTU/descriptor fixes, ELF/COPY-relocation/NEEDED checks, and existing app paths/data. Do not install/replace system Bluetooth libraries.

### 3. Acceptance checks

- **Host (red → green):** Fake-worker tests for connect/disconnect/error/timeout/stale callback and lost worker; target snapshot from normal board, analysis preview and Free Board; Confirm/Cancel; only one 35-byte Normal write; missing/partial telemetry, changed source, in-flight rejection, no retry/reconnect replay, and three-report verification. Assert no progress, collection, review or persisted settings changes from preview/send.
- **Render:** Snapshot and hit-test Settings states, added row, icon adjacency/visibility, Confirm/Done/error UI and both board sizes. Check minimum touch geometry and no unintended Review button.
- **Build:** Run `scripts/check.sh`, `sh support/chessnut-move/scripts/check.sh`, ARM stage/ELF/dependency checks and a clean package/install rollback check. Do not call host mocks a hardware PASS.
- **Supervised device:** First prove connect → ready → disconnect → reconnect without a write. Then check a **known, simple pawn target** with a freshly observed physical source, explicit confirmation, human Done and three exact post-motion reports; test cancellation before confirmation and error/disconnect recovery. Confirm the old puzzle app behavior, progress and sleep/wake path are unaffected. Record actual results separately.

## Deliberately out of scope

Board-driven taps/move recognition or grading; automatic opponent replies; background synchronization; general puzzle setup, captures, castling, promotion, parking or multi-piece motion guarantees; autonomous motion-completion detection; board scanning/pairing UI; general Game Review integration; and claims that STOP reliably interrupts motors. Those need separate tasks and physical acceptance.

**References:** [Probe plan](CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md) · [Validated handoff](CHESSNUT_MOVE_INTEGRATION_HANDOFF.md) · [Reference sources](../support/chessnut-move/README.md) · [Settings behavior](SETTINGS.md).
