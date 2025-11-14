# DepMgr - Unified Package Manager Dashboard

**A blazingly fast Rust GUI application that provides a unified visual dashboard for managing packages across all your package managers in one place.**

---

## 📋 Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Supported Package Managers](#supported-package-managers)
- [Quick Start](#quick-start)
- [Architecture Highlights](#architecture-highlights)
- [Screenshots & UI](#screenshots--ui)
- [Performance](#performance)
- [Documentation](#documentation)
- [Development](#development)
- [License](#license)

---

## Overview

**DepMgr** solves a common problem: developers use multiple package managers (Homebrew, npm, cargo, pip) but have no unified way to see what's installed, what's outdated, and what's taking up disk space. DepMgr provides:

- **Unified Dashboard**: See ALL packages from ALL managers in one table
- **One-Click Updates**: Update packages with buttons, not terminal commands
- **Project Mapping**: See which projects use which packages
- **Orphan Detection**: Find unused packages taking up space
- **Real-Time UI**: Incremental loading with immediate feedback

**For Product Managers**: This tool increases developer productivity by reducing context switching between multiple package management tools. Instead of running 10+ different CLI commands across different terminals, developers can manage all packages in one visual interface.

**For AI Engineers**: This is a showcase of Rust's capabilities for building high-performance desktop applications with modern async patterns, parallel processing, and immediate-mode GUI architecture.

---

## Key Features

### 🎯 Unified Package View

View packages from Homebrew, npm, cargo, and pip in a single searchable, filterable table.

**Implementation**: [`src/app.rs:40-179`](src/app.rs) - The `start_scan()` method orchestrates parallel scanning of all available package managers.

```rust
// Scans all available package managers in parallel
pub fn start_scan(&mut self) {
    // Homebrew scan (if available)
    if available_managers.contains(&PackageManager::Homebrew) {
        match crate::managers::homebrew_fast::list_homebrew_packages_fast().await
    }
    
    // npm, cargo, pip scans run concurrently
    // Results merge into single unified list
}
```

### ⚡ Blazingly Fast Performance

- **100-200x faster** than sequential CLI calls
- **First load**: 30-60 seconds (vs 8-10 minutes with old approach)
- **Cached loads**: <100ms (instant)
- **HTTP API + Parallel Processing**: Single API call fetches all 6,943 Homebrew formulas

**Implementation**: [`src/managers/homebrew_fast.rs:22-100`](src/managers/homebrew_fast.rs) - Uses Homebrew Formula API for batch fetching

```rust
// ONE API call replaces 83+ separate CLI invocations
let url = "https://formulae.brew.sh/api/formula.json";
let formulas: Vec<FormulaInfo> = client.get(url).send().await?.json().await?;

// Parallel filtering with Rayon
let packages: Vec<Package> = formulas
    .par_iter() // Parallel iterator
    .filter_map(|formula| installed.get(&formula.name))
    .collect();
```

### 🔍 Smart Project Detection

Automatically scans your project directories and maps packages to projects that use them.

**Implementation**: [`src/scanner/project_scanner.rs:6-226`](src/scanner/project_scanner.rs)

Detects project types by marker files:
- `package.json` → uses `node`, `npm`
- `Cargo.toml` → uses `rust`, `cargo`
- `requirements.txt`/`Pipfile` → uses `python`, `pip`
- `Gemfile` → uses `ruby`, `gem`

### 📦 Package Operations

- **Update**: Individual or bulk updates with progress tracking
- **Remove**: Uninstall packages with one-click reinstall safety net
- **Reinstall**: Accidentally removed something? Click "Reinstall" to restore

**Implementation**: [`src/app.rs:276-495`](src/app.rs) - Update, uninstall, and reinstall methods with async operation tracking

### 🎨 Modern GUI

Built with [egui](https://github.com/emilk/egui) (immediate-mode GUI framework):
- Resizable table columns
- Real-time search and filtering
- Color-coded status indicators
- Progress spinners for async operations
- Horizontal + vertical scrolling

**Implementation**: [`src/ui/dashboard.rs:5-292`](src/ui/dashboard.rs) - Complete dashboard UI with resizable table

---

## Supported Package Managers

| Manager | Status | List Command | Update Command |
|---------|--------|--------------|----------------|
| **Homebrew** | ✅ Fully Supported | API: `formulae.brew.sh/api/formula.json` | `brew upgrade <pkg>` |
| **npm** | ✅ Fully Supported | `npm list -g --depth=0 --json` | `npm update -g <pkg>` |
| **Cargo** | ✅ Fully Supported | `cargo install --list` | `cargo install --force <pkg>` |
| **pip** | ✅ Fully Supported | `pip3 list --format=json` | `pip3 install --upgrade <pkg>` |
| yarn | 🔄 Planned | - | - |
| pnpm | 🔄 Planned | - | - |
| gem | 🔄 Planned | - | - |

**Implementation Reference**:
- Package manager enum: [`src/models/package.rs:5-54`](src/models/package.rs)
- Detection logic: [`src/managers/detector.rs`](src/managers/detector.rs)
- Manager implementations: [`src/managers/`](src/managers/) directory

---

## Quick Start

### Prerequisites

- **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
- **macOS** (currently primary platform, Linux/Windows support planned)
- At least one package manager installed (Homebrew, npm, cargo, pip)

### Installation

**Option 1: Build from Source**

```bash
# Clone the repository
git clone <repository-url>
cd xyz

# Build release binary (optimized)
cargo build --release

# Binary location: target/release/depmgr
./target/release/depmgr
```

**Option 2: Development Build**

```bash
# Clone and run directly
git clone <repository-url>
cd xyz
cargo run
```

### First Run

1. **Launch the application**: `./target/release/depmgr` or `cargo run`
2. **Wait for scan**: The app automatically detects and scans all available package managers (~30-60 seconds first time)
3. **Explore packages**: Use search, filters, and sorting to find packages
4. **Take action**: Update, remove, or reinstall packages with one click

**What Happens on Startup**:
- [`src/main.rs:11-45`](src/main.rs) - Initializes GUI window and starts async scanning
- Package managers are auto-detected: [`src/managers/detector.rs`](src/managers/detector.rs)
- Initial scan runs in background: [`src/app.rs:41-179`](src/app.rs)

---

## Architecture Highlights

### Modular Design

```
src/
├── main.rs                 # Entry point, GUI initialization
├── app.rs                  # Application state and business logic
├── models/                 # Data models
│   ├── package.rs          # Package, PackageManager enums
│   ├── project.rs          # Project detection models
│   └── usage.rs            # Package usage tracking
├── managers/               # Package manager integrations
│   ├── homebrew_fast.rs    # Homebrew (HTTP API + parallel)
│   ├── npm.rs              # npm global packages
│   ├── cargo.rs            # Cargo installed binaries
│   ├── pip.rs              # pip/pip3 packages
│   └── detector.rs         # Auto-detect available managers
├── scanner/                # Project directory scanning
│   └── project_scanner.rs  # Detect tool usage in projects
├── ui/                     # GUI components
│   └── dashboard.rs        # Main dashboard table view
└── utils/                  # Shared utilities
    ├── cache.rs            # In-memory caching (DashMap)
    ├── http_client.rs      # HTTP client with connection pooling
    └── command.rs          # CLI command execution
```

### Technology Stack

**Core**:
- **Language**: Rust 2021 Edition ([`Cargo.toml:3`](Cargo.toml))
- **GUI Framework**: egui 0.33 + eframe ([`Cargo.toml:8-10`](Cargo.toml))
- **Async Runtime**: Tokio with full features ([`Cargo.toml:13`](Cargo.toml))

**Performance**:
- **HTTP Client**: reqwest 0.12 with connection pooling ([`Cargo.toml:17`](Cargo.toml))
- **Parallel Processing**: Rayon 1.10 ([`Cargo.toml:20`](Cargo.toml))
- **Caching**: DashMap 6.1 (lock-free concurrent hashmap) ([`Cargo.toml:23`](Cargo.toml))

**Data & Utilities**:
- **Serialization**: serde + serde_json + toml ([`Cargo.toml:30-32`](Cargo.toml))
- **File Operations**: walkdir, glob ([`Cargo.toml:35-36`](Cargo.toml))
- **Error Handling**: anyhow, thiserror ([`Cargo.toml:26-27`](Cargo.toml))

### Async Architecture

**Multi-Runtime Design**:
- GUI thread runs eframe's event loop (60 FPS)
- Tokio runtime handles all async I/O operations
- Background tasks spawn without blocking UI

**Implementation**: [`src/app.rs:14-36`](src/app.rs)

```rust
pub struct DepMgrApp {
    pub packages: Arc<RwLock<Vec<Package>>>,  // Shared state
    pub runtime: tokio::runtime::Runtime,      // Async runtime
    pub is_scanning: Arc<AtomicBool>,         // Lock-free flag
    // ...
}
```

### Caching Strategy

**Multi-Level Cache**:
1. **Memory Cache**: DashMap with 1-hour TTL
2. **HTTP Cache**: Connection pooling reduces API latency by 60-80%

**Implementation**: [`src/utils/cache.rs`](src/utils/cache.rs) and [`src/utils/http_client.rs:5-13`](src/utils/http_client.rs)

---

## Screenshots & UI

### Main Dashboard

The main view shows all packages in a unified table with:
- **Columns**: Name, Manager, Installed Version, Latest Version, Description, Usage, Status, Actions
- **Filters**: Search by name, filter by manager, show only outdated
- **Stats Sidebar**: Total packages, outdated count, unused packages
- **Action Buttons**: Update, Remove, Reinstall for each package

**UI Implementation**: [`src/ui/dashboard.rs:5-292`](src/ui/dashboard.rs)

### Table Features

- **Resizable Columns**: Drag column dividers to adjust width ([`src/ui/dashboard.rs:134-145`](src/ui/dashboard.rs))
- **Striped Rows**: Alternating colors for readability ([`src/ui/dashboard.rs:135`](src/ui/dashboard.rs))
- **Color Coding**: 
  - 🟢 Green: Up-to-date packages
  - 🟠 Orange: Outdated packages  
  - 🔴 Red: Unused packages

### Real-Time Updates

- **Incremental Loading**: Packages appear as soon as they're found
- **Progress Indicators**: Spinners show active operations
- **Status Messages**: Color-coded feedback (green for success, red for errors)

**Implementation**: [`src/ui/dashboard.rs:71-119`](src/ui/dashboard.rs) - Scanning status and update feedback

---

## Performance

### Benchmarks

| Operation | Old Approach | New Approach | Improvement |
|-----------|--------------|--------------|-------------|
| List Homebrew packages | 5-7 minutes (83 × `brew info`) | 1-3 seconds (API call) | **100-200x** |
| Get descriptions | 3-5 minutes | <10 seconds | **18-30x** |
| Check outdated | 30-60 seconds | Instant (from API) | **∞** |
| **Total first load** | **8-10 minutes** | **30-60 seconds** | **10-20x** |
| Cached load | N/A | <100ms | Instant |

**Source**: [`docs/worklog.md:353-469`](docs/worklog.md) - Detailed performance transformation notes

### Optimization Techniques

1. **HTTP API over Process Spawning**: 100-500x faster than CLI
2. **Batch Requests**: 1 API call replaces 83+ CLI invocations
3. **Parallel Processing**: Rayon parallelizes CPU-bound work across all cores
4. **Connection Pooling**: Reuse HTTP connections for 60-80% latency reduction
5. **Adaptive Concurrency**: 8 concurrent description fetches (no bottlenecks)
6. **Memory Caching**: 1-hour TTL avoids redundant API calls

**Implementation References**:
- Fast Homebrew implementation: [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)
- HTTP client setup: [`src/utils/http_client.rs`](src/utils/http_client.rs)
- Cache layer: [`src/utils/cache.rs`](src/utils/cache.rs)
- Parallel filtering: [`src/managers/homebrew_fast.rs:69-87`](src/managers/homebrew_fast.rs)

### Release Optimizations

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
```

**Source**: [`Cargo.toml:42-45`](Cargo.toml)

---

## Documentation

**Root Documentation**:
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow and contribution guidelines

**Technical Documentation** (`/docs`):
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - System design, component interactions, data flow
- **[SETUP.md](docs/SETUP.md)** - Detailed installation, configuration, and troubleshooting
- **[API.md](docs/API.md)** - Internal API documentation and module reference
- **[DEPLOYMENT.md](docs/DEPLOYMENT.md)** - Build instructions, distribution, CI/CD
- **[TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Common issues and solutions
- **[PERFORMANCE.md](docs/PERFORMANCE.md)** - Detailed performance analysis and benchmarks
- **[worklog.md](docs/worklog.md)** - Complete development history and decisions

---

## Development

### Building from Source

```bash
# Development build (faster compilation, slower runtime)
cargo build

# Release build (optimized, slower compilation)
cargo build --release

# Run directly
cargo run

# Run with release optimizations
cargo run --release
```

### Code Quality

```bash
# Type checking
cargo check

# Linting (auto-fix many issues)
cargo clippy --fix

# Formatting
cargo fmt

# Run all checks
cargo check && cargo clippy && cargo fmt
```

### Project Structure

See **[ARCHITECTURE.md](ARCHITECTURE.md)** for detailed component documentation.

### Adding New Package Managers

1. Create new file in `src/managers/` (e.g., `yarn.rs`)
2. Implement standard functions:
   - `list_<manager>_packages() -> Result<Vec<Package>>`
   - `check_outdated_<manager>(&mut [Package]) -> Result<()>`
   - `update_<manager>_package(name: String) -> Result<()>`
3. Add to `PackageManager` enum in [`src/models/package.rs:5-18`](src/models/package.rs)
4. Update detector in [`src/managers/detector.rs`](src/managers/detector.rs)
5. Add to scan flow in [`src/app.rs:41-179`](src/app.rs)

**Example**: See existing implementations in [`src/managers/npm.rs`](src/managers/npm.rs), [`src/managers/cargo.rs`](src/managers/cargo.rs), or [`src/managers/pip.rs`](src/managers/pip.rs)

---

## Roadmap

### Current (v0.1.0)
- ✅ Homebrew, npm, cargo, pip support
- ✅ Fast HTTP API + parallel processing
- ✅ Project usage scanning
- ✅ Update/remove/reinstall operations
- ✅ Resizable table UI

### Next (v0.2.0)
- 🔄 yarn, pnpm, gem support
- 🔄 Disk cache layer (persist between restarts)
- 🔄 Background refresh daemon
- 🔄 Export package lists (JSON, CSV)
- 🔄 Package dependency graph visualization

### Future (v1.0.0)
- 🔮 Linux and Windows support
- 🔮 CLI mode (headless operation)
- 🔮 Package search and install from UI
- 🔮 Homebrew cask support (GUI apps)
- 🔮 Custom package manager plugins

---

## License

[Add license information here]

---

## Acknowledgments

- **egui**: Excellent immediate-mode GUI framework
- **Homebrew API**: Public formula API enabling fast batch queries
- **Tokio**: Robust async runtime
- **Rayon**: Dead-simple data parallelism

---

## Support

For issues, questions, or contributions:
- See **[CONTRIBUTING.md](CONTRIBUTING.md)** for development guidelines
- See **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** for common issues
- Check **[docs/worklog.md](docs/worklog.md)** for development history

---

**Built with ❤️ and Rust 🦀**

