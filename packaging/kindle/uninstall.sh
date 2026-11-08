#!/bin/sh
set -eu
expected_app=puzzles
root=${1:-/mnt/us/kindle-chess}
# KPM removes its own package files on upgrade; retain the working install.
if [ "$root" = upgrade ]; then exit 0; fi
documents=$(dirname "$root")/documents
mkdir -p "$root"
if ! mkdir "$root/.install-lock" 2>/dev/null; then
    echo 'uninstall: installation active' >&2
    exit 1
fi
trap 'rmdir "$root/.install-lock"' EXIT
if [ -L "$root/.kindle-app-id" ] || { [ -e "$root/.kindle-app-id" ] && [ "$(cat "$root/.kindle-app-id")" != "$expected_app" ]; }; then
    echo 'uninstall: root belongs to another application' >&2
    exit 1
fi
for candidate in "$root/runtime" "$root/runtime.old" "$root/runtime.new"; do
    [ ! -e "$candidate" ] && [ ! -L "$candidate" ] && continue
    [ -d "$candidate" ] && [ ! -L "$candidate" ] || exit 1
    if [ -f "$candidate/app-id" ] && [ ! -L "$candidate/app-id" ]; then
        [ "$(cat "$candidate/app-id")" = "$expected_app" ] || exit 1
    else
        [ "$expected_app" = puzzles ] && [ -f "$candidate/kindle-chess" ] || exit 1
    fi
done
if [ -d /tmp/kindle-chess.lock ] || { command -v pidof >/dev/null 2>&1 && pidof kindle-chess >/dev/null 2>&1; }; then
    echo 'uninstall: exit this application first' >&2
    exit 1
fi
if [ -f "$documents/kindle-chess.sh" ] && cmp -s "$documents/kindle-chess.sh" "$root/runtime/scriptlet.sh"; then
    rm "$documents/kindle-chess.sh"
fi
rm -rf "$root/runtime" "$root/runtime.old" "$root/runtime.new"
echo 'Runtime removed; all user data and root identity preserved. Matching library scriptlet removed.'
