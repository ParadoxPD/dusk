#!/usr/bin/env bash
set -e

TARGETS=(
  x86_64-unknown-linux-gnu
  x86_64-unknown-linux-musl
  x86_64-pc-windows-gnu
)

for t in "${TARGETS[@]}"; do
  echo "Building for $t"
  cargo build --release --target "$t" --all-features
done
