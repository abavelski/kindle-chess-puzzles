#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 scripts/generate_sashite.py --check
python3 -m unittest tests/test_decode_evdev.py
python3 -m unittest tests/test_rich_analysis_contract.py
python3 -m unittest tests/test_game_review_contract.py
python3 -m unittest tests/test_pgn_converter.py
python3 -m unittest tests/test_collection_update.py
python3 -m unittest tests/test_clean_book_descriptions.py
python3 -m unittest tests/test_clean_book_analysis.py
python3 -m py_compile tools/decode_evdev.py tools/pgn_converter.py tools/collection_update.py tools/clean_book_descriptions.py tools/clean_book_analysis.py
sh -n scripts/kindle_device_probe.sh
sh -n scripts/kindle_fbink_smoke.sh
sh -n scripts/kindle_capture_input.sh

python3 -m unittest tests/test_lifecycle.py tests/test_fbink_bridge.py tests/test_input_bridge.py
sh -n scripts/kindle_launch.sh

python3 -m unittest tests/test_packaging.py
for script in scripts/*-kindle.sh packaging/kindle/*.sh; do
    sh -n "$script"
done
