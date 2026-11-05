#!/bin/sh
set -eu
[ "$#" = 3 ] || { echo 'Usage: check-native-adapter.sh <KindleBT include> <ACE SDK include> <log.c src>' >&2; exit 2; }
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
mkdir -p "$root/target"
cc -std=c11 -Wall -Wextra -Werror -pthread -I "$1" -I "$2" -I "$3" \
 -I "$root/native" "$root/tests/native_adapter_test.c" -ldl -o "$root/target/native-adapter-test"
"$root/target/native-adapter-test"
