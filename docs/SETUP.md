# Setup Guide

## Requirements

- **Rust 1.70+**: Get it from [rustup.rs](https://rustup.rs/)
- **macOS**: Only platform tested (might work on Linux, no idea about Windows)
- **At least one package manager**: Homebrew, npm, cargo, or pip

## Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustc --version  # verify
```

## Build and Run

```bash
# Clone
git clone <repo-url>
cd xyz

# Build release (optimized, takes 2-5 min first time)
cargo build --release

# Run
./target/release/depmgr
```

Or just `cargo run` for development builds (faster compile, slower runtime).

## First Run

When you launch:
1. Window appears (1200×800)
2. Detects package managers (~1 sec)
3. Scans packages in background (~30-60 sec first time)
4. Packages appear incrementally as found
5. Subsequent runs are instant (<1 sec from cache)

**Code flow**: [`src/main.rs`](../src/main.rs) → [`src/app.rs:41-179`](../src/app.rs) → manager modules

## What It Scans

**Package managers** (auto-detected):
- Homebrew: `brew --version`
- npm: `npm --version`
- cargo: `cargo --version`
- pip: `pip3 --version`

**Project directories** (for usage detection):
- `~/Desktop`
- `~/Documents`
- `~/projects`
- `~/dev`
- `~/Developer`
- `~/code`
- `~/workspace`

Scans up to 4 levels deep, looks for `package.json`, `Cargo.toml`, `requirements.txt`, etc.

**Code**: [`src/scanner/project_scanner.rs`](../src/scanner/project_scanner.rs)

## Configuration

There is none. Everything is auto-detected. If you want to customize scan directories, edit the code in [`src/scanner/mod.rs`](../src/scanner/mod.rs).

## Development Setup

```bash
# Install dev tools
rustup component add rustfmt clippy
cargo install cargo-watch

# IDE setup (optional)
# VS Code: Install rust-analyzer extension
# IntelliJ: Install Rust plugin
```

## Troubleshooting

### "cargo: command not found"

Rust not installed or not in PATH:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build fails with linker errors

macOS:
```bash
xcode-select --install
```

Linux:
```bash
sudo apt install build-essential libssl-dev pkg-config
```

### "No packages found"

No package managers installed. Install at least one:
```bash
# Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### App hangs during scan

Check if package managers work:
```bash
brew list --versions        # should be fast
npm list -g --depth=0       # should be fast
curl -I https://formulae.brew.sh/api/formula.json  # should work
```

### Slow first scan (5+ minutes)

Expected first time (no cache). Subsequent scans are <1 sec. If it's consistently slow:
- Check network (Homebrew uses API)
- Check if `brew update` helps
- Look for `[ERROR]` messages in console

## Build Configurations

**Development** (fast compile, slower runtime):
```bash
cargo build
./target/debug/depmgr
```

**Release** (optimized):
```bash
cargo build --release
./target/release/depmgr
```

Release config ([`Cargo.toml:42-45`](../Cargo.toml)):
```toml
[profile.release]
opt-level = 3          # max optimization
lto = true             # link-time optimization
codegen-units = 1      # single codegen unit
```

## Platform Notes

**macOS**: Works fine, tested on 10.15+

**Linux**: Might work, not tested. You'll need:
```bash
sudo apt install build-essential libssl-dev pkg-config
```

**Windows**: No idea, probably won't work without changes to path handling.

## Verification

After building, check:
```bash
# Binary exists
ls -lh target/release/depmgr  # ~15-20 MB

# Runs
./target/release/depmgr

# Package managers detected (check console output)
# Should see: [DEBUG] Found X package managers
```

## More Help

- **Troubleshooting**: [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md)
- **Architecture**: [`ARCHITECTURE.md`](ARCHITECTURE.md)
- **Development**: [`../CONTRIBUTING.md`](../CONTRIBUTING.md)
