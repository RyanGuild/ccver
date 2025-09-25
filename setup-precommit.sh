#!/bin/bash
# Setup script for pre-commit hooks in ccver project

set -e

echo "🔧 Setting up pre-commit hooks for ccver..."

# Check if pre-commit is installed
if ! command -v pre-commit &> /dev/null; then
    echo "📦 Installing pre-commit..."

    # Try different installation methods
    if command -v pip &> /dev/null; then
        pip install pre-commit
    elif command -v pip3 &> /dev/null; then
        pip3 install pre-commit
    elif command -v brew &> /dev/null; then
        brew install pre-commit
    else
        echo "❌ Could not find pip or brew to install pre-commit."
        echo "Please install pre-commit manually:"
        echo "  - Using pip: pip install pre-commit"
        echo "  - Using homebrew: brew install pre-commit"
        echo "  - Using conda: conda install -c conda-forge pre-commit"
        exit 1
    fi
else
    echo "✅ pre-commit is already installed"
fi

# Install the hooks
echo "🪝 Installing pre-commit hooks..."
pre-commit install

# Build the project first to ensure ccver binary exists for version updates
echo "🔨 Building ccver for version management..."
cargo build

echo "🧪 Running pre-commit hooks on all files to test setup..."
pre-commit run --all-files || {
    echo "⚠️  Some hooks failed on first run - this is normal for formatting hooks"
    echo "   Run 'pre-commit run --all-files' again to see if issues are resolved"
}

echo ""
echo "🎉 Pre-commit setup complete!"
echo ""
echo "📋 Configured hooks:"
echo "   • cargo fmt      - Format Rust code"
echo "   • cargo clippy   - Lint Rust code"
echo "   • cargo test     - Run tests"
echo "   • update-version - Update Cargo.toml version using ccver"
echo "   • check-yaml     - Validate YAML files"
echo "   • check-toml     - Validate TOML files"
echo "   • trailing-whitespace - Remove trailing whitespace"
echo "   • end-of-file-fixer   - Ensure files end with newline"
echo "   • check-merge-conflict - Check for merge conflicts"
echo "   • check-added-large-files - Prevent large files"
echo ""
echo "💡 Tips:"
echo "   • Hooks run automatically on 'git commit'"
echo "   • Run manually: pre-commit run --all-files"
echo "   • Skip hooks: git commit --no-verify"
echo "   • Update hooks: pre-commit autoupdate"
