#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
set -euo pipefail

CARGO_BUILD_TARGET="${CARGO_BUILD_TARGET:-}"
if [[ -z "$CARGO_BUILD_TARGET" ]]; then
  echo "CARGO_BUILD_TARGET is not set. Please set it to the desired target."
  exit 1
fi

DOCKER_IMAGE_TAG="${DOCKER_IMAGE_TAG:-}"
if [[ -z "$DOCKER_IMAGE_TAG" ]]; then
  echo "DOCKER_IMAGE_TAG is not set. Please set according to https://github.com/rust-cross/rust-musl-cross"
  exit 1
fi

DOCKER_IMAGE="ghcr.io/rust-cross/rust-musl-cross:${DOCKER_IMAGE_TAG}"

# Build the project
echo "Building the project for target: $CARGO_BUILD_TARGET"
docker run \
  --pull always \
  --rm \
  -v "$(pwd)":/home/rust/src \
  -v "$HOME/.cargo/registry/":"/root/.cargo/registry/" \
  -v "$HOME/.cargo/git/":"/root/.cargo/git/" \
  "$DOCKER_IMAGE" \
    cargo build --release

APP_NAME="csv2parquet"
mkdir -p dist
BUILT_FILE="target/${CARGO_BUILD_TARGET}/release/${APP_NAME}"
DIST_FILE="dist/${APP_NAME}-${CARGO_BUILD_TARGET}"
cp -v "$BUILT_FILE" "$DIST_FILE"
upx --best --lzma "$DIST_FILE"
