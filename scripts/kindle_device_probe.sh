#!/bin/sh
# Read-mostly hardware/environment probe for Kindle Chess Puzzles Task 00.
# Intentionally avoids serial numbers, network/account data, raw framebuffer reads,
# service changes, input grabs, and writes outside an optional tiny write check.

PROBE_VERSION=1
WRITE_CHECK=0
OUT=""

usage() {
    cat <<'USAGE'
Usage: kindle_device_probe.sh [--write-check] [OUTPUT_FILE]

Collects privacy-conscious Kindle hardware/environment facts for Task 00.
By default it performs read-only checks and writes only the report itself.

--write-check  additionally create/remove a tiny file in /mnt/us and /tmp
               to confirm those locations are writable. No user file is replaced.

If OUTPUT_FILE is omitted, the report is written to:
  /mnt/us/kindle-chess-probe.txt  when /mnt/us exists and is writable, else
  /tmp/kindle-chess-probe.txt
USAGE
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --write-check)
            WRITE_CHECK=1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
        *)
            if [ -n "$OUT" ]; then
                echo "Only one output file may be specified." >&2
                exit 2
            fi
            OUT=$1
            ;;
    esac
    shift
done

if [ -z "$OUT" ]; then
    if [ -d /mnt/us ] && [ -w /mnt/us ]; then
        OUT=/mnt/us/kindle-chess-probe.txt
    else
        OUT=/tmp/kindle-chess-probe.txt
    fi
fi

umask 077

# Keep stdout useful when run interactively while putting the complete probe in one file.
exec 3>&1
if ! : >"$OUT" 2>/dev/null; then
    echo "Cannot write probe report to: $OUT" >&2
    exit 1
fi
exec >>"$OUT" 2>&1

section() {
    printf '\n===== %s =====\n' "$1"
}

show_cmd() {
    label=$1
    shift
    printf '\n--- %s ---\n' "$label"
    if "$@"; then
        :
    else
        status=$?
        printf '[command unavailable or exited %s]\n' "$status"
    fi
}

show_file() {
    path=$1
    max_lines=${2:-80}
    printf '\n--- %s ---\n' "$path"
    if [ -r "$path" ]; then
        # Exclude common unique-device identifiers before writing anything to disk.
        sed -e '/Serial/d' -e '/serial/d' -e '/MAC/d' -e '/mac/d' \
            -e '/UUID/d' -e '/uuid/d' -e '/^U: Uniq=/d' "$path" 2>/dev/null | sed -n "1,${max_lines}p"
    else
        printf '[not readable]\n'
    fi
}

show_value() {
    label=$1
    path=$2
    printf '%s=' "$label"
    if [ -r "$path" ]; then
        tr '\000' '\n' <"$path" 2>/dev/null | sed -n '1p'
    else
        printf '[not readable]\n'
    fi
}

exists_line() {
    path=$1
    if [ -e "$path" ]; then
        printf '%s: present\n' "$path"
    else
        printf '%s: absent\n' "$path"
    fi
}

command_line() {
    name=$1
    if command -v "$name" >/dev/null 2>&1; then
        printf '%s: %s\n' "$name" "$(command -v "$name")"
    else
        printf '%s: not found\n' "$name"
    fi
}

write_check() {
    dir=$1
    marker="$dir/.kindle-chess-probe-write-test.$$"
    printf '%s: ' "$dir"
    if [ ! -d "$dir" ]; then
        printf 'directory absent\n'
        return
    fi
    if (umask 077; printf 'probe\n' >"$marker") 2>/dev/null; then
        rm -f "$marker"
        printf 'create/remove OK\n'
    else
        rm -f "$marker" 2>/dev/null || true
        printf 'write failed\n'
    fi
}

section "probe"
printf 'probe_version=%s\n' "$PROBE_VERSION"
printf 'utc_time='; date -u '+%Y-%m-%dT%H:%M:%SZ' 2>/dev/null || date
printf 'output=%s\n' "$OUT"
printf 'write_check=%s\n' "$WRITE_CHECK"
printf 'privacy_note=serial/network/account data intentionally excluded\n'

section "kernel and architecture"
show_cmd "uname -a" uname -a
show_cmd "uname -m" uname -m
if command -v getconf >/dev/null 2>&1; then
    show_cmd "getconf LONG_BIT" getconf LONG_BIT
    show_cmd "getconf GNU_LIBC_VERSION" getconf GNU_LIBC_VERSION
else
    printf 'getconf: not found\n'
fi
if command -v file >/dev/null 2>&1; then
    show_cmd "file /bin/sh" file /bin/sh
fi
if command -v readelf >/dev/null 2>&1; then
    printf '\n--- readelf ABI attributes for /bin/sh ---\n'
    readelf -A /bin/sh 2>/dev/null | grep -E 'Tag_CPU|Tag_ABI_VFP_args|Tag_FP_arch|Tag_THUMB_ISA_use' || true
fi
printf '\n--- selected /proc/cpuinfo fields ---\n'
if [ -r /proc/cpuinfo ]; then
    grep -E '^(processor|model name|Processor|Features|CPU architecture|Hardware|Revision)[[:space:]]*:' /proc/cpuinfo 2>/dev/null | sed -n '1,100p'
