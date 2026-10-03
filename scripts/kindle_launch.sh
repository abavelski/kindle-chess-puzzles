#!/bin/sh
# Foreground overlay supervisor. Exclusive finger input and native exit repaint
# are verified on Scribe 5.19.6; natural suspend/resume remains unverified.
set -u
umask 077
binary=${1:-/mnt/us/kindle-chess/kindle-chess}
if [ "$#" -gt 0 ]; then shift; fi
lock=${KINDLE_CHESS_LOCK:-/tmp/kindle-chess.lock}
log=${KINDLE_CHESS_LOG:-/mnt/us/kindle-chess/lifecycle.log}
child=
owned=0
child_started=0
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
"$binary" "$@" &
child=$!
child_started=1
printf '%s\n' "$child" > "$lock/child.pid"
wait "$child"
code=$?
# Reaped child no longer belongs to us. Never signal a reused PID.
child=
exit "$code"
