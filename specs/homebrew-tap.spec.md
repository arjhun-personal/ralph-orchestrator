---
status: draft
gap_analysis: 2026-01-14
---

# Homebrew Tap Specification

## Overview

Define the repository structure and automation for distributing Ralph via Homebrew. The tap repository (`mikeyobrien/homebrew-tap`) hosts the formula that installs pre-built binaries from GitHub Releases.

## Goals

1. **Zero friction install**: `brew install mikeyobrien/tap/ralph`
2. **Automated updates**: Formula auto-updates on new releases
3. **Binary distribution**: Install pre-built binaries, not compile from source
4. **Multi-platform support**: macOS x86_64 and arm64 (Apple Silicon)

## Repository Structure

### Location

- **GitHub repository**: `mikeyobrien/homebrew-tap`
- **Install command**: `brew install mikeyobrien/tap/ralph`
- **Tap add command**: `brew tap mikeyobrien/tap`

### Directory Layout

```
homebrew-tap/
├── Formula/
│   └── ralph.rb           # Main formula
├── README.md              # Tap documentation
└── .github/
    └── workflows/
        └── update.yml     # Auto-update workflow (optional)
```

## Formula Template

### Basic Structure

The formula downloads pre-built binaries from GitHub Releases:

```ruby
class Ralph < Formula
  desc "Multi-agent CLI orchestration framework"
  homepage "https://github.com/mikeyobrien/ralph-orchestrator"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/mikeyobrien/ralph-orchestrator/releases/download/vVERSION_PLACEHOLDER/ralph-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_PLACEHOLDER_ARM"
    else
      url "https://github.com/mikeyobrien/ralph-orchestrator/releases/download/vVERSION_PLACEHOLDER/ralph-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_PLACEHOLDER_X86"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/mikeyobrien/ralph-orchestrator/releases/download/vVERSION_PLACEHOLDER/ralph-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_PLACEHOLDER_LINUX_ARM"
    else
      url "https://github.com/mikeyobrien/ralph-orchestrator/releases/download/vVERSION_PLACEHOLDER/ralph-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_PLACEHOLDER_LINUX_X86"
    end
  end

  def install
    bin.install "ralph"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/ralph --version")
  end
end
```

### cargo-dist Integration

cargo-dist generates Homebrew formulas automatically when configured. The generated formula is placed in the `Formula/` directory of the tap repository.

Configuration in `Cargo.toml`:

```toml
[workspace.metadata.dist.homebrew]
tap = "mikeyobrien/tap"
formula = "ralph"
```

## Update Automation

### Option A: cargo-dist Homebrew Publisher (Recommended)

cargo-dist can publish directly to the tap repository during release. This requires:

1. A GitHub Personal Access Token (PAT) with `repo` scope
2. The token stored as `HOMEBREW_TAP_TOKEN` secret in the main repo
3. cargo-dist configured with `publish-jobs = ["homebrew"]`

Release workflow automatically:
1. Builds all platform binaries
2. Generates formula with correct URLs and SHA256 hashes
3. Opens PR or commits directly to the tap repository

### Option B: Manual Update Workflow

If automated publishing is not desired, a manual workflow in the tap repository:

```yaml
# .github/workflows/update.yml
name: Update Formula

on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to update to (e.g., 2.0.0)'
        required: true

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Update formula
        run: |
          VERSION="${{ github.event.inputs.version }}"
          # Download release assets and compute SHA256 hashes
          # Update Formula/ralph.rb with new version and hashes

      - name: Create PR
        uses: peter-evans/create-pull-request@v6
        with:
          title: "Update ralph to v${{ github.event.inputs.version }}"
          branch: "update-ralph-${{ github.event.inputs.version }}"
```

### Option C: Repository Dispatch from Main Repo

The main repo's release workflow triggers an update in the tap repo:

```yaml
# In ralph-orchestrator release.yml
- name: Trigger tap update
  uses: peter-evans/repository-dispatch@v3
  with:
    token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
    repository: mikeyobrien/homebrew-tap
    event-type: release
    client-payload: '{"version": "${{ github.ref_name }}"}'
```

## Initial Setup

### Creating the Tap Repository

1. Create repository `mikeyobrien/homebrew-tap` on GitHub
2. Add initial README.md:

```markdown
# Homebrew Tap

Custom Homebrew formulas for mikeyobrien projects.

## Installation

```bash
brew tap mikeyobrien/tap
brew install ralph
```

## Formulas

| Formula | Description |
|---------|-------------|
| ralph | Multi-agent CLI orchestration framework |
```

3. Create `Formula/` directory
4. Add placeholder formula (will be replaced on first release)

### Configuring Secrets

For automated publishing (Option A):

1. Create GitHub PAT with `repo` scope
2. Add to main repo as `HOMEBREW_TAP_TOKEN` secret
3. Enable cargo-dist Homebrew publishing

## Acceptance Criteria

### Installation

- **Given** the tap repository exists at `mikeyobrien/homebrew-tap`
- **When** user runs `brew install mikeyobrien/tap/ralph`
- **Then** the ralph binary is installed successfully

- **Given** user is on macOS arm64 (Apple Silicon)
- **When** installing ralph via Homebrew
- **Then** the aarch64-apple-darwin binary is downloaded

- **Given** user is on macOS x86_64
- **When** installing ralph via Homebrew
- **Then** the x86_64-apple-darwin binary is downloaded

### Formula Content

- **Given** a new release is published
- **When** the formula is updated
- **Then** version matches the release tag
- **And** SHA256 hashes match the release artifacts
- **And** download URLs point to correct release assets

### Update Automation

- **Given** cargo-dist Homebrew publisher is configured
- **When** a new release is tagged
- **Then** the tap repository is updated automatically
- **And** the formula contains correct version and hashes

### Version Test

- **Given** ralph is installed via Homebrew
- **When** user runs `ralph --version`
- **Then** the installed version is displayed

## Platform Coverage

| Platform | Architecture | Binary | Status |
|----------|--------------|--------|--------|
| macOS | arm64 (Apple Silicon) | ralph-aarch64-apple-darwin | Supported |
| macOS | x86_64 (Intel) | ralph-x86_64-apple-darwin | Supported |
| Linux | arm64 | ralph-aarch64-unknown-linux-gnu | Supported via Linuxbrew |
| Linux | x86_64 | ralph-x86_64-unknown-linux-gnu | Supported via Linuxbrew |

## Implementation Order

1. **Phase 1**: Create `mikeyobrien/homebrew-tap` repository with README
2. **Phase 2**: Add initial placeholder formula
3. **Phase 3**: Configure cargo-dist Homebrew publishing (add PAT secret)
4. **Phase 4**: Test first release updates formula correctly
5. **Phase 5**: Verify `brew install mikeyobrien/tap/ralph` works

## Notes

### Why Pre-built Binaries?

Homebrew supports both source builds and binary bottles. Using pre-built binaries:
- Faster installation (no compilation)
- No Rust toolchain required on user's machine
- Consistent binaries across all users
- cargo-dist already produces optimized release builds

### Tap vs Core

Using a tap (`mikeyobrien/tap`) instead of Homebrew Core because:
- Faster iteration (no Homebrew PR review cycle)
- Full control over formula updates
- Can support pre-release versions
- Suitable for tools that aren't yet widely adopted

### Linux Support

Linuxbrew (Homebrew on Linux) is supported. The formula includes Linux binary URLs for users who prefer Homebrew over other Linux package managers.
