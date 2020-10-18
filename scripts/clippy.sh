#!/usr/bin/env bash

set -e
cargo-clippy --all --tests -- -Dwarnings

# if [[ "$OSTYPE" == "darwin"* ]]; then
#   cd fifthtry_local
#   # cargo-clippy --all --tests -- -Dwarnings
#   # cargo test --all
# fi
