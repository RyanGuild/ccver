#!/bin/bash
# Script to update Cargo.toml version using ccver peek command

set -e

# Check if ccver binary exists
if [ ! -f target/debug/ccver ] && [ ! -f target/release/ccver ]; then
    echo "ccver binary not found. Please build the project first: cargo build"
    exit 0
fi

# Get the commit message from git (staged commit)
# In pre-commit context, we need to get the message from the commit being made
COMMIT_MESSAGE=""

# Try to get commit message from various sources
if [ ! -z "$1" ]; then
    # If message passed as argument, use it
    COMMIT_MESSAGE="$1"
elif [ -f ".git/COMMIT_EDITMSG" ]; then
    COMMIT_MESSAGE=$(cat .git/COMMIT_EDITMSG | head -n 1)
else
    # Fallback: try to get from git log if available
    COMMIT_MESSAGE=$(git log -1 --pretty=format:"%s" 2>/dev/null || echo "chore: update version")
fi

if [ -z "$COMMIT_MESSAGE" ]; then
    echo "Could not determine commit message, using current version"
    NEW_VERSION_WITH_V=$(cargo run --quiet 2>/dev/null || echo "")
else
    echo "Using commit message: $COMMIT_MESSAGE"
    # Use peek command to get version for this commit message
    NEW_VERSION_WITH_V=$(cargo run --quiet -- peek --message "$COMMIT_MESSAGE" 2>/dev/null || echo "")
fi

if [ -z "$NEW_VERSION_WITH_V" ]; then
    echo "Failed to get version from ccver"
    exit 0
fi

# Strip the 'v' prefix and any build metadata (commit hash)
NEW_VERSION=${NEW_VERSION_WITH_V#v}
NEW_VERSION=${NEW_VERSION%+*}

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
