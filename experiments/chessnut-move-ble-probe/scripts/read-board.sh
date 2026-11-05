#!/bin/sh
# Operator-started, single connection/read attempt; no automatic retry or LEDs.
set -eu
umask 077
root=/mnt/us/chessnut-ble-probe
runtime=$root/runtime
[ ! -L "$root" ] && [ ! -L "$runtime" ] &&
[ "$(cat "$root/.probe-owner")" = chessnut-ble-probe-v1 ] || exit 1
case "$#:${1:-}" in 0:) ;; 1:--check) ;; *) echo 'Usage: sh read-board.sh [--check]' >&2; exit 2;; esac
[ -f "$root/board-address" ] && [ ! -L "$root/board-address" ] || {
 echo "Missing private board address: $root/board-address" >&2; exit 2;
}
address=
IFS= read -r address < "$root/board-address" || [ -n "$address" ]
printf '%s\n' "$address" | grep -Eq '^([[:xdigit:]]{2}:){5}[[:xdigit:]]{2}$' || {
 echo 'Invalid private board address; no connection attempted' >&2; exit 2;
}
[ -f "$runtime/launch.sh" ] && [ -f "$runtime/build.json" ] || exit 2
sanitize() {
 awk '{
  gsub(/[[:xdigit:]][[:xdigit:]]:[[:xdigit:]][[:xdigit:]]:[[:xdigit:]][[:xdigit:]]:[[:xdigit:]][[:xdigit:]]:[[:xdigit:]][[:xdigit:]]:[[:xdigit:]][[:xdigit:]]/,"[REDACTED]");
  gsub(/gatt_cache_[[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]][[:xdigit:]]/,"gatt_cache_[REDACTED]");
  print; fflush();
 }'
}
[ "$(printf '12:34:56:78:90:AB\n' | sanitize)" = '[REDACTED]' ] || {
 echo 'Output redaction check failed; no connection attempted' >&2; exit 2;
}
if [ "${1:-}" = --check ]; then
 echo 'setup=PASS session=NOT_STARTED private_address=present output_redaction=PASS'
 exit 0
fi
[ ! -L "$root/logs" ] || exit 2
mkdir -p "$root/logs"
stamp=$(date +%Y%m%d-%H%M%S)
since=$(date +%y%m%d:%H%M%S)
report=$root/logs/read-board-$stamp-$$.log
work=$(mktemp -d "$root/logs/.read-board-XXXXXX")
child= logger= opened=false finished=false
cleanup() {
 result=$?
 trap - EXIT HUP INT TERM
 if [ -n "$child" ]; then
  kill -TERM "$child" 2>/dev/null || :
  wait "$child" 2>/dev/null || :
 fi
 if [ "$opened" = true ]; then
  if [ "$finished" = false ]; then
   printf '\nREAD_BOARD_RESULT=FAIL exit_status=%s interrupted_or_script_error=true\n' "$result" >&4
  fi
  exec 4>&-
 fi
 if [ -n "$logger" ]; then wait "$logger" 2>/dev/null || :; fi
 rm -rf -- "$work"
 exit "$result"
}
trap cleanup EXIT
trap 'exit 129' HUP
trap 'exit 130' INT
trap 'exit 143' TERM
mkfifo "$work/output"
sanitize < "$work/output" | tee "$report" &
logger=$!
exec 4> "$work/output"
opened=true
printf '=== CHESSNUT KINDLE READ TEST ===\n' >&4
printf 'started=%s\naddress=[REDACTED] attempts=1 LEDs=DISABLED motors=DISABLED\n' "$(date '+%Y-%m-%d %H:%M:%S %Z')" >&4
printf 'report=%s\n' "$report" >&4
grep 'source_sha256' "$runtime/build.json" >&4 || :
printf 'When connected, move one piece by hand; observation lasts 20 seconds.\n' >&4
# read-once never consumes stdin; preserve the SSH terminal for the operator.
sh "$runtime/launch.sh" read-once --address "$address" >&4 2>&1 4>&- < /dev/null &
child=$!
set +e
wait "$child"
result=$?
set -e
child=
printf '\n=== KINDLE BLUETOOTH LOG EXCERPT (since %s) ===\n' "$since" >&4
# Read existing logs only; include two archives in case rotation occurred.
{
 for archive in $(ls -t /var/local/log/messages_*.gz 2>/dev/null | head -n 2); do
  gzip -dc "$archive" 2>/dev/null || :
 done
 if [ -r /var/log/messages ]; then cat /var/log/messages; fi
} | awk -v since="$since" '$1 >= since && /btmanagerd|kindlebt-worker/ {print}' | tail -n 120 >&4
if [ -d "$root/.operation-lock" ]; then
 printf 'operation_lock=PRESENT (inspect before rerunning)\n' >&4
else
 printf 'operation_lock=cleared\n' >&4
fi
if [ "$result" = 0 ]; then outcome=PASS; else outcome=FAIL; fi
printf '\nREAD_BOARD_RESULT=%s exit_status=%s finished=%s\n' "$outcome" "$result" "$(date '+%Y-%m-%d %H:%M:%S %Z')" >&4
printf '=== END REPORT; paste this complete report ===\n' >&4
finished=true
exit "$result"
