#!/bin/sh
set -eu
root=/mnt/us/chessnut-ble-probe
[ ! -L "$root" ] && [ -d "$root" ] || exit 1
[ ! -L "$root/.probe-owner" ] && [ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ] || exit 1
incoming=$(pwd -P)
[ "$(dirname "$incoming")" = "$root" ]
case "$(basename "$incoming")" in incoming-*) ;; *) exit 1;; esac
[ ! -L "$root/runtime" ] && [ ! -L "$root/runtime.old" ] && [ ! -L "$root/runtime.new" ] || exit 1
[ ! -e "$root/runtime.old" ] && [ ! -e "$root/runtime.new" ] || {
  echo 'Incomplete previous install: inspect runtime.old/runtime.new and recover before updating' >&2; exit 1;
}
mkdir "$root/.operation-lock" || { echo 'Experiment busy; do not update during preflight' >&2; exit 1; }
success=0
cleanup() {
  result=$?
  trap - EXIT HUP INT TERM
  if [ "$success" = 0 ]; then
    rm -rf "$root/runtime.new"
    if [ -d "$root/runtime.old" ]; then
      rm -rf "$root/runtime"
      mv "$root/runtime.old" "$root/runtime" || echo 'RECOVERY: rename runtime.old to runtime manually' >&2
    fi
  fi
  rmdir "$root/.operation-lock"
  exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
verify() {
  (
    cd "$1"
    [ -f SHA256SUMS ] && [ ! -L SHA256SUMS ]
    [ -z "$(find . -type l -print)" ]
    while IFS= read -r line; do
      printf '%s\n' "$line" | grep -Eq '^[0-9a-f]{64}  [A-Za-z0-9_./+-]+$' || exit 1
      name=${line#*  }
      case "$name" in /*|*..*) exit 1;; esac
    done < SHA256SUMS
    [ "$(find . -type f ! -name SHA256SUMS | wc -l | tr -d ' ')" = "$(wc -l < SHA256SUMS | tr -d ' ')" ]
    sha256sum -c SHA256SUMS
  )
}
verify "$incoming"
verify "$incoming/runtime"
cp -R "$incoming/runtime" "$root/runtime.new"
verify "$root/runtime.new"
if [ -d "$root/runtime" ]; then
  mv "$root/runtime" "$root/runtime.old"
fi
mv "$root/runtime.new" "$root/runtime"
success=1
rm -rf "$root/runtime.old"
echo 'Installed Chessnut probe only. No command was launched. Run H1 preflight manually.'
