#!/bin/sh
# Minimal FBInk smoke test for Task 00.
# This DOES write a small temporary message to the framebuffer, but it does not
# stop services, change rotation, grab input, clear the screen, or touch user data.

FBINK=${FBINK:-fbink}
LOG=${1:-}

if ! command -v "$FBINK" >/dev/null 2>&1; then
    echo "fbink not found. Put a Kindle-compatible fbink binary in PATH or set FBINK=/path/to/fbink." >&2
    exit 1
fi

if [ -z "$LOG" ]; then
    if [ -d /mnt/us ] && [ -w /mnt/us ]; then
        LOG=/mnt/us/kindle-chess-fbink-smoke.txt
    else
        LOG=/tmp/kindle-chess-fbink-smoke.txt
    fi
fi

umask 077
: >"$LOG" || exit 1

{
    echo "===== fbink smoke test ====="
    printf 'utc_time='; date -u '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null || date
    printf 'fbink_path=%s\n' "$(command -v "$FBINK")"
    echo
    echo "--- build/target ---"
    "$FBINK" --help 2>&1 | sed -n '1,8p'
    echo
    echo "--- verbose draw ---"
} >>"$LOG"

cat <<'NOTICE'
This test will draw ONE small message near the middle of the current Kindle screen.
It does NOT stop the stock UI or change framebuffer rotation/mode.
The stock UI may overwrite the message immediately; that is useful evidence.

After the command returns, use the normal Kindle UI (Home/back/page change or
lock/unlock if needed) to make the stock UI redraw. Do not run historical
"stop framework" commands for this checkpoint.
NOTICE

# Overlay avoids painting a background rectangle and touches fewer pixels.
# Use FBInk defaults for waveform/dithering: Task 00 is identification/bring-up,
# not refresh-policy tuning.
"$FBINK" -v -o -m -M "Kindle Chess FBInk probe" >>"$LOG" 2>&1
status=$?

{
    echo
    printf 'draw_exit_status=%s\n' "$status"
    echo "stock_ui_restore=perform via normal Kindle UI; record observation in docs/device/ks1-barolo.md"
} >>"$LOG"

if [ "$status" -eq 0 ]; then
    echo "FBInk draw returned success. Log: $LOG"
    echo "Now trigger a normal stock-UI redraw and record whether/when the probe text disappears."
else
    echo "FBInk draw failed with status $status. See: $LOG" >&2
fi

exit "$status"