fi

section "firmware and userland"
for p in /etc/prettyversion.txt /etc/version.txt /etc/version /etc/os-release /etc/issue; do
    [ -e "$p" ] && show_file "$p" 80
done
if command -v ldd >/dev/null 2>&1; then
    printf '\n--- ldd version ---\n'
    ldd --version 2>&1 | sed -n '1,3p'
fi

section "board identity without unique IDs"
for p in /proc/device-tree/model /sys/firmware/devicetree/base/model /sys/devices/soc0/machine /sys/devices/soc0/family; do
    [ -e "$p" ] && show_value "$p" "$p"
done

section "framebuffer sysfs"
found_fb=0
for fb in /sys/class/graphics/fb[0-9]*; do
    [ -e "$fb" ] || continue
    found_fb=1
    printf '\n--- %s ---\n' "$fb"
    for attr in name virtual_size bits_per_pixel stride rotate mode modes; do
        [ -e "$fb/$attr" ] && show_value "$attr" "$fb/$attr"
    done
done
[ "$found_fb" -eq 1 ] || printf '[no /sys/class/graphics/fbN entries found]\n'
if command -v fbset >/dev/null 2>&1; then
    show_cmd "fbset -i" fbset -i
else
    printf '\nfbset: not found\n'
fi

section "FBInk availability"
command_line fbink
if command -v fbink >/dev/null 2>&1; then
    printf '\n--- fbink build/target (help header only; does not draw) ---\n'
    fbink --help 2>&1 | sed -n '1,8p'
fi
command_line input_scan
if command -v input_scan >/dev/null 2>&1; then
    printf '\n--- input_scan build/target ---\n'
    input_scan --help 2>&1 | sed -n '1,8p'
    printf '\n--- input_scan likely touch/tablet matches ---\n'
    input_scan -v -m touchscreen,tablet,scaled_tablet,frame_tap 2>&1 || true
fi

section "input devices"
show_file /proc/bus/input/devices 300
printf '\n--- event sysfs summary ---\n'
for ev in /sys/class/input/event*; do
    [ -e "$ev" ] || continue
    event_name=$(basename "$ev")
    printf '\n[%s]\n' "$event_name"
    show_value name "$ev/device/name"
    [ -e "$ev/device/phys" ] && show_value phys "$ev/device/phys"
    for attr in bustype vendor product version; do
        [ -e "$ev/device/id/$attr" ] && show_value "id_$attr" "$ev/device/id/$attr"
    done
    for cap in ev key abs rel sw; do
        [ -e "$ev/device/capabilities/$cap" ] && show_value "cap_$cap" "$ev/device/capabilities/$cap"
    done
done

section "storage candidates"
for dir in /mnt/us /mnt/base-us /var/local /tmp; do
    if [ -e "$dir" ]; then
        printf '\n[%s]\n' "$dir"
        [ -d "$dir" ] && printf 'directory=yes\n' || printf 'directory=no\n'
        [ -r "$dir" ] && printf 'readable=yes\n' || printf 'readable=no\n'
        [ -w "$dir" ] && printf 'writable_by_current_user=yes\n' || printf 'writable_by_current_user=no\n'
        if command -v df >/dev/null 2>&1; then
            df -P "$dir" 2>/dev/null | sed -n '1,2p'
        fi
    else
        printf '\n[%s]\nabsent\n' "$dir"
    fi
done
if [ "$WRITE_CHECK" -eq 1 ]; then
    printf '\n--- optional create/remove write check ---\n'
    write_check /mnt/us
    write_check /tmp
fi

section "homebrew mechanism hints"
for cmd in kpm mrpi-helper dispatch-command fbink input_scan evtest ssh dropbear; do
    command_line "$cmd"
done
for path in /mnt/us/extensions /mnt/us/scriptlets /mnt/us/.addons /mnt/us/koreader; do
    exists_line "$path"
done
if command -v kpm >/dev/null 2>&1; then
    printf '\n--- kpm version/help header ---\n'
    kpm --version 2>&1 | sed -n '1,5p' || kpm --help 2>&1 | sed -n '1,8p' || true
fi

section "safe lifecycle observations"
printf 'stock_services_changed=no (probe never stops/disables services)\n'
printf 'input_grabbed=no (probe never EVIOCGRABs devices)\n'
printf 'framebuffer_written=no (probe itself never writes framebuffer)\n'
printf 'raw_framebuffer_read=no\n'
printf 'next_step=run scripts/kindle_fbink_smoke.sh separately, then capture candidate input devices\n'

section "commands useful for the human checkpoint"
printf '%s\n' '1. Run kindle_fbink_smoke.sh only after reviewing its screen-write behavior.'
printf '%s\n' '2. Use kindle_capture_input.sh /dev/input/eventN finger to record corner/center taps.'
printf '%s\n' '3. Repeat for the stylus/tablet candidate with label stylus.'
printf '%s\n' '4. Copy *.bin and *.meta.txt to the development machine and run tools/decode_evdev.py.'

printf '\nProbe complete.\n'
printf 'Report: %s\n' "$OUT"

printf 'Probe complete: %s\n' "$OUT" >&3
