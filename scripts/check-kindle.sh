#!/bin/sh
set -eu

TARGET=armv7-unknown-linux-gnueabihf
LINKER=${KINDLE_LINKER:-arm-linux-gnueabihf-gcc}

if ! command -v "$LINKER" >/dev/null 2>&1; then
    echo "Missing cross linker: $LINKER" >&2
    echo "Set KINDLE_LINKER to a compatible ARMv7 hard-float GCC linker." >&2
    exit 1
fi

CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="$LINKER" \
    cargo build -p kindle-chess --target "$TARGET"
