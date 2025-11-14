# Deployment Guide

**Build, distribution, and deployment instructions for DepMgr**

---

## Table of Contents

- [Overview](#overview)
- [Build Configurations](#build-configurations)
- [Building for Production](#building-for-production)
- [Binary Optimization](#binary-optimization)
- [Platform-Specific Builds](#platform-specific-builds)
- [Distribution Methods](#distribution-methods)
- [Installation Packaging](#installation-packaging)
- [CI/CD Integration](#cicd-integration)
- [Release Process](#release-process)

---

## Overview

This guide covers building and distributing production-ready binaries of DepMgr. The application is a **self-contained binary** with no runtime dependencies beyond the operating system.

**Key Points**:
- Written in Rust (compiles to native code)
- No runtime dependencies (statically linked where possible)
- Single binary deployment
- GUI application (not a CLI tool)

---

## Build Configurations

### Development Build

**Purpose**: Fast compilation, debug symbols, slower runtime

```bash
cargo build
```

**Output**: `target/debug/depmgr`

**Characteristics**:
- ✅ Fast compilation (30-60 seconds)
- ✅ Debug symbols included
- ✅ Better error messages
- ❌ Slower runtime (10-30% slower)
- ❌ Larger binary (~50 MB)

**Configuration** (implicit):
```toml
[profile.dev]
opt-level = 0
debug = true
```

### Release Build

**Purpose**: Maximum performance, optimized for distribution

```bash
cargo build --release
```

**Output**: `target/release/depmgr`

**Characteristics**:
- ✅ Maximum performance
- ✅ Smaller binary (~15-20 MB)
- ✅ No debug symbols
- ❌ Slower compilation (2-5 minutes)

**Configuration**: [`Cargo.toml:42-45`](Cargo.toml)

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
```

---

## Building for Production

### Step-by-Step Build Process

**1. Clone and Prepare**:

```bash
# Clone repository
git clone <repository-url>
cd xyz

# Verify Rust toolchain
rustc --version  # Should be 1.70+
cargo --version
```

**2. Clean Previous Builds**:

```bash
# Remove all build artifacts
cargo clean

# Verify clean state
ls target/  # Should not exist or be empty
```

**3. Build Release Binary**:

```bash
# Build with release optimizations
cargo build --release

# This takes 2-5 minutes on first build
# Progress shown in terminal
```

**4. Verify Build**:

```bash
# Check binary exists
ls -lh target/release/depmgr

# Expected output:
# -rwxr-xr-x  1 user  staff  15M Nov 14 12:00 depmgr

# Test run
./target/release/depmgr
```

**5. Strip Debug Symbols (Optional)**:

```bash
# Further reduce binary size (5-10% reduction)
strip target/release/depmgr

# Check new size
ls -lh target/release/depmgr
```

### Build Time Optimization

**Parallel Compilation**:

```bash
# Use all CPU cores (default)
cargo build --release

# Or specify core count
cargo build --release -j 8
```

**Incremental Builds**:

```bash
# First build: 2-5 minutes (full)
cargo build --release

# Subsequent builds: 30-60 seconds (incremental)
# Only recompiles changed code
```

**Caching Dependencies** (for CI/CD):

```bash
# Dependencies are cached in ~/.cargo/registry
# In CI, cache this directory:
# - ~/.cargo/registry
# - ~/.cargo/git
# - target/
```

---

## Binary Optimization

### Size Optimization

**Current Configuration** [`Cargo.toml:42-45`](Cargo.toml):

```toml
[profile.release]
opt-level = 3        # Max performance (larger binary)
lto = true           # Link-time optimization (reduces size)
codegen-units = 1    # Single codegen (reduces size)
```

**Alternative: Optimize for Size**:

```toml
[profile.release]
opt-level = "z"      # Optimize for size (not speed)
lto = true
codegen-units = 1
strip = true         # Auto-strip symbols
panic = "abort"      # Smaller panic handler
```

**Trade-off**: 20-30% smaller binary, 5-10% slower runtime

### Performance vs Size

| Configuration | Binary Size | Performance | Build Time |
|--------------|-------------|-------------|------------|
| `opt-level = 3` | ~15-20 MB | 100% | 2-5 min |
| `opt-level = "z"` | ~12-15 MB | 90-95% | 2-5 min |
| With `strip = true` | ~10-12 MB | Same | Same |

**Recommendation**: Use `opt-level = 3` (current) for best performance

### Runtime Performance Checks

**Profile Before Release**:

```bash
# Build with profiling
cargo build --release

# Run with timing
time ./target/release/depmgr

# Check memory usage (macOS)
/usr/bin/time -l ./target/release/depmgr
```

---

## Platform-Specific Builds

### macOS

**Native Build**:

```bash
# Build for current architecture
cargo build --release

# Output: Universal binary (Intel or Apple Silicon)
# Architecture detected automatically
```

**Universal Binary** (Intel + Apple Silicon):

```bash
# Install targets
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Build for Intel
cargo build --release --target x86_64-apple-darwin

# Build for Apple Silicon
cargo build --release --target aarch64-apple-darwin

# Combine into universal binary
lipo -create \
  target/x86_64-apple-darwin/release/depmgr \
  target/aarch64-apple-darwin/release/depmgr \
  -output depmgr-universal

# Verify
file depmgr-universal
# Output: Mach-O universal binary with 2 architectures
```

**Code Signing** (for distribution):

```bash
# Sign binary
codesign --sign "Developer ID Application: Your Name" \
         --timestamp \
         --options runtime \
         target/release/depmgr

# Verify signature
codesign --verify --verbose target/release/depmgr
```

### Linux

**Build for Linux**:

```bash
# Native build
cargo build --release

# Output: target/release/depmgr
```

**Static Linking** (portable binary):

```bash
# Install musl target
rustup target add x86_64-unknown-linux-musl

# Build with musl (static linking)
cargo build --release --target x86_64-unknown-linux-musl

# Output: Fully static binary (no system dependencies)
# Location: target/x86_64-unknown-linux-musl/release/depmgr
```

**Cross-Compilation from macOS**:

```bash
# Install cross-compilation tool
cargo install cross

# Build for Linux
cross build --release --target x86_64-unknown-linux-gnu

# Output: target/x86_64-unknown-linux-gnu/release/depmgr
```

### Windows

**Status**: Not yet supported (planned)

**Planned Build Commands**:

```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Cross-compile from macOS/Linux
cross build --release --target x86_64-pc-windows-msvc

# Output: target/x86_64-pc-windows-msvc/release/depmgr.exe
```

**Blockers**:
- GUI framework compatibility (egui works on Windows)
- Package manager detection needs Windows-specific logic
- Path handling differences

---

## Distribution Methods

### Method 1: Direct Binary Distribution

**Simplest approach**: Distribute raw binary

**Steps**:

```bash
# Build release binary
cargo build --release

# Create distribution directory
mkdir -p dist
cp target/release/depmgr dist/
cp README.md dist/
cp LICENSE dist/

# Create archive
tar -czf depmgr-v0.1.0-macos.tar.gz dist/

# Or create ZIP
zip -r depmgr-v0.1.0-macos.zip dist/
```

**Distribution**:
- Upload to GitHub Releases
- Users download and extract
- Move binary to `/usr/local/bin/` or run directly

### Method 2: Homebrew Formula (macOS)

**Create Homebrew Formula**:

```ruby
# Formula/depmgr.rb
class Depmgr < Formula
  desc "Unified package manager dashboard"
  homepage "https://github.com/yourusername/depmgr"
  url "https://github.com/yourusername/depmgr/archive/v0.1.0.tar.gz"
  sha256 "HASH_HERE"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release"
    bin.install "target/release/depmgr"
  end

  test do
    system "#{bin}/depmgr", "--version"
  end
end
```

**Installation**:

```bash
# Users install via:
brew install yourusername/tap/depmgr
```

### Method 3: Cargo Install (crates.io)

**Publish to crates.io**:

```bash
# 1. Create account on crates.io
# 2. Login
cargo login <your-token>

# 3. Verify Cargo.toml metadata
# [package]
# name = "depmgr"
# version = "0.1.0"
# authors = ["Your Name <email@example.com>"]
# description = "Unified package manager dashboard"
# license = "MIT"
# repository = "https://github.com/yourusername/depmgr"

# 4. Publish
cargo publish
```

**Installation**:

```bash
# Users install via:
cargo install depmgr
```

### Method 4: macOS .app Bundle

**Create Application Bundle**:

```bash
# Create .app structure
mkdir -p DepMgr.app/Contents/MacOS
mkdir -p DepMgr.app/Contents/Resources

# Copy binary
cp target/release/depmgr DepMgr.app/Contents/MacOS/

# Create Info.plist
cat > DepMgr.app/Contents/Info.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" 
          "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>depmgr</string>
    <key>CFBundleIdentifier</key>
    <string>com.yourdomain.depmgr</string>
    <key>CFBundleName</key>
    <string>DepMgr</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
</dict>
</plist>
EOF

# Create DMG (optional)
hdiutil create -volname "DepMgr" -srcfolder DepMgr.app -ov -format UDZO DepMgr-v0.1.0.dmg
```

**Distribution**: Users drag DepMgr.app to Applications folder

---

## Installation Packaging

### Debian/Ubuntu Package (.deb)

**Using `cargo-deb`**:

```bash
# Install cargo-deb
cargo install cargo-deb

# Configure in Cargo.toml
[package.metadata.deb]
maintainer = "Your Name <email@example.com>"
copyright = "2024, Your Name <email@example.com>"
license-file = ["LICENSE", "4"]
extended-description = """\
A unified package manager dashboard that provides a visual interface \
for managing packages across Homebrew, npm, cargo, and pip."""
depends = "$auto"
section = "utility"
priority = "optional"

# Build .deb package
cargo deb

# Output: target/debian/depmgr_0.1.0_amd64.deb
```

**Installation**:

```bash
sudo dpkg -i depmgr_0.1.0_amd64.deb
```

### RPM Package (Fedora/RHEL)

**Using `cargo-generate-rpm`**:

```bash
# Install tool
cargo install cargo-generate-rpm

# Build RPM
cargo build --release
cargo generate-rpm

# Output: target/generate-rpm/depmgr-0.1.0-1.x86_64.rpm
```

**Installation**:

```bash
sudo rpm -i depmgr-0.1.0-1.x86_64.rpm
```

---

## CI/CD Integration

### GitHub Actions

**`.github/workflows/release.yml`**:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]
        
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Cache target directory
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-target-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build release binary
        run: cargo build --release
      
      - name: Strip binary (Linux)
        if: matrix.os == 'ubuntu-latest'
        run: strip target/release/depmgr
      
      - name: Create archive
        run: |
          mkdir -p dist
          cp target/release/depmgr dist/
          cp README.md dist/
          cp LICENSE dist/
          tar -czf depmgr-${{ github.ref_name }}-${{ matrix.os }}.tar.gz dist/
      
      - name: Upload Release Asset
        uses: actions/upload-release-asset@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./depmgr-${{ github.ref_name }}-${{ matrix.os }}.tar.gz
          asset_name: depmgr-${{ github.ref_name }}-${{ matrix.os }}.tar.gz
          asset_content_type: application/gzip
```

### Build Matrix

**Multi-Platform Builds**:

```yaml
strategy:
  matrix:
    include:
      - os: macos-latest
        target: x86_64-apple-darwin
        artifact_name: depmgr-macos-intel
      - os: macos-latest
        target: aarch64-apple-darwin
        artifact_name: depmgr-macos-arm64
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        artifact_name: depmgr-linux-amd64
      - os: ubuntu-latest
        target: x86_64-unknown-linux-musl
        artifact_name: depmgr-linux-musl
```

---

## Release Process

### Version Management

**Update Version**:

```toml
# Cargo.toml
[package]
version = "0.2.0"
```

**Version Scheme**: Semantic Versioning (SemVer)
- `0.1.0` → Initial release
- `0.2.0` → New features (backward compatible)
- `1.0.0` → Stable API
- `1.1.0` → Minor feature additions
- `1.0.1` → Bug fixes

### Release Checklist

**Pre-Release**:

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run all tests: `cargo test`
- [ ] Run linter: `cargo clippy`
- [ ] Format code: `cargo fmt`
- [ ] Build release: `cargo build --release`
- [ ] Test binary: `./target/release/depmgr`
- [ ] Update documentation
- [ ] Commit changes
- [ ] Create git tag: `git tag -a v0.2.0 -m "Release v0.2.0"`

**Release**:

```bash
# Push tag to trigger CI
git push origin v0.2.0

# CI automatically:
# - Builds for all platforms
# - Creates GitHub Release
# - Uploads artifacts
```

**Post-Release**:

- [ ] Verify GitHub Release created
- [ ] Download and test artifacts
- [ ] Update Homebrew formula (if applicable)
- [ ] Publish to crates.io: `cargo publish`
- [ ] Update documentation website
- [ ] Announce release (blog, Twitter, etc.)

### Changelog Format

**`CHANGELOG.md`**:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2024-11-20

### Added
- yarn, pnpm, gem support
- Disk cache layer for instant cold starts
- Export package lists (JSON, CSV)

### Changed
- Improved description fetching (2x faster)
- Updated egui to 0.34

### Fixed
- Fix crash on empty package lists
- Fix timeout issues on slow connections

## [0.1.0] - 2024-11-14

### Added
- Initial release
- Homebrew, npm, cargo, pip support
- GUI dashboard with resizable columns
- Project usage scanning
- Update/remove/reinstall operations
```

---

## Deployment Verification

### Pre-Deployment Testing

**Manual Tests**:

```bash
# Build binary
cargo build --release

# Test basic functionality
./target/release/depmgr

# Verify package managers detected
# Verify packages load
# Verify update works
# Verify search/filter works
```

**Automated Tests** (future):

```bash
# Run test suite
cargo test

# Run integration tests
cargo test --test integration

# Run benchmarks
cargo bench
```

### Post-Deployment Verification

**Verify Download**:

```bash
# Download from GitHub Releases
curl -LO https://github.com/user/depmgr/releases/download/v0.1.0/depmgr-macos.tar.gz

# Extract
tar -xzf depmgr-macos.tar.gz

# Run
./dist/depmgr
```

**Verify Installation** (Homebrew):

```bash
# Install
brew install yourusername/tap/depmgr

# Verify
which depmgr
depmgr --version

# Run
depmgr
```

---

## Troubleshooting Builds

### Common Build Issues

**1. "linker error" or "cannot find -lssl"**:

```bash
# macOS
xcode-select --install

# Linux (Ubuntu/Debian)
sudo apt install build-essential libssl-dev pkg-config

# Linux (Fedora/RHEL)
sudo dnf install gcc openssl-devel pkg-config
```

**2. "failed to compile" with Rust version error**:

```bash
# Update Rust
rustup update stable
rustc --version  # Verify 1.70+
```

**3. Out of memory during build**:

```bash
# Reduce parallel jobs
cargo build --release -j 2

# Or build with less optimization
cargo build --release
```

**4. Slow builds**:

```bash
# Install sccache (build cache)
cargo install sccache

# Configure in ~/.cargo/config.toml
[build]
rustc-wrapper = "sccache"
```

---

## Further Reading

- **[SETUP.md](SETUP.md)** - Installation and configuration
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow
- **[README.md](README.md)** - Project overview

---

**Deployment documentation complete.** For CI/CD pipeline setup, consult your platform's documentation (GitHub Actions, GitLab CI, etc.).

