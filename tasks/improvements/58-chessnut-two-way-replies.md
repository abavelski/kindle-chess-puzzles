# Task 58 — Automatic puzzle replies and unobstructed puzzle text

Status: Ready for human verification (software and ARM checks passed)

Dependencies: Task 57 deployed board input; user reports it works great on
2026-11-06. That is general acceptance, not an itemized inventory/move checklist.
Continue on feature/chessnut-move-mvp. No commits or pushes.

## Scope and contract

- Restore the text panel completely to its prior puzzle topic/description/analysis
  behavior. Remove all board communication overlays and their hit-test masks,
  including sending/listening/correction/error notices. Keep connection controls
  in Settings, Send state and normal correct/wrong/complete board feedback.
- After explicit Send and observed target agreement, accept settled physical
  solver moves as before. When an accepted physical move advances the authored
  solution through an automatic opponent reply, emit exactly one Send of the
  resulting canonical Kindle placement. No additional sync tap.
- Submit via the same owner-thread Arm/Normal/realtime sequence. Pause move input
  during submission and until the exact reply target is observed. Motor echoes,
  partial positions and repeated reports never grade or trigger another reply.
- Wrong moves, Free Board edits, terminal solver moves without a reply, touch
  actions, analysis and Review never initiate automatic board movement. Existing
  navigation/manual-edit/sleep/error invalidation still applies. Errors never
  retry/replay; logical grading/progress stays authoritative.
- Route Board effects produced by asynchronous position updates to BoardService,
  preserving ordinary progress effects and damage tracking. Use one shared effect
  router for touch and board input; test dispatch success and failure on host.
- No new protocol, engine, piece-inventory or compound-move guarantees. No DONE.

## Acceptance

Test-first reply target/effect, duplicate/intermediate/echo rejection, full
solver-reply-solver sequence, wrong/Free Board/final/promotion/invalid reply cases,
progress and error isolation. Test runtime routing and restored text/analysis hit
areas across connection/input states and both board sizes; inspect snapshots.
Run full host/support gates, pinned ARM build, stage/private-source scan and
install/rollback preservation checks. Deploy with fresh runtime backup and data
hash baseline if Kindle app is closed. Do not connect to or move the board from
agent tools; human verifies automatic motion and reply reconciliation.

## Automated verification — 2026-11-06

Full host gate PASS: fmt, Clippy with warnings denied, 271 Rust tests, snapshots
and all Python/lifecycle/packaging contracts. Preserved support gate PASS: 20 Rust
and 11 Python/native tests. Pinned ARM build/header/ELF/dependency checks PASS.
Reviewed 32 updated snapshots, including descriptions throughout all listener
states, and pixel/hit tests for unobstructed notes/topic and analysis links.

Human checkpoint: initial Send, physical solver move, automatic authored reply
on the board, target agreement, next solver move and completion. Check wrong
moves/no reply, disconnect/error/sleep recovery and absence of duplicate motion.
No agent board connection or motor tests. No commits or pushes.

## Package/deployment handoff — 2026-11-06

Stage/manifest/private-source scan and actual host install, uninstall and backup
restoration PASS with fixture user-data hashes unchanged. Package:
`target/kindle/kindle_chess_0.1.0_kindlehf.kpkg`; receipt:
`target/kindle/two-way-verification.json`.

Deployment not performed: SSH first timed out, then reported No route to host.
User readiness was requested asynchronously. The installed Task 57 version is
unchanged. After the Kindle is reachable and app/worker closed, take a fresh
runtime/scriptlet backup and data hash baseline, deploy, verify installed hashes
and loader/native preflight without connecting to the board, then hand off the
automatic-reply physical checklist. No commits or pushes.

## Deployment — 2026-11-07

The user explicitly requested deployment after the package-only handoff. The
Kindle was reachable and the app/worker closed. Installed the unchanged validated
Task 58 stage after taking a fresh runtime/scriptlet backup and current puzzle/
game/state hash baseline. Previous backups remain retained. Exact backup path:
ignored `local-kindle/chessnut-move/two-way-deployment.private.json`.

Installed app/worker/KindleBT/dependency-file hashes match stage; all pre-update
puzzle/game/state hashes are unchanged. Private config remains root/mode 600
with root/mode-700 parent and no insecure ordinary-state copy. App loader and
native preflight PASS on Barolo 5.19.6 (UID/GID, dependencies and symbols).
Session NOT_OPENED; connection NOT_ATTEMPTED. No app/board worker remains running.

No board connection or motor command was triggered by agent tools. The Task 58
automatic-reply physical checklist remains pending. Receipt:
`target/kindle/two-way-verification.json`. No commits or pushes.
