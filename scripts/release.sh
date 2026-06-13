#!/usr/bin/env bash
set -euo pipefail

# Navigate to the project root directory
CDPATH= cd -- "$(dirname -- "$0")/.."

echo "==> Running test script..."
./scripts/test.sh

echo "==> Running build script..."
./scripts/build.sh

# Extract version dynamically from Cargo.toml
VERSION=$(cargo metadata --format-version 1 | grep -o '"version":"[^"]*"' | head -n 1 | cut -d'"' -f4)
TAG="v$VERSION"

echo "==> Tagging Git release as $TAG..."
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "Tag $TAG already exists, skipping tag creation."
else
    git tag -a "$TAG" -m "Release $TAG"
    echo "Git tag $TAG created."
fi

echo "==> Pushing tags to origin..."
# Check if there is an origin remote configured before pushing
if git remote | grep -q 'origin'; then
    git push origin "$TAG"
else
    echo "No remote 'origin' configured. Skipping push."
fi

echo "==> Release $TAG complete!"
