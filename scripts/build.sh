#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
set -euo pipefail

CARGO_BUILD_TARGET="${CARGO_BUILD_TARGET:-}"
if [[ -z "$CARGO_BUILD_TARGET" ]]; then
  echo "CARGO_BUILD_TARGET is not set. Please set it to the desired target."
  exit 1
fi

function build_x86_64-pc-windows-gnu() {
    cross build --release --target x86_64-pc-windows-gnu
    upx --best --lzma target/x86_64-pc-windows-gnu/release/csv2parquet.exe
}

function build_aarch64-linux-android() {
    cross build --release --target aarch64-linux-android
    upx --best --lzma target/x86_64-pc-windows-gnu/release/csv2parquet.exe
}

case $CARGO_BUILD_TARGET in
x86_64-pc-windows-gnu)
  build_x86_64-pc-windows-gnu
  ;;
  *)
    echo "Invalid build target $CARGO_BUILD_TARGET"
    exit 1
    ;;
esac
