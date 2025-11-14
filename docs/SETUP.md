# Setup and Installation Guide

**Complete setup instructions for DepMgr - Unified Package Manager Dashboard**

---

## Table of Contents

- [System Requirements](#system-requirements)
- [Prerequisites](#prerequisites)
- [Installation Methods](#installation-methods)
- [First Run](#first-run)
- [Configuration](#configuration)
- [Platform-Specific Setup](#platform-specific-setup)
- [Development Setup](#development-setup)
- [Verification](#verification)
- [Troubleshooting](#troubleshooting)

---

## System Requirements

### Operating System

| Platform | Status | Notes |
|----------|--------|-------|
| **macOS** | ✅ Fully Supported | Primary development platform (10.15+) |
| **Linux** | 🔄 Planned | Should work, not extensively tested |
| **Windows** | 🔄 Planned | Requires Windows-specific adjustments |

### Hardware

**Minimum**:
- **CPU**: 2 cores, 1 GHz
- **RAM**: 512 MB available memory
- **Disk**: 50 MB for binary + cache space

**Recommended**:
- **CPU**: 4+ cores (better parallel processing)
- **RAM**: 2 GB available memory
- **Disk**: 200 MB (for cache and package metadata)

### Package Managers

At least **one** of the following must be installed:

| Manager | Detection Command | Installation |
|---------|-------------------|--------------|
| **Homebrew** | `brew --version` | [brew.sh](https://brew.sh/) |
| **npm** | `npm --version` | Comes with Node.js from [nodejs.org](https://nodejs.org/) |
| **Cargo** | `cargo --version` | Comes with Rust from [rustup.rs](https://rustup.rs/) |
| **pip** | `pip3 --version` | Comes with Python 3 |

**Detection Logic**: [`src/managers/detector.rs`](src/managers/detector.rs)

---

## Prerequisites

### 1. Install Rust

DepMgr is written in Rust and requires the Rust toolchain to build from source.

**Installation**:

```bash
# Install Rust via rustup (official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the on-screen instructions
# Restart your terminal after installation

# Verify installation
rustc --version  # Should show: rustc 1.70.0 or later
cargo --version  # Should show: cargo 1.70.0 or later
```

**Rust Version**: 1.70+ required (uses Rust 2021 edition)

**Configuration**: [`Cargo.toml:3`](Cargo.toml)

```toml
[package]
edition = "2021"
```

### 2. Install Package Managers (Optional)

Install any package managers you want to manage with DepMgr:

**Homebrew (macOS/Linux)**:
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**npm (via Node.js)**:
```bash
# macOS with Homebrew
brew install node

# Or download from https://nodejs.org/
```

**Cargo (via Rust)**:
```bash
# Installed automatically with Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**pip (via Python)**:
```bash
# macOS with Homebrew
brew install python3

# pip3 is included with Python 3
```

---

## Installation Methods

### Method 1: Build from Source (Recommended)

**Step 1: Clone the Repository**

```bash
# Clone the repository
git clone <repository-url>
cd xyz
```

**Step 2: Build Release Binary**

```bash
# Build optimized release binary
cargo build --release

# This takes 2-5 minutes on first build
# Subsequent builds are faster (incremental compilation)
```

**Build Configuration**: [`Cargo.toml:42-45`](Cargo.toml)

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization (slower build, faster runtime)
codegen-units = 1      # Single codegen unit for better optimization
```

**Step 3: Locate the Binary**

```bash
# Binary location
ls -lh target/release/depmgr

# Size: ~15-20 MB (stripped, optimized)
```

**Step 4: Run the Application**

```bash
# Run directly
./target/release/depmgr

# Or add to PATH for system-wide access
sudo cp target/release/depmgr /usr/local/bin/
depmgr  # Run from anywhere
```

### Method 2: Development Build (Faster Compilation)

**For rapid development and testing** (slower runtime, faster compilation):

```bash
# Clone repository
git clone <repository-url>
cd xyz

# Run directly (builds and runs in one step)
cargo run

# Or build separately
cargo build
./target/debug/depmgr
```

**Trade-offs**:
- ✅ Faster compilation (30-60 seconds vs 2-5 minutes)
- ✅ Includes debug symbols (better error messages)
- ❌ Slower runtime (10-30% slower)
- ❌ Larger binary size (~50 MB)

### Method 3: Direct Cargo Install (Future)

**Once published to crates.io**:

```bash
# Install from crates.io (not yet available)
cargo install depmgr

# Binary installed to ~/.cargo/bin/depmgr
depmgr
```

---

## First Run

### Startup Process

When you launch DepMgr for the first time:

**Step 1: Window Initialization** ([`src/main.rs:11-18`](src/main.rs))
- Creates 1200×800 GUI window
- Minimum size: 800×600
- Window title: "Dependency Manager"

**Step 2: Package Manager Detection** ([`src/main.rs:28-36`](src/main.rs))
- Checks for available package managers (`brew`, `npm`, `cargo`, `pip`)
- Detection is **blocking but fast** (<1 second)
- Runs: `<manager> --version` for each potential manager

**Step 3: Initial Scan** ([`src/app.rs:41-179`](src/app.rs))
- Starts **async** package scanning (non-blocking)
- Scans all detected package managers **in parallel**
- UI updates incrementally as packages are found

**Timeline**:

```
0s     → Window appears
0-1s   → "Scanning packages..." message appears
1-5s   → First packages appear in table
5-30s  → Descriptions and project usage fill in
30-60s → All data complete (first run)
```

**Subsequent Runs**:
- Cached data loads in <100ms
- Cache TTL: 1 hour ([`src/utils/cache.rs`](src/utils/cache.rs))

### What You'll See

1. **Sidebar (Left)**:
   - Available package managers with checkboxes
   - Statistics: Total, Outdated, Unused counts
   - Refresh button
   - "Update All" button (if outdated packages exist)

2. **Main Panel (Center)**:
   - Search bar
   - Filter toggles (Outdated Only, Orphaned Only)
   - Package table with resizable columns
   - Real-time status messages

3. **Package Table**:
   - **Columns**: Name, Manager, Installed, Latest, Description, Usage, Status, Action
   - **Color Coding**: Green (current), Orange (outdated), Red (unused)
   - **Actions**: Update, Remove, Reinstall buttons

**UI Implementation**: [`src/ui/dashboard.rs:5-292`](src/ui/dashboard.rs)

---

## Configuration

### Default Configuration

DepMgr works out-of-the-box with **zero configuration required**. All settings are auto-detected.

### Scan Directories

**Where DepMgr looks for projects**:

Default scan paths: [`src/scanner/mod.rs`](src/scanner/mod.rs)

```rust
pub fn get_scan_directories() -> Vec<PathBuf> {
    vec![
        home_dir.join("Desktop"),
        home_dir.join("Documents"),
        home_dir.join("projects"),
        home_dir.join("dev"),
        home_dir.join("Developer"),
        home_dir.join("code"),
        home_dir.join("workspace"),
    ]
}
```

**Scan Depth**: Up to 4 levels deep ([`src/scanner/project_scanner.rs:21-22`](src/scanner/project_scanner.rs))

**Excluded Directories**: `.git`, `node_modules`, `target`, `dist`, `build`, `__pycache__` ([`src/scanner/project_scanner.rs:24-33`](src/scanner/project_scanner.rs))

### Custom Configuration (Future Enhancement)

**Planned**: `~/.config/depmgr/config.toml`

```toml
[scan]
directories = [
    "~/projects",
    "~/work",
    "/opt/projects"
]
max_depth = 4

[cache]
ttl_seconds = 3600  # 1 hour

[ui]
theme = "dark"  # or "light"
default_sort = "name"  # or "manager", "status"
```

**Note**: Custom configuration is not yet implemented but planned for v0.2.0

---

## Platform-Specific Setup

### macOS

**Recommended Setup**:

```bash
# 1. Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Clone and build DepMgr
git clone <repository-url>
cd xyz
cargo build --release

# 4. Run
./target/release/depmgr
```

**Optional: Install to Applications**:

```bash
# Create .app bundle (future enhancement)
# For now, add to PATH or create alias

# Add to PATH
sudo cp target/release/depmgr /usr/local/bin/

# Or create alias in ~/.zshrc or ~/.bash_profile
alias depmgr='/path/to/xyz/target/release/depmgr'
```

### Linux

**Prerequisites**:

```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential curl libssl-dev pkg-config

# Fedora/RHEL
sudo dnf install gcc curl openssl-devel pkg-config

# Arch
sudo pacman -S base-devel curl openssl pkg-config
```

**Installation**:

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 2. Clone and build
git clone <repository-url>
cd xyz
cargo build --release

# 3. Run
./target/release/depmgr
```

**Note**: Linux support is experimental. Please report issues.

### Windows

**Status**: Not yet supported

**Planned Support**: Windows 10+ with WSL2 or native builds

**Blockers**:
- GUI framework compatibility (egui should work)
- Package manager detection needs Windows-specific commands
- Path handling differences

---

## Development Setup

### Full Development Environment

**Step 1: Clone and Setup**

```bash
# Clone repository
git clone <repository-url>
cd xyz

# Build in debug mode (includes debug symbols)
cargo build
```

**Step 2: Install Development Tools**

```bash
# Rust tools
rustup component add rustfmt     # Code formatter
rustup component add clippy      # Linter

# Optional: cargo-watch for auto-rebuild
cargo install cargo-watch
```

**Step 3: Run in Development Mode**

```bash
# Run directly (fastest for development)
cargo run

# Or with auto-reload on file changes
cargo watch -x run

# Run tests (when available)
cargo test
```

### IDE Setup

**VS Code** (Recommended):

1. Install extensions:
   - **rust-analyzer**: IntelliSense, code completion
   - **CodeLLDB**: Debugging support
   - **Even Better TOML**: Cargo.toml syntax

2. Settings (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy"
   }
   ```

**IntelliJ IDEA / CLion**:

1. Install **Rust plugin**
2. Import project: File → Open → Select `xyz` directory
3. Wait for Cargo indexing to complete

### Code Quality Checks

```bash
# Type checking (fast, no code generation)
cargo check

# Linting with auto-fix
cargo clippy --fix

# Formatting
cargo fmt

# Run all checks
cargo check && cargo clippy && cargo fmt --check
```

**CI/CD Integration**: These commands should be part of your CI pipeline.

---

## Verification

### Verify Installation

**Check Binary**:

```bash
# Verify binary exists and is executable
ls -lh target/release/depmgr

# Expected output:
# -rwxr-xr-x  1 user  staff    15M Nov 14 12:00 depmgr
```

**Run Version Check** (Future):

```bash
# Display version and build info (not yet implemented)
./target/release/depmgr --version

# Planned output:
# depmgr 0.1.0
# Built with Rust 1.70.0
```

### Verify Package Managers

**Check Detection**:

```bash
# Run the app
./target/release/depmgr

# Check console output:
# [DEBUG] Found 4 package managers
# [DEBUG] Scanning Homebrew packages...
# [DEBUG] Scanning npm packages...
# [DEBUG] Scanning cargo packages...
# [DEBUG] Scanning pip packages...
```

**Manual Detection Test**:

```bash
# Test each package manager manually
brew --version     # Should output version or "command not found"
npm --version
cargo --version
pip3 --version
```

### Verify Functionality

**Test Checklist**:

- [ ] Application window opens
- [ ] Sidebar shows detected package managers
- [ ] Packages appear in table within 60 seconds
- [ ] Search filter works (type package name)
- [ ] "Outdated Only" filter works
- [ ] Column resizing works (drag column dividers)
- [ ] "Update" button appears for outdated packages
- [ ] "Refresh" button triggers rescan

**Performance Check**:

```bash
# First run (no cache)
time ./target/release/depmgr
# Should complete scan in 30-60 seconds

# Second run (cached)
# Packages should appear in <1 second
```

---

## Troubleshooting

### Common Issues

#### 1. "Command not found: cargo"

**Problem**: Rust toolchain not installed or not in PATH

**Solution**:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or source the env
source $HOME/.cargo/env

# Verify
cargo --version
```

#### 2. Build Fails with "linker error"

**Problem**: Missing system dependencies

**Solution (macOS)**:

```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Solution (Linux)**:

```bash
# Ubuntu/Debian
sudo apt install build-essential libssl-dev pkg-config

# Fedora
sudo dnf install gcc openssl-devel pkg-config
```

#### 3. "No packages found" on First Run

**Problem**: No package managers detected

**Solution**:

```bash
# Verify at least one package manager is installed
brew --version || echo "Homebrew not found"
npm --version || echo "npm not found"
cargo --version || echo "cargo not found"
pip3 --version || echo "pip not found"

# Install at least one package manager
# See Prerequisites section above
```

#### 4. Application Hangs During Scan

**Problem**: Timeout or network issue with package manager

**Solution**:

```bash
# Check package manager commands manually
brew list --versions  # Should complete in 1-2 seconds
npm list -g --depth=0 --json  # Should complete in 1-2 seconds

# Check internet connection (Homebrew API requires network)
curl -I https://formulae.brew.sh/api/formula.json

# If hangs persist, check logs in terminal for error messages
```

#### 5. Slow Performance (5+ minutes)

**Problem**: First-run API fetch or slow network

**Expected**: First run takes 30-60 seconds. Cached runs are <1 second.

**Solution**:

```bash
# Check network speed
time curl -I https://formulae.brew.sh/api/formula.json

# If slow, wait for first scan to complete and cache
# Subsequent runs will be instant

# Force cache clear (if needed):
# Cache is in-memory only, restart app to clear
```

#### 6. Build Fails: "failed to compile"

**Problem**: Rust version too old

**Solution**:

```bash
# Update Rust to latest stable
rustup update stable
rustc --version  # Should be 1.70+

# Retry build
cargo clean
cargo build --release
```

### Getting Help

**Console Output**: Check terminal for debug messages

```bash
# Run with debug output
./target/release/depmgr 2>&1 | tee depmgr.log

# Look for error messages starting with [ERROR]
```

**Check Dependencies**:

```bash
# Verify all dependencies are available
cargo tree

# Check for version conflicts
cargo tree --duplicates
```

**Full Rebuild**:

```bash
# Clean all build artifacts
cargo clean

# Rebuild from scratch
cargo build --release
```

**Consult Documentation**:
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Detailed troubleshooting guide
- **[docs/worklog.md](docs/worklog.md)** - Development history and known issues
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Technical details

---

## Next Steps

After successful installation:

1. **Explore the UI**: Try search, filters, and column resizing
2. **Update Packages**: Test the "Update" button on outdated packages
3. **Review Usage**: Check which projects use which packages
4. **Customize**: Add your project directories to scan paths (future feature)

**Further Reading**:
- **[README.md](README.md)** - Feature overview and quick start
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and internals
- **[API.md](API.md)** - Internal API documentation
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development guidelines

---

**Setup Complete!** 🎉

You're now ready to manage all your packages in one unified dashboard.

