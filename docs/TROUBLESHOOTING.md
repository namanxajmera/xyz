# Troubleshooting Guide

**Common issues and solutions for DepMgr**

---

## Table of Contents

- [Installation Issues](#installation-issues)
- [Build Problems](#build-problems)
- [Runtime Errors](#runtime-errors)
- [Performance Issues](#performance-issues)
- [Package Manager Issues](#package-manager-issues)
- [UI Problems](#ui-problems)
- [Platform-Specific Issues](#platform-specific-issues)
- [Debug Mode](#debug-mode)
- [Getting Help](#getting-help)

---

## Installation Issues

### Issue: "command not found: cargo"

**Symptom**: Cannot build DepMgr, `cargo` command not recognized

**Cause**: Rust toolchain not installed or not in PATH

**Solution**:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or source environment
source $HOME/.cargo/env

# Verify
cargo --version
```

**Code Reference**: Prerequisites in [`SETUP.md`](SETUP.md)

---

### Issue: Rust version too old

**Symptom**: Build fails with "edition 2021 is unstable" or similar

**Cause**: Rust 1.70+ required

**Solution**:

```bash
# Update Rust to latest stable
rustup update stable

# Verify version
rustc --version  # Should show 1.70.0 or later
```

**Project Configuration**: [`Cargo.toml:3`](Cargo.toml) - `edition = "2021"`

---

## Build Problems

### Issue: "linker 'cc' not found"

**Symptom**: Build fails during linking phase

**Cause**: Missing C compiler

**Solution (macOS)**:

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify
cc --version
```

**Solution (Linux - Ubuntu/Debian)**:

```bash
sudo apt update
sudo apt install build-essential
```

**Solution (Linux - Fedora/RHEL)**:

```bash
sudo dnf install gcc
```

---

### Issue: "error: failed to run custom build command for `openssl-sys`"

**Symptom**: Build fails with OpenSSL errors

**Cause**: Missing OpenSSL development headers

**Solution (macOS)**:

```bash
# Usually included with Xcode, but if needed:
brew install openssl@3

# Set environment variables
export OPENSSL_DIR=$(brew --prefix openssl@3)
```

**Solution (Linux - Ubuntu/Debian)**:

```bash
sudo apt install libssl-dev pkg-config
```

**Solution (Linux - Fedora/RHEL)**:

```bash
sudo dnf install openssl-devel pkg-config
```

---

### Issue: Build is very slow

**Symptom**: `cargo build --release` takes 10+ minutes

**Expected**: 2-5 minutes on modern hardware

**Causes & Solutions**:

**1. Link-time optimization (LTO)**:
- LTO is enabled in release builds ([`Cargo.toml:43`](Cargo.toml))
- This is intentional for performance
- Expect longer build times for optimized binary

**2. Limited CPU cores**:

```bash
# Use all available cores (default)
cargo build --release

# Or limit if running out of memory
cargo build --release -j 2
```

**3. No build caching**:

```bash
# First build is always slow (compiles all dependencies)
# Subsequent builds are incremental (much faster)

# To speed up rebuilds, avoid `cargo clean` unless necessary
```

**Workaround for development**:

```bash
# Use debug builds for development (30-60 seconds)
cargo build

# Only use release builds for final testing
cargo build --release
```

---

### Issue: "error: package collision" or duplicate dependencies

**Symptom**: Build fails with conflicting versions

**Solution**:

```bash
# Update Cargo.lock
cargo update

# Or clean and rebuild
cargo clean
cargo build --release
```

---

## Runtime Errors

### Issue: Application crashes on startup

**Symptom**: Window appears briefly, then crashes

**Debug Steps**:

```bash
# 1. Run from terminal to see error messages
./target/release/depmgr

# 2. Check console output for [ERROR] messages

# 3. Run debug build for better error messages
cargo run

# 4. Check for missing package managers
brew --version   # Should work if you have packages to manage
```

**Common Causes**:
- No package managers installed → Install at least one (Homebrew, npm, cargo, pip)
- Network issues → Homebrew API requires internet connection
- Permission issues → Check file system permissions

**Code Reference**: Startup code in [`src/main.rs:11-45`](src/main.rs)

---

### Issue: "No packages found"

**Symptom**: Application runs but shows empty table

**Cause 1**: No package managers detected

**Solution**:

```bash
# Verify at least one package manager is installed
brew --version   # Should output version number
npm --version
cargo --version
pip3 --version

# Install a package manager if none found
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Cause 2**: No packages installed in detected managers

**Solution**:

```bash
# Install some packages
brew install wget curl
npm install -g npm
pip3 install requests

# Refresh DepMgr
# Click "Refresh" button in sidebar
```

**Code Reference**: Detection logic in [`src/managers/detector.rs`](src/managers/detector.rs)

---

### Issue: Application hangs during scan

**Symptom**: "Scanning packages..." never completes

**Timeout Limits**:
- Homebrew API: 30 seconds
- Individual commands: 30 seconds per package manager
- Total scan: Should complete within 60 seconds

**Debug Steps**:

```bash
# 1. Test package manager commands manually
brew list --versions  # Should complete in 1-2 seconds
npm list -g --depth=0 --json  # Should complete in 1-2 seconds

# 2. Check network connectivity (Homebrew requires API access)
curl -I https://formulae.brew.sh/api/formula.json
# Should return: HTTP/2 200

# 3. Run DepMgr from terminal to see debug output
./target/release/depmgr
# Look for timeout messages: [ERROR] Failed to ...
```

**Workaround**:
- Wait for timeout (will skip failing manager)
- Fix network connection
- Update package manager: `brew update`

**Code Reference**: Scan orchestration in [`src/app.rs:41-179`](src/app.rs)

---

### Issue: Descriptions not loading

**Symptom**: Description column shows "-" for all packages

**Expected**: Descriptions fill in within 10-30 seconds

**Debug Steps**:

```bash
# 1. Check if Homebrew API is accessible
curl -I https://formulae.brew.sh/api/formula.json

# 2. Check if packages actually have descriptions
brew info wget  # Should show description

# 3. Check console for errors
# Look for: [ERROR] Failed to fetch descriptions
```

**Causes**:
- Network timeout
- API rate limiting (rare)
- Firewall blocking API access

**Solution**:
- Wait and click "Refresh" button
- Check firewall settings
- Descriptions will load from cache on next run

**Code Reference**: Description fetching in [`src/managers/homebrew_fast.rs:210-270`](src/managers/homebrew_fast.rs)

---

## Performance Issues

### Issue: First scan takes 5+ minutes

**Expected**: 30-60 seconds for first scan

**Debug Steps**:

```bash
# Time each component manually
time brew list --versions  # Should be <2s
time npm list -g --depth=0  # Should be <2s
time curl -I https://formulae.brew.sh/api/formula.json  # Should be <1s

# If any command is slow:
# - Check network speed
# - Check if package manager needs update
# - Check if disk is full
```

**Workaround**:
- First scan is always slower (no cache)
- Subsequent scans are <1 second (cached)
- Let it complete once, then enjoy fast loads

**Performance Details**: See [`PERFORMANCE.md`](PERFORMANCE.md) and [`docs/worklog.md:353-512`](docs/worklog.md)

---

### Issue: High memory usage

**Expected**: 50-200 MB typical, 500 MB max

**If using >1 GB**:
- Likely a bug or memory leak
- Report as issue with steps to reproduce

**Debug**:

```bash
# Monitor memory (macOS)
/usr/bin/time -l ./target/release/depmgr

# Check for memory leaks (requires Xcode)
leaks --atExit -- ./target/release/depmgr
```

---

### Issue: UI feels sluggish

**Symptom**: Slow rendering, lag when scrolling

**Causes**:
- Too many packages (1000+)
- Debug build (use release)
- Outdated GPU drivers

**Solutions**:

```bash
# 1. Use release build
cargo build --release
./target/release/depmgr

# 2. Close other GPU-intensive apps

# 3. Reduce number of packages displayed
# Use search filter to narrow down list
```

**UI Framework**: egui renders at ~60 FPS ([`src/main.rs:47-54`](src/main.rs))

---

## Package Manager Issues

### Issue: Homebrew packages not appearing

**Symptom**: Homebrew detected but no packages shown

**Debug**:

```bash
# Check if Homebrew is working
brew list --versions

# Expected output: List of installed packages
# If empty: No packages installed
# If error: Homebrew needs repair

# Repair Homebrew
brew doctor  # Shows issues
brew update  # Update Homebrew itself
```

**Code Reference**: Homebrew integration in [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)

---

### Issue: npm packages not appearing

**Symptom**: npm detected but no global packages shown

**Debug**:

```bash
# Check npm global packages
npm list -g --depth=0

# Check npm prefix (where globals are installed)
npm config get prefix

# If prefix is wrong, set it:
npm config set prefix ~/.npm-global
```

**Code Reference**: npm integration in [`src/managers/npm.rs`](src/managers/npm.rs)

---

### Issue: "Update" button doesn't work

**Symptom**: Clicking "Update" shows no progress or fails

**Debug**:

```bash
# Test update manually
brew upgrade wget  # Replace with your package

# If manual update works, check DepMgr console output
# Look for: [ERROR] Failed to update...
```

**Common Causes**:
- Permission issues (need sudo for some packages)
- Package manager not in PATH
- Dependency conflicts

**Solution**:
- Check error message in status area
- Run manual update to see detailed error
- Fix underlying package manager issue

**Code Reference**: Update methods in [`src/app.rs:276-336`](src/app.rs)

---

### Issue: "Remove" button shows dependency errors

**Symptom**: Cannot remove package due to dependencies

**Example Error**:
```
Error: Refusing to uninstall brotli
because it is required by other packages
```

**This is expected**: Homebrew protects dependencies

**Solution**:
- Remove dependent packages first
- Or use Homebrew CLI directly: `brew uninstall --ignore-dependencies brotli`

**DepMgr shows the error message** ([`src/ui/dashboard.rs:96-102`](src/ui/dashboard.rs) - error display)

---

## UI Problems

### Issue: Columns are too narrow

**Solution**: Drag column dividers to resize

- Hover over column header dividers
- Cursor changes to resize icon
- Click and drag to adjust width

**UI Implementation**: [`src/ui/dashboard.rs:134-145`](src/ui/dashboard.rs) - Resizable table

---

### Issue: Text is cut off

**Symptom**: Long descriptions or paths truncated

**Solution**:
1. Resize columns (drag dividers wider)
2. Enable horizontal scrolling (scroll bar at bottom)
3. Hover for tooltip (planned feature, not yet implemented)

---

### Issue: Window doesn't resize

**Symptom**: Cannot make window smaller than initial size

**Expected**: Minimum size is 800×600 ([`src/main.rs:16`](src/main.rs))

**Solution**: This is intentional to maintain readability

**Workaround**: If you need smaller, modify `min_inner_size` in [`src/main.rs`](src/main.rs) and recompile

---

### Issue: Search doesn't find packages

**Symptom**: Typing in search box shows no results

**Debug**:
- Search is case-insensitive substring match
- Only matches package names (not descriptions)
- Check if manager filters are excluding packages

**Solution**:
1. Clear search box
2. Enable all package managers (checkboxes in sidebar)
3. Disable "Outdated Only" filter
4. Try again

**Code Reference**: Filter logic in [`src/app.rs:192-228`](src/app.rs)

---

## Platform-Specific Issues

### macOS: "depmgr is damaged and can't be opened"

**Symptom**: Gatekeeper blocks application

**Cause**: Binary not code-signed

**Solution**:

```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine ./target/release/depmgr

# Or allow in System Preferences:
# System Preferences → Security & Privacy → Allow anyway
```

---

### macOS: Application appears then immediately closes

**Symptom**: Window flashes briefly, then disappears

**Debug**:

```bash
# Run from terminal to see crash log
./target/release/depmgr

# Check Console.app for crash reports
# Application: Console.app
# Search for: depmgr
```

**Common Causes**:
- Missing egui dependencies (shouldn't happen with static linking)
- macOS version too old (requires 10.15+)

---

### Linux: "error while loading shared libraries"

**Symptom**: Cannot run binary on different Linux system

**Cause**: Missing system libraries

**Solution**:

```bash
# Check missing libraries
ldd target/release/depmgr

# Install missing libraries (Ubuntu/Debian)
sudo apt install libgtk-3-0 libssl3

# Or build with musl for static linking
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

---

## Debug Mode

### Enable Verbose Logging

**Run from terminal** to see debug output:

```bash
# Normal output
./target/release/depmgr

# Redirect to file
./target/release/depmgr 2>&1 | tee depmgr.log

# Use debug build for more verbose output
cargo run
```

**Console Output Format**:
```
[DEBUG] Starting package scan...
[DEBUG] Found 4 package managers
[DEBUG] Scanning Homebrew packages...
[DEBUG] Found 83 Homebrew packages
[INFO] Successfully updated wget
[ERROR] Failed to list npm packages: ...
```

**Code Locations**:
- `println!` statements throughout codebase
- Look for `[DEBUG]`, `[INFO]`, `[ERROR]` prefixes

---

### Run with Debugger

**VS Code (with CodeLLDB extension)**:

1. Open project in VS Code
2. Set breakpoint in [`src/main.rs`](src/main.rs)
3. Press F5 to start debugging
4. Step through code to find issue

**LLDB (command line)**:

```bash
# Build with debug symbols
cargo build

# Run with lldb
lldb target/debug/depmgr

# Set breakpoint
(lldb) breakpoint set --name main
(lldb) run
```

---

### Check Dependencies

**List All Dependencies**:

```bash
cargo tree
```

**Check for Version Conflicts**:

```bash
cargo tree --duplicates
```

**Update Dependencies**:

```bash
cargo update
cargo build --release
```

---

## Getting Help

### Before Reporting Issues

**Gather Information**:

```bash
# System information
uname -a
sw_vers  # macOS only

# Rust version
rustc --version
cargo --version

# Package manager versions
brew --version
npm --version
cargo --version
pip3 --version

# DepMgr version
./target/release/depmgr --version  # (Not yet implemented)

# Console output
./target/release/depmgr 2>&1 | tee depmgr.log
# Attach depmgr.log when reporting
```

### Where to Get Help

1. **Documentation**:
   - [`README.md`](README.md) - Overview and quick start
   - [`SETUP.md`](SETUP.md) - Installation guide
   - [`ARCHITECTURE.md`](ARCHITECTURE.md) - Technical details
   - [`docs/worklog.md`](docs/worklog.md) - Development history

2. **GitHub Issues**:
   - Search existing issues
   - Create new issue with system info and logs
   - Include steps to reproduce

3. **Debug Yourself**:
   - Run from terminal
   - Check console output
   - Test package managers manually
   - Try debug build: `cargo run`

### Issue Report Template

```markdown
**System Information:**
- OS: macOS 14.0 / Ubuntu 22.04 / etc.
- Rust: 1.70.0
- DepMgr: v0.1.0 (or commit hash)

**Package Managers:**
- Homebrew: 4.0.0 (installed: yes)
- npm: 9.0.0 (installed: yes)
- cargo: 1.70.0 (installed: yes)
- pip: 22.0.0 (installed: yes)

**Problem Description:**
Clear description of the issue

**Steps to Reproduce:**
1. Launch DepMgr
2. Click "Refresh"
3. ...

**Expected Behavior:**
What should happen

**Actual Behavior:**
What actually happens

**Console Output:**
```
[Paste relevant console output here]
```

**Screenshots:**
[If applicable]
```

---

## Quick Reference

### Common Commands

```bash
# Build and run
cargo run

# Build release
cargo build --release

# Clean build
cargo clean && cargo build --release

# Update dependencies
cargo update

# Check for issues
cargo check
cargo clippy

# Format code
cargo fmt
```

### Diagnostic Commands

```bash
# Test package managers
brew --version
npm --version
cargo --version
pip3 --version

# List packages manually
brew list --versions
npm list -g --depth=0
cargo install --list
pip3 list --format=json

# Check network
curl -I https://formulae.brew.sh/api/formula.json

# Monitor resources
top -pid $(pgrep depmgr)
```

---

**Still having issues?** Consult [`docs/worklog.md`](docs/worklog.md) for detailed development history and known issues, or create a GitHub issue with system info and logs.

