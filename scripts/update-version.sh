#!/bin/bash
# Script to update Cargo.toml version using ccver output

set -e

# Check if ccver binary exists
if [ ! -f target/debug/ccver ] && [ ! -f target/release/ccver ]; then
    echo "ccver binary not found. Please build the project first: cargo build"
    exit 0
fi

# Get version from ccver (with 'v' prefix)
NEW_VERSION_WITH_V=$(cargo run --quiet 2>/dev/null || echo "")

if [ -z "$NEW_VERSION_WITH_V" ]; then
    echo "Failed to get version from ccver"
    exit 0
fi

# Strip the 'v' prefix
NEW_VERSION=${NEW_VERSION_WITH_V#v}

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep "^version" Cargo.toml | cut -d'"' -f2)

# Compare and update if different
if [ "$NEW_VERSION" != "" ] && [ "$NEW_VERSION" != "$CURRENT_VERSION" ]; then
    # Create backup and update
    sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
    rm -f Cargo.toml.bak
    echo "Updated version from $CURRENT_VERSION to $NEW_VERSION"
    exit 0
else
    echo "Version is up to date ($CURRENT_VERSION)"
    exit 0
fi
