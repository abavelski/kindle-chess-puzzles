# Chessnut Move / Kindle BLE handoff — 2026-11-05

Resume from this file and the user's next complete `read-board.sh` report.

## Scope and status

- Repository: `/Users/aba/dev/kindle-chess-puzzles`.
- Dedicated branch: `experiment/chessnut-move-kindlebt-probe`. Commit only here;
  do not move `main`. Changes stay inside `experiments/chessnut-move-ble-probe/`.
- Read repository AGENTS.md and its required documents, then
  `docs/CHESSNUT_MOVE_KINDLEBT_PROBE_PLAN.md` before further implementation.
- M0–M2 implemented; H1/H2 human-accepted, including Kindle discovery/reconnect.
  M3 implementation exists, but **H3 remains incomplete**. No M4/motor work.
- User wants to control connection timing: deploy fixes when needed, then let
  them enable board Bluetooth and run the script themselves. Do not start a
  background connection, Mac scan, or repeated button/reboot cycle.

## Latest device evidence and fix

The user's report at device time **2026-11-05 17:49:45**, from source `9c88bc2`,
connected in **205 ms** with successful API/callback/context/handle evidence.
Discovery matched the expected Chessnut services and characteristic properties.

Position subscription API returned 0; response subscription and position
unsubscribe returned **-15**. The daemon explicitly logged **Pending GATT
Operation**. The probe issued another CCCD descriptor write before the previous
one completed. No telemetry queries ran (`write_api_calls=0`); no position,
battery/piece status or MTU evidence was obtained. Cleanup reported FAIL, although
disconnect/GATT/BLE/session API returns were 0; the operation lock cleared.

Fix **`0e51faf`**, now deployed:

- Serialize notification enable/disable using the public descriptor-write
  completion callback, matched by connection and discovered CCCD handle.
- Log API acceptance and descriptor completion separately. Wait at most five
  seconds; missing/invalid completion blocks further GATT writes and falls back
  to disconnect with failed cleanup reported. No automatic retries.
- Pinned KindleBT omitted descriptor callback forwarding. Build generates a copy
  with one bridge include and one callback-table entry. The upstream cache and
  stock Kindle libraries stay untouched; the experiment's bundled library changes.
  See `native/kindlebt_descriptor_bridge.h` and `scripts/delivery.py`.

Installed source SHA256:
`a88a4b6a4a9b93224f3600d45af2a5c238a5ad79f490e5c65aca7655bde0cc7e`.
Deployment checksums/receipt, preflight and script setup pass. All 121 original
app/data files were unchanged. **The deployed fix has not been board-tested.**
Subsequent commits, including `76d86f6`, record results; they are documentation.

## User's next run

Close the official app, enable the board's blinking Bluetooth icon, then on Kindle:

```sh
sh /mnt/us/chessnut-ble-probe/runtime/read-board.sh
```

The script makes one connection attempt, discovers services, subscribes to both
notification streams, sends SDK realtime/battery/piece queries, observes for
20 seconds and disconnects. The user may move one piece by hand during observation.
**No LED or motor commands.** Telemetry queries and subscription descriptor writes
are BLE writes; do not describe this as sending no writes.

It prints a redacted report through `END REPORT`, also saved under
`/mnt/us/chessnut-ble-probe/logs/read-board-*.log`. Private board address is already
configured at `/mnt/us/chessnut-ble-probe/board-address`; do not print/commit it.
`read-board.sh --check` validates setup without opening a Bluetooth session.

In the next report, first verify both `subscribe_*_descriptor_callback` statuses,
then query write callbacks, notifications, decoded position/status, MTU and cleanup.
Do not assume success from connection/API acceptance alone. H3 additionally needs
human position/FEN comparison, battery/status and guarded LED on/off verification,
MTU review and acceptance. Motor/STOP packets remain blocked; 35-byte delivery is
UNVERIFIED.

## Validation and references

- Independent gate: 39 Rust tests, 23 Python tests; real-pinned-header adapter and
  bridge tests; ARM/ELF/glibc build and stage verification all PASS.
- Parent fmt/clippy and 237 workspace tests PASS. Workspace tests need existing
  `experiments/chessnut-move-ble-probe/target/host-check-venv/bin` on PATH for the
  documented host-only `python-chess` dependency.
- Upstream KindleBT remains pinned at
  `59e1d8e69a52e3cd58cee6fcc416d1774ad55791`; local bridge provenance is recorded in
  `notes/SOURCE_PINS.json` and the installed build receipt. SDK crate is
  `chessnut-move=1.0.2`; Rust 1.85.1, Zig 0.13, ARMv7 hard-float, glibc <=2.35.
- Full evidence: `notes/DEVICE_RESULTS.md`, `notes/CONNECTION_DEBUG.md`,
  `native/README.md`. Earlier connection failures included GATT 133/controller
  reason 0x3e; those did not recur in the latest user run and are not proven fixed.
- Ignored local helpers `target/run-m3-deploy.py` and `target/setup-read-board.py`
  handle known SSH access/private configuration and setup-only verification.
  User authorized root SSH with an empty password. Do not expose private captures.
- For a future code change: failing host test first, then check/build/stage/verify
  using the experiment scripts before deployment. New documentation affects the
  source digest: rebuild before another deployment instead of using a stale stage.
