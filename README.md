# CCVer

> A zero dependency tool for conventional commits and semver

## Overview

CCVer is a command-line tool designed for automating version management in git repositories. It leverages conventional commit message conventions to help you:

- **Parse and validate commit messages**  
  Utilize a custom parser built with [Pest](https://pest.rs/) ([`src/parser/interpreter.rs`](src/parser/interpreter.rs) & [`src/parser/rules.pest`](src/parser/rules.pest)) to ensure commit messages adhere to a conventional format.

- **Automate semantic versioning**  
  Extract version and tagging information from commits to automatically bump version numbers following [semver](https://semver.org/) principles.


  ```mermaid
  flowchart TD
      A[git log] --> C[Parse raw logs with Pest]
      C --> D[Create DiGraph]
      D --> E[Construct Commit Graph<br>using `CommitGraphData::new`]
      E --> F[CommitGraph]
  ```

  ```mermaid
    gitGraph
        commit id: "initial commit" tag: "0.0.0"
        commit id: "unconventional commit" tag: "0.0.0-build.1"
        branch staging
        branch develop
        commit id: "feat: conventional commit" tag: "0.1.0-alpha.1"
        branch ryans-fix
        commit id: "chore: formatting" tag: "0.1.0-ryans-fix.1"
        checkout main
        merge ryans-fix id: "Merge branch 'ryans-fix'" tag: "0.1.0"
        checkout develop
        commit id: "fix: conventional commit" tag: "0.1.1-alpha.1"
        commit id: "whooops" tag: "0.1.1-alpha.2"
        checkout staging
        merge develop id: "Merge branch 'develop'" tag: "0.1.1-rc.1"
        merge main id: "Merge branch 'main'" tag: "0.1.1-rc.2"
        checkout main
        merge staging id: "Merge branch 'staging'" tag: "0.1.1"
        checkout develop
        commit type: HIGHLIGHT id: "uncommited changes" tag: "0.1.1-build.1"
  ```

- **Provide an extensible CLI**  
  Run various subcommands such as initializing (`Init`), installing hooks (`Install`), and tagging commits (`Tag`) to integrate version management into your workflow.

This tool is ideal for projects that want to maintain a clear commit history and manage releases automatically, all while ensuring that commit messages and version tags meet established conventions.

## Installation & Usage

### Package Managers

#### Homebrew (macOS/Linux)

```bash
brew tap ryanguild/tap
brew install ccver
```

#### Winget (Windows)

```powershell
winget install ryanguild.ccver
```

#### Debian/Ubuntu (APT)

```bash
# Add the repository
echo "deb [trusted=yes] https://ryanguild.github.io/ccver/apt stable main" | \
  sudo tee /etc/apt/sources.list.d/ccver.list

# Update and install
sudo apt update
sudo apt install ccver
```

#### Alpine Linux (APK)

```bash
# Download and trust the repository key
wget -O /etc/apk/keys/ccver.rsa.pub \
  https://ryanguild.github.io/ccver/apk/*.rsa.pub

# Add repository
echo "https://ryanguild.github.io/ccver/apk/v3.19/main" >> /etc/apk/repositories

# Update and install
apk update
apk add ccver
```

#### Cargo

```bash
cargo install ccver
```

### Direct Download

Download pre-built binaries from the [latest release](https://github.com/ryanguild/ccver/releases/latest):

**Linux (GNU libc):**
- `ccver-linux-amd64` - Standard Linux x86_64
- `ccver-linux-arm64` - Standard Linux ARM64

**Linux (musl - static, recommended for Docker/Alpine):**
- `ccver-linux-amd64-musl` - Static x86_64 (works on any Linux)
- `ccver-linux-arm64-musl` - Static ARM64 (works on any Linux)

**macOS:**
- `ccver-macos-amd64` - Intel Macs
- `ccver-macos-arm64` - Apple Silicon

**Windows:**
- `ccver-windows-amd64.exe` - Windows x86_64

```bash
# Example: Download and install musl build (works on any Linux distro)
curl -L -o ccver https://github.com/ryanguild/ccver/releases/latest/download/ccver-linux-amd64-musl
chmod +x ccver
sudo mv ccver /usr/local/bin/
```

### Local Installation

```bash
# Install from source
git clone https://github.com/ryanguild/ccver.git
cd ccver
cargo install --path .
```

### Development Setup

For contributors, set up pre-commit hooks to ensure code quality:

```bash
# Quick setup (installs pre-commit and configures hooks)
./setup-precommit.sh

# Manual setup
pip install pre-commit
pre-commit install
cargo build  # Build ccver for version management hook
```

The pre-commit hooks will automatically run on each commit and include:
- `cargo fmt` - Code formatting
- `cargo clippy` - Linting with warnings as errors
- `cargo test` - Run all tests
- Version update - Update `Cargo.toml` version using ccver itself
- YAML/TOML validation and other file checks

### Cross-Compilation

CCVer supports multiple platforms through Rust's cross-compilation. All targets are defined in `rust-toolchain.toml` and `.cargo/config.toml`.

#### Supported Platforms

- **Linux**: x86_64, ARM64 (both GNU and musl)
- **macOS**: x86_64 (Intel), ARM64 (Apple Silicon)
- **Windows**: x86_64, ARM64

#### Quick Build Commands

Use the predefined cargo aliases:

```bash
# Build for specific platform
cargo build-linux-x64
cargo build-linux-arm64
cargo build-macos-x64
cargo build-macos-arm64
cargo build-windows-x64
cargo build-windows-arm64

# Build all targets for a platform
cargo build-all-linux
cargo build-all-macos
cargo build-all-windows
```

#### Manual Cross-Compilation

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build for target
cargo build --release --target aarch64-unknown-linux-gnu
```

#### Using `cross` for Easy Cross-Compilation

For platforms without native toolchains:

```bash
# Install cross
cargo install cross

# Build with cross (handles toolchain setup)
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target x86_64-unknown-linux-musl
```

See `.cargo/config.toml` for detailed platform-specific notes and requirements.

### Docker Usage

```bash
# Build the Docker image
docker build -t ccver .

# Run ccver in a container
docker run --rm -v "$(pwd):/github/workspace" ccver
```

### GitHub Action Usage

Use CCVer directly in your GitHub workflows:

```yaml
- name: Get Version
  id: version
  uses: your-username/ccver@v1
  with:
    format: 'v{major}.{minor}.{patch}'

- name: Create Release
  uses: actions/create-release@v1
  with:
    tag_name: ${{ steps.version.outputs.version }}
```
