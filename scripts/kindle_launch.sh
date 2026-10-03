#!/bin/sh
# Foreground overlay supervisor. Exclusive finger input and native exit repaint
# are verified on Scribe 5.19.6; natural suspend/resume remains unverified.
# Pause verified awesome/Xorg processes to prevent native launch-time repaint.
set -u
umask 077
binary=${1:-/mnt/us/kindle-chess/kindle-chess}
if [ "$#" -gt 0 ]; then shift; fi
lock=${KINDLE_CHESS_LOCK:-/tmp/kindle-chess.lock}
log=${KINDLE_CHESS_LOG:-/mnt/us/kindle-chess/lifecycle.log}
child=
owned=0
child_started=0
wm_pid=
wm_start=
wm_owned=0
x_pid=
x_start=
x_owned=0
proc_root=${KINDLE_CHESS_PROC_ROOT:-/proc}
display_signal=${KINDLE_CHESS_SIGNAL:-/bin/kill}

display_identity() {
    [ "$(cat "$proc_root/$1/comm" 2>/dev/null)" = "$2" ] || return 1
    current_start=$(awk '{print $22}' "$proc_root/$1/stat" 2>/dev/null) || return 1
    [ -n "$3" ] && [ "$current_start" = "$3" ]
}

verify_display_process() {
    case "$1" in ''|*[!0-9]*) echo "lifecycle: expected one $2 PID" >&2; return 1;; esac
    verified_start=$(awk '{print $22}' "$proc_root/$1/stat") || return 1
    display_identity "$1" "$2" "$verified_start" || return 1
    display_state=$(awk '/^State:/ {print $2}' "$proc_root/$1/status") || return 1
    case "$display_state" in
        R|S|D|I) ;;
        *) echo "lifecycle: $2 is not running (state=$display_state); refusing handoff" >&2; return 1;;
    esac
}

resume_display_process() {
    [ "$4" -eq 1 ] || return 0
    if display_identity "$1" "$2" "$3"; then
        echo "lifecycle: resuming $2 pid=$1" >&2
        "$display_signal" -CONT "$1" || echo "lifecycle: $2 resume failed; use manual recovery" >&2
    else
        echo "lifecycle: $2 identity changed; refusing to signal reused PID" >&2
    fi
}
# Native repaint verified on Scribe 5.19.6. An explicit empty override disables it.
repaint=${KINDLE_CHESS_XREFRESH-xrefresh}
repaint_timeout=${KINDLE_CHESS_TIMEOUT:-/usr/bin/timeout}

cleanup() {
    code=$?
    trap - EXIT INT TERM HUP
    if [ -n "$child" ] && kill -0 "$child" 2>/dev/null; then
        echo "lifecycle: forwarding TERM to child $child" >&2
        kill -TERM "$child" 2>/dev/null || true
        attempts=0
        while kill -0 "$child" 2>/dev/null && [ "$attempts" -lt 5 ]; do
            sleep 1
            attempts=$((attempts + 1))
        done
        if kill -0 "$child" 2>/dev/null; then
            echo "lifecycle: child timeout; forwarding KILL" >&2
            kill -KILL "$child" 2>/dev/null || true
        fi
        wait "$child" 2>/dev/null || true
    fi
    # Resume the X server before its window manager, then repaint native UI.
    resume_display_process "$x_pid" Xorg "$x_start" "$x_owned"
    resume_display_process "$wm_pid" awesome "$wm_start" "$wm_owned"
    if [ "$child_started" -eq 1 ] && [ -n "$repaint" ]; then
        if command -v "$repaint" >/dev/null 2>&1 && command -v "$repaint_timeout" >/dev/null 2>&1; then
            echo "lifecycle: requesting native repaint on :0.0" >&2
            if ! "$repaint_timeout" -k 1 5 "$repaint" -display :0.0; then
                echo "lifecycle: native repaint failed/timed out; use Home/back or lock/unlock" >&2
            fi
        else
            echo "lifecycle: native repaint tools unavailable; use Home/back or lock/unlock" >&2
        fi
    fi
    if [ "$owned" -eq 1 ]; then
        rm -f "$lock/child.pid" "$lock/launcher.pid"
        rmdir "$lock" || true
    fi
    echo "lifecycle: exit=$code; overlay released" >&2
    exit "$code"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

if ! mkdir "$lock" 2>/dev/null; then
    echo "lifecycle: lock exists: $lock; inspect recorded PIDs before recovery" >&2
    exit 1
fi
owned=1
printf '%s\n' "$$" > "$lock/launcher.pid"
if [ ! -x "$binary" ]; then
    echo "lifecycle: executable missing: $binary" >&2
    exit 1
fi
# Also refuse an app started outside this supervisor on the Kindle.
if command -v pidof >/dev/null 2>&1 && pidof kindle-chess >/dev/null 2>&1; then
    echo "lifecycle: an unsupervised kindle-chess instance is already running" >&2
    exit 1
fi
mkdir -p "$(dirname "$log")" || exit 1
exec 2>>"$log"
echo "lifecycle: launch $(date -u '+%Y-%m-%dT%H:%M:%SZ') pid=$$ binary=$binary" >&2
# Scribe 5.19.6: winmgr's Active App timeout and pending Xorg writes
# overpaint direct FBInk output. Validate both before taking ownership.
# Refuse pre-stopped processes so we cannot resume another app's handoff.
if [ "${KINDLE_CHESS_DISPLAY_HANDOFF:-1}" = 1 ]; then
    wm_pid=$(pidof awesome) || { echo "lifecycle: awesome unavailable" >&2; exit 1; }
    verify_display_process "$wm_pid" awesome || exit 1
    wm_start=$verified_start
    x_pid=$(pidof Xorg) || { echo "lifecycle: Xorg unavailable" >&2; exit 1; }
    verify_display_process "$x_pid" Xorg || exit 1
    x_start=$verified_start
    echo "lifecycle: pausing awesome pid=$wm_pid start=$wm_start" >&2
    # Own cleanup before STOP so a signal at the handoff cannot strand either.
    wm_owned=1
    "$display_signal" -STOP "$wm_pid" || exit 1
    echo "lifecycle: pausing Xorg pid=$x_pid start=$x_start" >&2
    x_owned=1
    "$display_signal" -STOP "$x_pid" || exit 1
fi
"$binary" "$@" &
child=$!
child_started=1
printf '%s\n' "$child" > "$lock/child.pid"
wait "$child"
code=$?
# Reaped child no longer belongs to us. Never signal a reused PID.
child=
exit "$code"
