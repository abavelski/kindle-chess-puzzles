#!/bin/sh
set -eu
root=/mnt/us/chessnut-ble-probe
[ ! -L "$root" ] && [ -d "$root" ] || exit 1
[ ! -L "$root/.probe-owner" ] && [ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ] || exit 1
mkdir "$root/.operation-lock" || { echo 'Experiment busy; exit preflight before uninstalling' >&2; exit 1; }
# Fixed, ownership-checked experiment root. rm removes internal symlinks, never follows them.
trap 'rmdir "$root/.operation-lock" 2>/dev/null || :' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
rm -rf "$root"
echo 'Removed /mnt/us/chessnut-ble-probe only.'
