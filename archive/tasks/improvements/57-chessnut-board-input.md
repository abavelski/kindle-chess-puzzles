# Task 57 — Chessnut moves as puzzle input

Status: Implemented (general user acceptance reported 2026-11-06)

## Scope

User request: after explicitly sending a visible position, listen to the physical
board and apply completed moves on Kindle. Continue on feature/chessnut-move-mvp;
no commits/pushes. Preserve send-and-forget: no DONE, motor completion claims,
automatic motor retries or automatic opponent motor replies.

## Contract

- After Normal, request realtime telemetry without STOP. While waiting to observe
  the sent target, repeat only the realtime request at bounded intervals. This
  re-enable behavior during/after motion requires physical verification; the
  manufacturer documents reporting disabled during auto-motion, not completion.
- Forward only complete 38-byte placements received after the latest submission,
  debounced for 500 ms without a change. Identical repeated reports do not postpone
  settlement; partial/malformed observations invalidate the candidate. No timer
  alone declares motor completion. Observe the exact sent target before input.
- Infer only one unambiguous move representable by existing Board UCI semantics:
  ordinary moves, captures and explicit promotion replacement. Ignore lifts,
  capture removals and ambiguous/multi-piece changes. Castling/en-passant are
  outside the existing runtime move model; do not guess their intermediate states
  as ordinary moves. No legality engine.
- Apply through the existing grading/Free Board paths. Wrong move feedback rolls
  back Kindle and waits for physical correction. Logical opponent replies remain
  on Kindle; wait for that exact position before recognizing another solver move.
  Physical correction/reply echoes cannot grade or solve a puzzle.
- No input before Send, while submitting, in Review, analysis previews, collection/
  goto pickers or promotion dialogs. Navigation/reset/mode changes, manual board
  input, sleep, disconnect/error invalidate the listener; a new Send rearms it.
  Flip and ordinary Settings do not rotate or alter sensor coordinates.
- Listening is transient and separate from connection state. Render monochrome
  waiting/listening/match-Kindle notices without blocking Send or adding DONE.
- Runtime persists ordinary progress effects from board moves and calculates board
  damage just as for touch. Uploaded data, settings and review state remain intact.

## Acceptance

Test-first core inference, normal/capture/promotion/intermediate/special cases,
arming, corrections, automatic replies, stale/navigation/preview isolation and
progress effects. Platform tests cover report completeness, debounce/change-only
notifications, buffered pre-send exclusion, fresh-send reset, query-only refresh,
worker errors and real service transport. Native tests prove realtime allowed
while motion_maybe, preserving one-shot Arm, serialized writes and cleanup.
Inspect snapshots for every new notice and both sizes. Run full host/support gates,
pinned ARM build/stage and install/rollback data preservation tests.

Human checkpoint: verify notifications resume without STOP/DONE after Send, target
baseline acquisition, physical quiet move/capture/promotion timing, wrong-move
correction, logical replies, UI/navigation/sleep isolation and persistent progress.
Agent deployment/preflight must not connect to the board or issue motor commands.

## Automated verification — 2026-11-06

Full host gate and final latest-source fmt/Clippy/workspace rerun PASS (265 Rust
tests and Python/lifecycle/packaging contracts). Support gate PASS (20 Rust and
11 Python/native tests); pinned ARM build and real-header native contracts PASS.
Reviewed 32 board-session/input-state snapshots; actual stage manifest/private
scan/install/uninstall/backup restoration/data-preservation checks PASS.
The extra mode-change invalidation regression is covered. Physical report
resumption and move recognition remain pending; do not call this hardware PASS.

## Deployment — 2026-11-06

Installed on the recorded Scribe with a fresh previous-runtime/scriptlet backup.
Installed artifact hashes, unchanged puzzle/game/state hashes, private config
permissions, app loader and native preflight PASS. No Bluetooth session or motor
command was attempted by the agent. Verification receipt:
`target/kindle/board-input-verification.json`. No commits or pushes.

The task remains at the human checkpoint: Send → observe waiting/listening →
move a piece → confirm normal grading/Free Board behavior. Verify realtime report
resumption without STOP/DONE first; capture/promotion timing, corrections/replies,
navigation/sleep and durable progress follow. Record observations before marking
the physical checkpoint complete.

## User acceptance — 2026-11-06

The user reports board input works great and requests the next extension: remove
text-panel communication messages and send authored replies automatically. This
closes the general Task 57 handoff; it does not assert individual inventory,
promotion or timing checklist results. Task 58 supersedes the manual-reply and
communication-notice parts of this contract.
