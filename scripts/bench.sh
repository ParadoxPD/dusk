#!/usr/bin/env bash
set -euo pipefail

# Build release binary first so command-path benches can invoke it.
cargo build --release
cargo bench
