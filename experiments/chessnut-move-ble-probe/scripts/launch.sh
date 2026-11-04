#!/bin/sh
set -eu
root=/mnt/us/chessnut-ble-probe
runtime=$root/runtime
[ ! -L "$root" ] && [ ! -L "$runtime" ] || exit 1
[ ! -L "$root/.probe-owner" ] && [ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ] || exit 1
[ "$#" = 0 ] && set -- preflight
case "$#:$1" in
  1:preflight) ;;
  3:session) [ "$2" = --address ] || exit 1 ;;
  *) echo 'UNSUPPORTED: use preflight or session --address <private BLE address>' >&2; exit 1 ;;
esac
mkdir "$root/.operation-lock" || { echo 'Experiment busy' >&2; exit 1; }
child=
cleanup() {
  result=$?
  trap - EXIT HUP INT TERM
  if [ -n "$child" ]; then
    kill -TERM "$child" 2>/dev/null || :
    wait "$child" 2>/dev/null || :
  fi
  rmdir "$root/.operation-lock"
  exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
cd "$runtime"
sha256sum -c SHA256SUMS
[ "$(uname -m)" = armv7l ] && [ -x /lib/ld-linux-armhf.so.3 ] || { echo 'Unsupported architecture/loader' >&2; exit 1; }
LD_LIBRARY_PATH="$runtime" "$runtime/chessnut-ble-probe" "$@" <&0 &
child=$!
set +e
wait "$child"
result=$?
set -e
child=
exit "$result"
