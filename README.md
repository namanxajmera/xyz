# DepMgr

A fast Rust GUI app that shows all your packages from Homebrew, npm, cargo, and pip in one place. Update or remove them with actual buttons instead of remembering CLI commands.

## What It Does

I got tired of running `brew list`, `npm list -g`, `cargo install --list`, and `pip3 list` separately and having no idea what was outdated or taking up space. So I built this.

- **One table, all packages**: See everything from all your package managers
- **Actually fast**: 30-60 seconds to load everything (used to take 8-10 minutes)
- **Click to update**: Buttons instead of typing commands
- **See what's used**: Shows which projects actually use each package
- **Find orphans**: See what's installed but not used anywhere

## Performance

The key breakthrough was switching from spawning 83 `brew info` commands (5-7 minutes) to one HTTP call to Homebrew's API (1-3 seconds). Then using Rayon to filter 6,943 formulas in parallel. Full details in [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md).

**Before vs After**:
- List Homebrew packages: 5-7 min → 1-3 sec (100-200x faster)
- Total first load: 8-10 min → 30-60 sec (10-20x faster)
- Cached loads: instant (<100ms)

## Install & Run

**Prerequisites**: 
- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- macOS (only platform I've tested)
- At least one package manager (Homebrew, npm, cargo, pip)

**Build**:
```bash
git clone <this-repo>
cd xyz
cargo build --release
./target/release/depmgr
```

Or just `cargo run` for development builds.

## What You'll See

The UI is a table with these columns:
- **Name**: Package name
- **Manager**: Which manager it's from (Homebrew, npm, etc.)
- **Installed**: Version you have
- **Latest**: Latest available version (if outdated)
- **Description**: What the package does
- **Usage**: Which projects use it (or "Unused")
- **Status**: Current/Outdated (color coded)
- **Action**: Update/Remove/Reinstall buttons

Plus a sidebar with:
- Checkboxes to filter by package manager
- Stats (total, outdated, unused counts)
- Search box
- "Refresh" and "Update All" buttons

## Supported Package Managers

| Manager | Status |
|---------|--------|
| Homebrew | ✅ Works |
| npm | ✅ Works |
| Cargo | ✅ Works |
| pip | ✅ Works |

That's it. I'll add yarn/pnpm/gem if I ever need them.

## How It Works

**Tech stack**:
- Rust (fast, no GC pauses)
- egui/eframe (immediate-mode GUI)
- Tokio (async runtime)
- Rayon (parallel processing)
- reqwest (HTTP client with connection pooling)

**Architecture** (simplified):
```
GUI Thread (60 FPS)
    ↓
App State (Arc<RwLock<Vec<Package>>>)
    ↑
Tokio Runtime (async tasks)
    ↓
Package Managers (parallel scanning)
```

Full details in [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

**Key optimizations**:
1. HTTP API instead of CLI spawning (100-500x faster)
2. Parallel processing with Rayon (uses all CPU cores)
3. Connection pooling (60-80% latency reduction)
4. Memory caching (1-hour TTL)

## Project Structure

```
src/
├── main.rs                 # Entry point
├── app.rs                  # App state & logic
├── models/                 # Data structures
│   ├── package.rs          # Package model
│   ├── project.rs          # Project detection
│   └── usage.rs            # Usage tracking
├── managers/               # Package manager integrations
│   ├── homebrew_fast.rs    # Homebrew (HTTP API)
│   ├── npm.rs              # npm
│   ├── cargo.rs            # Cargo
│   ├── pip.rs              # pip
│   └── detector.rs         # Auto-detect managers
├── scanner/                # Project scanning
│   └── project_scanner.rs  # Find tool usage
├── ui/                     # GUI
│   └── dashboard.rs        # Main table UI
└── utils/                  # Helpers
    ├── cache.rs            # Memory cache
    ├── http_client.rs      # HTTP client
    └── command.rs          # CLI execution
```

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System design
- [`docs/API.md`](docs/API.md) - API reference
- [`docs/SETUP.md`](docs/SETUP.md) - Installation
- [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) - Performance details
- [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) - Common issues
- [`docs/worklog.md`](docs/worklog.md) - Development history
- [`CONTRIBUTING.md`](CONTRIBUTING.md) - Contributing

## Development

```bash
# Build and run
cargo run

# Check for errors
cargo check

# Lint and auto-fix
cargo clippy --fix

# Format code
cargo fmt
```

To add a new package manager, see the pattern in `src/managers/npm.rs` or `src/managers/cargo.rs`. Basically:
1. Create `src/managers/yourmanager.rs`
2. Implement list/update/check functions
3. Add to detector
4. Add to app scan flow

Full guide in [`CONTRIBUTING.md`](CONTRIBUTING.md).

## License

MIT (do whatever you want with it)

## Notes

This is a personal project. No formal roadmap, no screenshots (just run it), no corporate BS. The code is well-documented if you want to understand how it works or add features.

If something breaks, check [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md) or look at the console output (I log everything with `[DEBUG]`/`[ERROR]` prefixes).
