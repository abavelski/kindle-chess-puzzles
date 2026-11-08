# Task 59 — Physical board seating and solver-facing positions

Status: Ready for human verification (deployed on 2026-11-07 after explicit user authorization)

Dependencies: Task 58 two-way board integration. Work on the existing
feature/chessnut-move-mvp branch. User explicitly says do not deploy to Kindle.
No commits or pushes; no Kindle connection or motor tests.

## Scope

Add a durable, global physical-board seating choice in Puzzles Settings:
WHITE SIDE, BLACK SIDE, CHANGING SIDE (cycle in that order). Default to CHANGING
SIDE for existing files, preserving current wire coordinates. This belongs to
physical seating, independent of workspace presentation preferences.

| Seat | White solver | Black solver |
| --- | --- | --- |
| White side | canonical | rotate 180 degrees |
| Black side | rotate 180 degrees | canonical |
| Changing side | canonical | canonical |

Rotate square positions only, preserving piece colors. Solver color always comes
from the puzzle's original FEN, not the current ply, preview turn or manual Flip.
The Kindle retains existing automatic solver orientation/lock/Flip behavior.

Initial Send captures the rotation with its canonical logical baseline. Normalize
incoming raw sensor placements through the inverse rotation before matching,
move inference and grading. Automatic reply targets use that same captured
rotation. Promotions are recognized only after conversion to canonical squares.
Rotation is self-inverse, including captures and asymmetric arrangements.

Changing the seat saves Settings only, pauses the listener, and never moves the
board or modifies a puzzle/grading/progress. A new explicit Send applies it.
Old/future/malformed settings protection and existing workspace profiles remain
compatible. Review controls and persistence stay independent.

## Acceptance

Red-to-green tests for all six combinations, exact asymmetric rotated placement
and round trip, typed version-1 settings defaults/roundtrip/invalid input, Settings
persistence effect and isolation, raw rotated input and automatic replies across
solver colors, promotion/capture, correction/duplicate handling, flip independence
and changed-setting rearm requirements. Renderer cycle/hit/minimum-touch tests
and reviewed snapshots for every seat and both board sizes/orientations. Keep
Review geometry unchanged and retain all text/analysis content.

Run full host/support gates, pinned ARM build and stage checks plus actual host
install/uninstall/backup restoration and private-source scan. Do not deploy or
connect to the Kindle. Physical seat/move/reply verification remains pending for
an eventual user-authorized deployment.

## Automated validation — 2026-11-07

Full host gate PASS: fmt, Clippy with warnings denied, 278 Rust tests and all
Python/lifecycle/packaging contracts. Support gate PASS (20 Rust and 11 Python/
native tests). Pinned ARM/header/ELF/dependency build PASS.

New core tests cover all six seat/solver combinations, capture/promotion, wrong
correction, previews, original-root-color selection, Flip, persistence and Review
isolation. Real SDK packet/sensor round trips cover six combinations and reply
targets. Reviewed 12 seat/size/orientation Settings snapshots; other golden
changes are confined to Puzzles Settings. Review and ordinary puzzle frames are
unchanged. Physical acceptance awaits a future authorized deployment.

No Kindle contact, deployment, motor commands, commits or pushes.

## Package-only handoff — 2026-11-07

Actual staged manifest, private-source scan and host install/uninstall/backup
restoration PASS with fixture puzzle/game/state hashes unchanged. The build
receipt still matches tested source. Package:
`target/kindle/kindle_chess_0.1.0_kindlehf.kpkg`; verification receipt:
`target/kindle/physical-seat-verification.json`.

No Kindle contact or deployment, as explicitly requested. The device retains its
previous installed Task 58 runtime. No motor commands, commits or pushes. Future
physical verification must cover all seat/color combinations, incoming moves and
automatic replies, especially promotion/capture and changing seats mid-session.
After choosing a seat, a new explicit Send must reacquire the physical baseline.

## Authorized deployment — 2026-11-07

The user subsequently confirmed the Kindle was ready and explicitly requested
deployment. Validated unchanged tested-source/package hashes and staged manifest;
confirmed app and worker were closed; preserved the previous runtime/scriptlet
and recorded puzzle/game/state hashes before installing.

Installed runtime and scriptlet hashes match the tested stage. All existing
puzzle/game/state hashes are unchanged. Secure configuration permissions,
ARM/glibc loader and native dependency/symbol/privilege preflight PASS. Session
NOT_OPENED; connection NOT_ATTEMPTED. No app or worker left running. No agent
board connection or motor command; physical seating acceptance remains pending.

Private backup/receipt: `local-kindle/chessnut-move/physical-seat-deployment.private.json`.
Verification receipt: `target/kindle/physical-seat-verification.json`.
No commits or pushes. Open the library app, choose PHYSICAL BOARD in Puzzles
Settings and press Send to apply it; verify incoming moves and automatic replies.
