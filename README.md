# CCVer

> A zero dependency tool for conventional commits and semver

## Overview

CCVer is a command-line tool designed for automating version management in git repositories. It leverages conventional commit message conventions to help you:

- **Parse and validate commit messages**  
  Utilize a custom parser built with [Pest](https://pest.rs/) ([`src/parser/interpreter.rs`](src/parser/interpreter.rs) & [`src/parser/rules.pest`](src/parser/rules.pest)) to ensure commit messages adhere to a conventional format.

- **Automate semantic versioning**  
  Extract version and tagging information from commits to automatically bump version numbers following [semver](https://semver.org/) principles.

- [Git Log Flow](git-log-flow.mmd)

- [Version Example](version-example.mmd)

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

## Performance

CCVer is designed for speed and efficiency:

### Benchmarks

Performance characteristics for typical workloads:

| Repository Size | Parse Time | Graph Construction | Total Time |
|----------------|------------|-------------------|------------|
| 10 commits     | ~18 µs     | ~14 µs           | ~32 µs     |
| 100 commits    | ~159 µs    | ~122 µs          | ~281 µs    |
| 1000 commits   | ~1.99 ms   | ~1.27 ms         | ~3.26 ms   |

### Complexity

All operations scale linearly with repository size:

- **Log Parsing:** O(n) - ~600-650K elements/s throughput
- **Graph Construction:** O(n) - ~800K elements/s throughput
- **Version Lookup:** O(1) - ~57 ns via hash map
- **Parent/Child Ops:** O(1) - ~30-110 ns via memoization

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench parse_bench
cargo bench --bench graph_bench
cargo bench --bench command_bench
cargo bench --bench e2e_bench

# View HTML reports
open target/criterion/report/index.html
```

## Profiling

CCVer includes profiling capabilities using `cargo-flamegraph` and `pprof` to help analyze performance and identify bottlenecks.

### Profiling with cargo-flamegraph

Profile benchmarks using cargo-flamegraph to generate flamegraphs:

```bash
# Install cargo-flamegraph
cargo install flamegraph

# Profile specific benchmarks (automatically includes debug symbols)
cargo flamegraph --profile profiling --bench parse_bench
cargo flamegraph --profile profiling --bench graph_bench
cargo flamegraph --profile profiling --bench e2e_bench

# Profile the main binary
cargo flamegraph --profile profiling --bin ccver

# Flamegraphs are saved as flamegraph.svg in the current directory
```

### Ad-hoc Profiling

Use the profiling utility binary for targeted profiling:

```bash
# Profile specific operations (use --profile profiling for proper symbols)
cargo run --profile profiling --features profiling --bin profile -- parse 1000
cargo run --profile profiling --features profiling --bin profile -- graph 1000
cargo run --profile profiling --features profiling --bin profile -- e2e 1000

# Profile all operations
cargo run --profile profiling --features profiling --bin profile -- all 1000

# Custom output directory
cargo run --profile profiling --features profiling --bin profile -- parse 1000 --output my-profiles/

# Get help
cargo run --profile profiling --features profiling --bin profile -- --help
```

**Note:** Use `--profile profiling` instead of `--release` to ensure debug symbols are included for readable flamegraphs.

Flamegraphs are saved to `target/profiling/` by default.

### Running Regular Benchmarks

Run Criterion benchmarks without profiling:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench parse_bench
cargo bench --bench graph_bench

# View HTML reports
open target/criterion/report/index.html
```

### Interpreting Flamegraphs

Flamegraphs visualize where your program spends time:

- **Width**: Percentage of total time spent in a function
- **Height**: Call stack depth (bottom = entry point, top = leaf functions)
- **Colors**: Random, used only for visual distinction
- **Interactive**: Click to zoom into specific call stacks

Look for:
- Wide blocks indicating hot paths
- Unexpected function calls
- Optimization opportunities in frequently-called code

### CI Profiling

Profiling runs automatically in CI on pushes to main. View results:

```bash
# Flamegraphs are uploaded as GitHub Actions artifacts
# Download from: Actions → Profile → Artifacts
```

Manually trigger profiling:

```bash
# Via GitHub UI: Actions → Profile → Run workflow
```

## Software Bill of Materials (SBOM)

CCVer automatically generates comprehensive Software Bill of Materials (SBOM) for all releases and Docker images to support supply chain security and compliance.

### What is an SBOM?

An SBOM is a complete inventory of all components, libraries, and dependencies used in the software, enabling:

- **Supply Chain Security**: Track and verify all software components
- **Vulnerability Management**: Identify and respond to security issues quickly
- **Compliance**: Meet regulatory requirements (e.g., Executive Order 14028)
- **License Management**: Track open source licenses and obligations

### SBOM Formats

CCVer generates SBOMs in multiple industry-standard formats:

- **CycloneDX**: OWASP standard format optimized for software supply chain
- **SPDX**: Linux Foundation standard for license compliance
- **SARIF**: Security analysis format integrated with GitHub Security

### Automated SBOM Generation

SBOMs are automatically generated for:

1. **Cargo Dependencies**: Complete dependency tree from `Cargo.lock`
2. **Binary Analysis**: Deep analysis including system libraries
3. **Docker Images**: Full container image composition

The SBOM workflow (`docker.yml`) runs on:
- Pushes to `main`/`master` branches
- Tagged releases (`v*`)
- Pull requests modifying dependencies or source code
- Can be triggered manually via workflow_dispatch
- Called by other workflows via workflow_call

### Accessing SBOMs

#### From GitHub Artifacts

```bash
# Download SBOM artifacts from GitHub Actions
# Navigate to: Actions → Build and Push Docker Image with SBOM → Artifacts
```

#### From Docker Images

SBOMs are embedded in Docker images at `/usr/share/sbom/`:

```bash
# Extract SBOM from Docker image
docker run --rm ghcr.io/ryanguild/ccver:latest cat /usr/share/sbom/sbom-docker-cyclonedx.json

# Copy SBOM files from container
docker create --name temp ghcr.io/ryanguild/ccver:latest
docker cp temp:/usr/share/sbom ./sbom
docker rm temp
```

#### SBOM Attestation

Docker images include cryptographically signed SBOM attestations:

```bash
# Verify SBOM attestation using GitHub CLI
gh attestation verify oci://ghcr.io/ryanguild/ccver:latest --owner ryanguild

# View attestation details
gh attestation list --owner ryanguild --repo ccver
```

### SBOM Files

The following SBOM files are generated:

| File | Format | Description |
|------|--------|-------------|
| `sbom-cargo-cyclonedx.json` | CycloneDX | Rust dependency tree |
| `sbom-cargo-spdx.json` | SPDX | Rust dependency tree |
| `sbom-binary-cyclonedx.json` | CycloneDX | Binary analysis with system libs |
| `sbom-binary-spdx.json` | SPDX | Binary analysis with system libs |
| `sbom-binary.sarif` | SARIF | Security-focused analysis |
| `sbom-docker-cyclonedx.json` | CycloneDX | Complete Docker image |
| `sbom-docker-spdx.json` | SPDX | Complete Docker image |

### Supply Chain Security Features

- ✅ **Cryptographic Signing**: SBOMs are signed and verifiable
- ✅ **Provenance Tracking**: Full build transparency and reproducibility
- ✅ **Attestation Storage**: SBOMs stored in GitHub Container Registry
- ✅ **GitHub Security Integration**: SARIF format uploaded to Security tab
- ✅ **Sigstore Compatible**: Works with cosign and other verification tools

### Using SBOMs for Vulnerability Scanning

```bash
# Scan SBOM with Grype
grype sbom:./sbom-docker-cyclonedx.json

# Generate vulnerability report
grype sbom:./sbom-cargo-cyclonedx.json -o json > vulnerabilities.json

# Use with Trivy
trivy sbom ./sbom-docker-spdx.json
```

### Manual SBOM Generation

To generate SBOMs locally:

```bash
# Install cargo-sbom
cargo install cargo-sbom

# Generate Cargo-based SBOM
cargo sbom --output-format cyclone_dx_json_1_5 > sbom.json

# Install Syft
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin

# Generate binary SBOM
syft target/release/ccver -o cyclonedx-json=sbom.json
```
