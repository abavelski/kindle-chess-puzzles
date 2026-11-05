# Probe closed — integration planning handoff

The standalone probe was uninstalled from Kindle, including all probe-owned logs
and binaries. Raw local logs/build/download artifacts were removed after findings
were consolidated. The original app/data hashes were unchanged during removal.

Read [the integration handoff](../../../docs/CHESSNUT_MOVE_INTEGRATION_HANDOFF.md)
and [the curated device results](DEVICE_RESULTS.md). Private board/device identities
are saved locally in ignored `local-kindle/chessnut-move/devices.private.json`.

The user accepts that STOP/Ctrl+C did not reliably halt physical movement. Keep
that limitation explicit; no automatic replay, implicit movement or full-H4 PASS.

Keep the full probe source on `experiment/chessnut-move-kindlebt-probe`. Only docs,
source pins and the curated standalone `support/chessnut-move/` reference go to main.
Do not merge the experiment branch or modify the app until scoped integration
planning/tasks are prepared. No further standalone probe run is needed merely
to start coding the integration.
