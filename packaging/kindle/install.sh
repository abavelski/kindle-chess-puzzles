#!/bin/sh
# Runtime-only install. No writes to puzzles, state, logs, or system services.
set -eu
root=${1:-/mnt/us/kindle-chess}
if [ "$root" = upgrade ]; then root=/mnt/us/kindle-chess; fi
documents=$(dirname "$root")/documents
package=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
mkdir -p "$root"
if ! mkdir "$root/.install-lock" 2>/dev/null; then
    echo 'install: another installation is active; inspect .install-lock' >&2
    exit 1
fi
cleanup() {
    code=$?
    trap - EXIT INT TERM HUP
    if [ -d "$root/runtime.old" ] && [ ! -d "$root/runtime" ]; then
        mv "$root/runtime.old" "$root/runtime" || true
    fi
    rm -rf "$root/runtime.new"
    rmdir "$root/.install-lock"
    exit "$code"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
if [ -d /tmp/kindle-chess.lock ] || { command -v pidof >/dev/null 2>&1 && pidof kindle-chess >/dev/null 2>&1; }; then
    echo 'install: exit Kindle Chess before updating' >&2
    exit 1
fi
if [ -e "$documents/kindle-chess.sh" ] && ! cmp -s "$documents/kindle-chess.sh" "$package/kindle-chess.sh"; then
    echo 'install: documents/kindle-chess.sh belongs to another file; move it first' >&2
    exit 1
fi
# Verify before touching the old runtime.
(cd "$package" && sha256sum -c SHA256SUMS) >/dev/null
# An interrupted update may leave the backup; never discard the only runtime.
if [ -d "$root/runtime.old" ] && [ ! -d "$root/runtime" ]; then
    mv "$root/runtime.old" "$root/runtime"
fi
rm -rf "$root/runtime.new"
mkdir "$root/runtime.new"
cp -R "$package/runtime/." "$root/runtime.new/"
chmod 755 "$root/runtime.new/kindle-chess" "$root/runtime.new/launch.sh" "$root/runtime.new/kindle_launch.sh"
cp "$package/kindle-chess.sh" "$root/runtime.new/scriptlet.sh"
cp "$package/uninstall.sh" "$root/runtime.new/uninstall.sh"
rm -rf "$root/runtime.old"
if [ -d "$root/runtime" ]; then mv "$root/runtime" "$root/runtime.old"; fi
mv "$root/runtime.new" "$root/runtime"
rm -rf "$root/runtime.old"
mkdir -p "$documents"
cp "$package/kindle-chess.sh" "$documents/kindle-chess.sh"
echo "Installed runtime at $root/runtime; user data preserved"
