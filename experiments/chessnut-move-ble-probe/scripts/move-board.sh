#!/bin/sh
# User-started guided M4; no automatic confirmation or repeated motion.
set -eu
umask 077
root=/mnt/us/chessnut-ble-probe
runtime=$root/runtime
[ ! -L "$root" ] && [ ! -L "$runtime" ] &&
[ ! -L "$root/.probe-owner" ] && [ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ] || exit 1
mode=move-once
case "$#:${1:-}" in
 0:) ;;
 1:--return) mode=return-once ;;
 1:--check) mode=check ;;
 *) echo 'Usage: sh move-board.sh [--return|--check]' >&2; exit 2;;
esac
[ -f "$root/board-address" ] && [ ! -L "$root/board-address" ] || exit 2
address=
IFS= read -r address < "$root/board-address" || [ -n "$address" ]
printf '%s\n' "$address" | grep -Eq '^([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}$' || exit 2
[ -f "$runtime/launch.sh" ] && [ ! -L "$runtime/launch.sh" ] || exit 2
if [ "$mode" = check ]; then
 echo 'guided_setup=PASS session=NOT_STARTED motor_writes=0'
 exit 0
fi
[ -t 0 ] && [ -t 1 ] || { echo 'Use an interactive Kindle SSH terminal; piped confirmations are refused.' >&2; exit 2; }
[ ! -L "$root/logs" ] || exit 2
mkdir -p "$root/logs"
stamp=$(date +%Y%m%d-%H%M%S)-$$
report=$root/logs/move-$stamp-session.log
native=$root/logs/move-$stamp-native.log
[ ! -e "$report" ] && [ ! -e "$native" ] || exit 2
printf '\nGuided move: one white pawn; white king e1, black king e8; all other squares empty.\n'
if [ "$mode" = return-once ]; then
 echo 'Expected pawn e7; proposed return e7 -> e2.'
else
 echo 'Expected pawn e2; proposed move e2 -> e7.'
fi
printf 'Clear travel area and verify piece registration/batteries.\nThe script connects, checks sensors, then waits for confirm or cancel.\nWhen the pawn finishes, type done to verify its final position.
During motion type stop to request STOP. Keep physical power-off accessible.\n'
printf 'Session log: %s\nNative diagnostics: %s\n\n' "$report" "$native"
export PROBE_SESSION_LOG=$report
# exec preserves terminal signals and stdin; launch.sh owns worker cleanup.
# Native raw telemetry is logged privately and never floods the terminal.
exec sh "$runtime/launch.sh" "$mode" --address "$address" 2>"$native"
