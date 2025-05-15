#!/usr/bin/env bash
cd "$(dirname "$0")/.." || exit 1
set -euo pipefail

# deduct version from Cargo.toml
VERSION=$(grep -m 1 '^version = ' Cargo.toml | cut -d '"' -f 2)
echo "Releasing version $VERSION"

# ensure github cli is available
if ! command -v gh &> /dev/null; then
    echo "gh could not be found. Please install it from https://cli.github.com/"
    exit 1
fi

# release the version
gh release create "$VERSION" \
    --generate-notes \
    ./dist/* \
;
