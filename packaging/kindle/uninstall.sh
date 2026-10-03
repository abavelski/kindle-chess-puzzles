#!/bin/sh
set -eu
root=${1:-/mnt/us/kindle-chess}
# KPM removes its own package files on upgrade; preserve the working install.
if [ "$root" = upgrade ]; then exit 0; fi
documents=$(dirname "$root")/documents
mkdir -p "$root"
if ! mkdir "$root/.install-lock" 2>/dev/null; then
    echo 'uninstall: installation active' >&2
    exit 1
fi
trap 'rmdir "$root/.install-lock"' EXIT
if [ -d /tmp/kindle-chess.lock ] || { command -v pidof >/dev/null 2>&1 && pidof kindle-chess >/dev/null 2>&1; }; then
    echo 'uninstall: exit Kindle Chess first' >&2
    exit 1
fi
if [ -f "$documents/kindle-chess.sh" ] && cmp -s "$documents/kindle-chess.sh" "$root/runtime/scriptlet.sh"; then
    rm "$documents/kindle-chess.sh"
fi
rm -rf "$root/runtime" "$root/runtime.old" "$root/runtime.new"
echo 'Runtime removed; puzzles, progress and logs preserved. Matching library scriptlet removed.'
