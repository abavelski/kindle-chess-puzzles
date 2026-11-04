#!/bin/sh
set -eu
experiment_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$experiment_dir"
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
