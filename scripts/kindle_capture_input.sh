#!/bin/sh
# Capture raw Linux input_event records for a short Task 00 trace.
# Read-only with respect to the input device: no EVIOCGRAB and no ioctls.

usage() {
    cat <<'USAGE'
Usage: kindle_capture_input.sh DEVICE LABEL [SECONDS] [OUTPUT_DIR]

Example:
  kindle_capture_input.sh /dev/input/event3 finger 20 /mnt/us/kindle-chess-input

During a finger capture, tap in this order with a short pause between taps:
  top-left, top-right, center, bottom-left, bottom-right

For a stylus/tablet candidate, repeat with LABEL=stylus using pen taps at the
same locations. Do not press side buttons unless you intentionally want them
in the trace.

The script NEVER grabs the input device. The stock Kindle UI will continue to
receive the same events.
USAGE
}

if [ "$#" -lt 2 ] || [ "$#" -gt 4 ]; then
    usage >&2
    exit 2
fi

DEVICE=$1
LABEL=$2
SECONDS=${3:-20}
OUTDIR=${4:-/mnt/us/kindle-chess-input}

case "$LABEL" in
    ''|*[!A-Za-z0-9._-]*)
        echo "LABEL may contain only letters, digits, dot, underscore, and hyphen" >&2
        exit 2
        ;;
esac

case "$DEVICE" in
    /dev/input/event*) ;;
    *)
        echo "DEVICE must look like /dev/input/eventN" >&2
        exit 2
        ;;
esac
case "$SECONDS" in
    ''|*[!0-9]*)
        echo "SECONDS must be a positive integer" >&2
        exit 2
        ;;
esac
[ "$SECONDS" -gt 0 ] || { echo "SECONDS must be > 0" >&2; exit 2; }

if [ ! -r "$DEVICE" ]; then
    echo "Cannot read $DEVICE (run through the jailbreak shell/root context used for homebrew)." >&2
    exit 1
fi

mkdir -p "$OUTDIR" || exit 1
umask 077
base="$OUTDIR/${LABEL}-$(basename "$DEVICE")"
bin="$base.bin"
meta="$base.meta.txt"

sys="/sys/class/input/$(basename "$DEVICE")/device"
{
    echo "label=$LABEL"
    echo "device=$DEVICE"
    printf 'utc_time='; date -u '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null || date
    if [ -r "$sys/name" ]; then printf 'name='; cat "$sys/name"; fi
    if [ -r "$sys/phys" ]; then printf 'phys='; cat "$sys/phys"; fi
    for attr in bustype vendor product version; do
        if [ -r "$sys/id/$attr" ]; then printf 'id_%s=' "$attr"; cat "$sys/id/$attr"; fi
    done
    for cap in ev key abs rel sw; do
        if [ -r "$sys/capabilities/$cap" ]; then printf 'cap_%s=' "$cap"; cat "$sys/capabilities/$cap"; fi
    done
    echo "seconds=$SECONDS"
    echo "grabbed=no"
} >"$meta"

: >"$bin" || exit 1

cat <<EOF2
Capturing $DEVICE for $SECONDS seconds -> $bin
Sequence for '$LABEL': top-left, top-right, center, bottom-left, bottom-right.
The device is NOT grabbed, so Kindle will also react to these touches.
EOF2

# Avoid relying on a particular BusyBox timeout implementation.
cat "$DEVICE" >"$bin" &
pid=$!
cleanup() {
    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
}
trap cleanup EXIT HUP INT TERM
sleep "$SECONDS"
cleanup
trap - EXIT HUP INT TERM

bytes=$(wc -c <"$bin" 2>/dev/null | tr -d ' ')
printf 'bytes=%s\n' "${bytes:-unknown}" >>"$meta"

echo "Capture complete: $bin"
echo "Metadata:         $meta"
echo "Copy both to your computer, then run:"
echo "  python3 tools/decode_evdev.py '$bin'"
