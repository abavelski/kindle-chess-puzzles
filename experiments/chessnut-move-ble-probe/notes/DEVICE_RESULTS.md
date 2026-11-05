# Curated device results — 2026-11-05

Full technical findings, limits and planning requirements:
[Chessnut Move integration handoff](../../../docs/CHESSNUT_MOVE_INTEGRATION_HANDOFF.md).
Raw reports and chronological debugging diaries have been deleted after consolidation.

| Checkpoint | Result |
| --- | --- |
| H1 native load, privileges, original app | PASS |
| H2 address-based connection/discovery/disconnect/reconnect | PASS; intermittent stale daemon-session mapping remains documented |
| H3 positions, manual e6→e7, battery/pieces, green e4/all-off | PASS; MTU 500 requested, 300 negotiated; 38-byte positions, 139-byte piece response |
| H4 small move/return | Human confirmed e2→e4 and e4→e2 |
| H4 final-position verification | PASS after `done`/STOP/realtime: three reports matched e4 |
| H4 longer e2↔e7 STOP probe | Exercised; physical interruption not reliably demonstrated |
| H4 physical STOP / Ctrl+C | FAIL in reported tests; user explicitly accepts limitation |
| H5 complete reliability, sleep/wake and post-removal visual acceptance | NOT COMPLETED |
| Requested uninstall and hash independence | PASS: probe root/logs removed, no probe process/lock; 121 original app/data files unchanged |

Last uploaded e7-probe source digest:
`35ad29dda2dda0b9345a1503dcb604d77bed369cd8b5d5034b7ff1505a4b87c3`.
Hardware provenance and artifact/build metadata are preserved with the reusable
reference, not as binaries. Stack API/callback success does not prove physical
STOP or final arrival. No physical move was started by the agent while diagnosing,
archiving or uninstalling the experiment.

Private device identifiers and the small cleanup receipt remain under ignored
`local-kindle/chessnut-move/`, outside this experiment. No real address or raw log
is committed. Do not call the accepted STOP limitation a passed safety feature.
