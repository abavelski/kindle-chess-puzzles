#!/bin/sh
# Runtime-only transaction. Libraries, state, logs and private config stay intact.
set -eu
expected_app=puzzles
root=${1:-/mnt/us/kindle-chess}
if [ "$root" = upgrade ]; then root=/mnt/us/kindle-chess; fi
documents=$(dirname "$root")/documents
package=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
mkdir -p "$root"
if ! mkdir "$root/.install-lock" 2>/dev/null; then
    echo 'install: another installation is active; inspect .install-lock' >&2
    exit 1
fi
installing=0
committed=0
transaction_allowed=0
had_runtime=0
temporary_scriptlet=
owned_runtime() {
    [ ! -e "$1" ] && [ ! -L "$1" ] && return 0
    [ -d "$1" ] && [ ! -L "$1" ] || return 1
    if [ -f "$1/app-id" ] && [ ! -L "$1/app-id" ]; then
        [ "$(cat "$1/app-id")" = "$expected_app" ]
    else
        # Permit the owner's existing pre-split Puzzles runtime to upgrade.
        [ "$expected_app" = puzzles ] && [ -f "$1/kindle-chess" ]
    fi
}
cleanup() {
    code=$?
    trap - EXIT INT TERM HUP
    if [ "$transaction_allowed" -eq 1 ]; then
        if [ "$installing" -eq 1 ] && [ "$committed" -eq 0 ]; then
            if [ -d "$root/runtime.old" ]; then
                rm -rf "$root/runtime"
                mv "$root/runtime.old" "$root/runtime" || true
            elif [ "$had_runtime" -eq 0 ]; then
                rm -rf "$root/runtime"
            fi
        elif [ -d "$root/runtime.old" ] && [ ! -d "$root/runtime" ]; then
            mv "$root/runtime.old" "$root/runtime" || true
        fi
        if owned_runtime "$root/runtime.new"; then rm -rf "$root/runtime.new"; fi
        if [ "$committed" -eq 1 ]; then rm -rf "$root/runtime.old"; fi
    fi
    if [ -n "$temporary_scriptlet" ]; then rm -f "$temporary_scriptlet"; fi
    rmdir "$root/.install-lock"
    exit "$code"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP
if [ -L "$root/.kindle-app-id" ] || { [ -e "$root/.kindle-app-id" ] && [ "$(cat "$root/.kindle-app-id")" != "$expected_app" ]; }; then
    echo 'install: root belongs to another application' >&2
    exit 1
fi
for candidate in "$root/runtime" "$root/runtime.old" "$root/runtime.new"; do
    owned_runtime "$candidate" || { echo 'install: unrecognized runtime preserved; inspect it first' >&2; exit 1; }
done
if [ -d /tmp/kindle-chess.lock ] || { command -v pidof >/dev/null 2>&1 && pidof kindle-chess >/dev/null 2>&1; }; then
    echo 'install: exit this application before updating' >&2
    exit 1
fi
if [ -e "$documents/kindle-chess.sh" ] && ! cmp -s "$documents/kindle-chess.sh" "$package/kindle-chess.sh" && ! cmp -s "$documents/kindle-chess.sh" "$root/runtime/scriptlet.sh"; then
    echo 'install: library document belongs to another file; move it first' >&2
    exit 1
fi
# Verify payload and identity before replacing any installed runtime.
(cd "$package" && sha256sum -c SHA256SUMS) >/dev/null
[ "$(cat "$package/runtime/app-id")" = "$expected_app" ] || exit 1
transaction_allowed=1
if [ ! -e "$root/.kindle-app-id" ]; then
    (set -C; printf '%s\n' "$expected_app" > "$root/.kindle-app-id") || exit 1
fi
if [ -d "$root/runtime.old" ] && [ ! -d "$root/runtime" ]; then mv "$root/runtime.old" "$root/runtime"; fi
rm -rf "$root/runtime.new"
mkdir "$root/runtime.new"
printf '%s\n' "$expected_app" > "$root/runtime.new/app-id"
cp -R "$package/runtime/." "$root/runtime.new/"
chmod 755 "$root/runtime.new/kindle-chess" "$root/runtime.new/launch.sh" "$root/runtime.new/kindle_launch.sh"
cp "$package/kindle-chess.sh" "$root/runtime.new/scriptlet.sh"
cp "$package/uninstall.sh" "$root/runtime.new/uninstall.sh"
mkdir -p "$documents"
scriptlet_candidate="$documents/.kindle-chess.sh.new.$$"
(set -C; : > "$scriptlet_candidate") || exit 1
temporary_scriptlet=$scriptlet_candidate
cp "$package/kindle-chess.sh" "$temporary_scriptlet"
rm -rf "$root/runtime.old"
if [ -d "$root/runtime" ]; then had_runtime=1; fi
installing=1
if [ -d "$root/runtime" ]; then mv "$root/runtime" "$root/runtime.old"; fi
mv "$root/runtime.new" "$root/runtime"
mv "$temporary_scriptlet" "$documents/kindle-chess.sh"
temporary_scriptlet=
committed=1
echo "Installed $expected_app runtime at $root/runtime; user data preserved"
