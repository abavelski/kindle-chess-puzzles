#!/bin/sh
set -eu

TARGET=armv7-unknown-linux-gnueabihf
LINKER=${KINDLE_LINKER:-arm-linux-gnueabihf-gcc}
AR=${KINDLE_AR:-arm-linux-gnueabihf-ar}
RANLIB=${KINDLE_RANLIB:-arm-linux-gnueabihf-ranlib}

for tool in "$LINKER" "$AR" "$RANLIB" make; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Missing cross-build tool: $tool" >&2
        exit 1
    fi
done

if [ ! -f vendor/FBInk/fbink.h ]; then
    echo "Pinned FBInk submodule is missing. Run: git submodule update --init --recursive" >&2
    exit 1
fi

KINDLE_CC="$LINKER" KINDLE_AR="$AR" KINDLE_RANLIB="$RANLIB" CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="$LINKER"     cargo build -p kindle-chess --target "$TARGET"
