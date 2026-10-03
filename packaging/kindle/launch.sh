#!/bin/sh
set -eu
runtime=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(dirname "$runtime")
mkdir -p "$root/logs"
exec 2>>"$root/logs/launch.log"
if [ "$(uname -m)" != armv7l ]; then
    echo 'launch: requires armv7l Scribe; refusing incompatible architecture' >&2
    exit 1
fi
if [ ! -e /lib/ld-linux-armhf.so.3 ]; then
    echo 'launch: ARM hard-float loader missing' >&2
    exit 1
fi
export KINDLE_CHESS_LOG="$root/logs/lifecycle.log"
export KINDLE_CHESS_PUZZLE_DIR=${KINDLE_CHESS_PUZZLE_DIR:-$root/puzzles}
export KINDLE_CHESS_PROGRESS_FILE=${KINDLE_CHESS_PROGRESS_FILE:-$root/state/progress.json}
exec sh "$runtime/kindle_launch.sh" "$runtime/kindle-chess" "$@"
