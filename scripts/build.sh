#!/usr/bin/env bash
set -euo pipefail

# Navigate to the project root directory
CDPATH= cd -- "$(dirname -- "$0")/.."

echo "==> Building in release mode..."
cargo build --release

echo "==> Copying binaries to dist/binaries..."
mkdir -p dist/binaries
cp target/release/beams dist/binaries/

echo "==> Build complete! Binary copied to dist/binaries/beams"
