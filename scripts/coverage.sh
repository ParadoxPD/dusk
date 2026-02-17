#!/usr/bin/env bash
set -euo pipefail

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov not found. Install with: cargo install cargo-llvm-cov"
  exit 1
fi

# Strict gate for 100% line coverage.
cargo llvm-cov --workspace --all-features --fail-under-lines 100
