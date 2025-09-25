# CCVer Docker & GitHub Action

This document describes how to use CCVer as a containerized tool and as a GitHub Action.

## Docker Usage

### Building the Docker Image

```bash
docker build -t ccver .
```

### Running CCVer in Docker

```bash
# Get version for current directory
docker run --rm -v "$(pwd):/github/workspace" ccver

# Get version with custom format
docker run --rm -v "$(pwd):/github/workspace" ccver --format "v{major}.{minor}.{patch}"

# Generate changelog
docker run --rm -v "$(pwd):/github/workspace" ccver changelog

# Check if repository is clean (CI mode)
docker run --rm -v "$(pwd):/github/workspace" ccver --ci
```

## GitHub Action Usage

### Basic Usage

Add this to your workflow file (`.github/workflows/release.yml`):

```yaml
name: Release

on:
  push:
    branches: [ main ]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0  # Fetch full history for ccver to analyze commits
    
    - name: Get Version
      id: version
      uses: ./  # Use this action from the same repository
      # Or use: uses: your-username/ccver@v1  # Use from another repository
      
    - name: Create Release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ steps.version.outputs.version }}
        release_name: Release ${{ steps.version.outputs.version }}
```

### Advanced Usage

```yaml
name: Advanced Release

on:
  push:
    branches: [ main ]

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Get Version
      id: version
      uses: ./
      with:
        format: 'v{major}.{minor}.{patch}-{prerelease}'
        no-pre: 'false'
        ci: 'true'
    
    - name: Generate Changelog
      id: changelog
      uses: ./
      with:
        command: 'changelog'
    
    - name: Create Release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ steps.version.outputs.version }}
        release_name: Release ${{ steps.version.outputs.version }}
        body: ${{ steps.changelog.outputs.changelog }}
```

### Action Inputs

| Input | Description | Required | Default |
|-------|-------------|----------|---------|
| `path` | Path to the git repository | No | `.` |
| `format` | Version format string | No | Default format |
| `no-pre` | Exclude pre-release identifiers | No | `false` |
| `ci` | Throw error if repository is dirty | No | `true` |
| `command` | CCVer subcommand to run | No | None |

### Action Outputs

| Output | Description |
|--------|-------------|
| `version` | The computed version string |
| `changelog` | Generated changelog (when `command=changelog`) |

### Example Workflows

#### Simple Version Tagging

```yaml
name: Tag Version

on:
  push:
    branches: [ main ]

jobs:
  tag:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Get Version
      id: version
      uses: ./
    
    - name: Create Tag
      run: |
        git tag ${{ steps.version.outputs.version }}
        git push origin ${{ steps.version.outputs.version }}
```

#### Multi-format Releases

```yaml
name: Multi-format Release

on:
  push:
    tags: [ 'v*' ]

jobs:
  release:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        format:
          - 'v{major}.{minor}.{patch}'
          - '{major}.{minor}.{patch}'
          - 'release-{major}-{minor}-{patch}'
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Get Version
      id: version
      uses: ./
      with:
        format: ${{ matrix.format }}
    
    - name: Print Version
      run: echo "Version with format '${{ matrix.format }}': ${{ steps.version.outputs.version }}"
```

## Docker Image Registry

The Docker image is automatically built and pushed to GitHub Container Registry (ghcr.io) when changes are pushed to the main branch or when tags are created.

You can pull the latest image with:

```bash
docker pull ghcr.io/your-username/ccver:latest
```

## Local Development

To test the action locally, you can use [act](https://github.com/nektos/act):

```bash
# Install act
# brew install act  # macOS
# sudo apt install act  # Ubuntu

# Run the action locally
act -j test
```
