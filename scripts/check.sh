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
python3 -m unittest tests/test_pgn_review_converter.py
python3 -m unittest tests/test_collection_update.py
python3 -m unittest tests/test_migrate_app_data.py
python3 -m unittest tests/test_clean_book_descriptions.py
python3 -m unittest tests/test_clean_book_analysis.py
python3 -m unittest tests/test_clean_pgn.py
python3 -m py_compile tools/migrate_app_data.py tools/decode_evdev.py tools/pgn_converter.py tools/pgn_review_converter.py tools/collection_update.py tools/clean_book_descriptions.py tools/clean_book_analysis.py tools/clean_pgn.py
sh -n scripts/kindle_device_probe.sh
sh -n scripts/kindle_fbink_smoke.sh
sh -n scripts/kindle_capture_input.sh

python3 -m unittest tests/test_lifecycle.py tests/test_fbink_bridge.py tests/test_input_bridge.py
sh -n scripts/kindle_launch.sh

python3 -m unittest tests/test_packaging.py tests/test_chessnut_config.py tests/test_app_cli.py tests/test_split_baseline.py tests/test_platform_boundary.py tests/test_release_matrix.py
sh -n scripts/app.sh
python3 -m py_compile tools/kindle_build.py tools/verify_app_artifacts.py
for script in scripts/*-kindle.sh packaging/kindle/*.sh; do
    sh -n "$script"
done
