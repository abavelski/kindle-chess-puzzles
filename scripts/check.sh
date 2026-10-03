#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 scripts/generate_sashite.py --check
python3 -m unittest tests/test_decode_evdev.py
python3 -m py_compile tools/decode_evdev.py
sh -n scripts/kindle_device_probe.sh
sh -n scripts/kindle_fbink_smoke.sh
sh -n scripts/kindle_capture_input.sh

python3 -m unittest tests/test_lifecycle.py tests/test_fbink_bridge.py tests/test_input_bridge.py
sh -n scripts/kindle_launch.sh

python3 -m unittest tests/test_packaging.py
for script in scripts/*-kindle.sh packaging/kindle/*.sh; do
    sh -n "$script"
done
