ru#!/usr/bin/env bash
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

function build_x86_64-unknown-linux-gnu() {
    cross build --release --target x86_64-unknown-linux-gnu
    upx --best --lzma target/x86_64-unknown-linux-gnu/release/csv2parquet
}

function build_aarch64-unknown-linux-gnu() {
    cross build --release --target aarch64-unknown-linux-gnu
    upx --best --lzma target/aarch64-unknown-linux-gnu/release/csv2parquet
}

function build_arm-unknown-linux-gnueabi() {
    cross build --release --target arm-unknown-linux-gnueabi
    upx --best --lzma target/arm-unknown-linux-gnueabi/release/csv2parquet
}

function build_aarch64-apple-darwin () {
  MACOS_SDK_URL="https://github.com/joseluisq/macosx-sdks/releases/download/13.0/MacOSX13.0.sdk.tar.xz"
  MACOS_SDK_FILE="macos.sdk.tar.xz"
  wget -O "$MACOS_SDK_FILE" "$MACOS_SDK_URL"
  cargo build-docker-image aarch64-apple-darwin-cross --build-arg "MACOS_SDK_FILE=${MACOS_SDK_FILE}"
  cross build --release --target aarch64-apple-darwin
}

function install_cross () {
  workdir="$(mktemp --directory)"
  git clone --depth 1 --filter=blob:none --recurse-submodules -j"$(nproc)" --remote-submodules "https://github.com/cross-rs/cross" "$workdir"
  cd "$workdir"
  cargo xtask configure-crosstool
  cd -
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
