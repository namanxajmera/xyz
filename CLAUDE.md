# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**DepMgr** is a high-performance Rust GUI application that provides a unified visual dashboard for managing packages across multiple package managers (Homebrew, npm, Cargo, pip). The project demonstrates advanced Rust patterns with async/await, parallel processing, immediate-mode GUI (egui), and intelligent caching, achieving 100-200x performance improvements over traditional CLI approaches.

**Key Achievement**: First load reduced from 8-10 minutes to 30-60 seconds through HTTP API optimization and parallel processing.

## Development Commands

### Build and Run
```bash
# Development build (faster compilation)
cargo build
cargo run

# Release build (optimized, slower compilation)
cargo build --release
./target/release/depmgr

# Type checking (fast)
cargo check

# Watch mode (auto-reload on changes)
cargo install cargo-watch  # Install once
cargo watch -x run
```

### Code Quality
```bash
# Linting
cargo clippy
cargo clippy --fix  # Auto-fix issues

# Formatting
cargo fmt
cargo fmt -- --check  # Check without modifying

# All checks before committing
cargo check && cargo clippy && cargo fmt --check
```

### Testing
```bash
# Run tests (when available)
cargo test

# Manual testing checklist:
# - Application launches
# - All package managers detected
# - Packages load within 60 seconds
# - Search/filters work
# - Update/remove/reinstall operations work
```

## Architecture

### Project Structure

```
src/
├── main.rs                 # Entry point, GUI initialization (45 lines)
├── app.rs                  # Application state & async orchestration (495 lines)
├── models/                 # Data models
│   ├── package.rs          # Package, PackageManager enums
│   ├── project.rs          # Project detection models
│   └── usage.rs            # Package usage tracking
├── managers/               # Package manager integrations
│   ├── detector.rs         # Auto-detect available managers
│   ├── homebrew_fast.rs    # Homebrew (HTTP API approach)
│   ├── npm.rs              # npm global packages
│   ├── cargo.rs            # Cargo installed binaries
│   └── pip.rs              # pip/pip3 packages
├── scanner/                # Project directory scanning
│   └── project_scanner.rs  # Tool usage detection (226 lines)
├── ui/                     # GUI components
│   └── dashboard.rs        # Main table view (292 lines)
└── utils/                  # Shared utilities
    ├── cache.rs            # In-memory caching (DashMap)
    ├── http_client.rs      # HTTP client with connection pooling
    └── command.rs          # CLI command execution with timeout
```

### Technology Stack

**Core**:
- Rust 2021 Edition
- egui 0.33 (immediate-mode GUI framework)
- eframe 0.33 (native window manager)
- Tokio (async runtime with full features)

**Performance**:
- reqwest 0.12 (HTTP client with connection pooling, gzip)
- Rayon 1.10 (data parallelism across CPU cores)
- DashMap 6.1 (lock-free concurrent hashmap for caching)

**Data & Utilities**:
- serde + serde_json + toml (serialization)
- anyhow + thiserror (error handling)
- walkdir + glob (file operations)
- chrono (date/time handling)

### Key Architectural Patterns

**1. Async Multi-Threading Model**:
- **GUI thread**: Runs eframe's 60 FPS event loop (never blocks)
- **Tokio runtime**: Spawns async tasks for all I/O operations
- **Synchronization**: `Arc<RwLock<Vec<Package>>>` for shared state, `Arc<AtomicBool>` for lock-free flags
- **Pattern**: GUI reads with `blocking_read()`, background tasks write with `.write().await`

**2. Phased Loading Strategy** (see `src/app.rs:start_scan()`):
- Phase 1: Instant UI (0-1s) - Show empty table
- Phase 2: Basic list (1-3s) - Load packages from API/CLI
- Phase 3: Project usage (3-10s) - Scan directories, map packages to projects
- Phase 4: Outdated status (instant for Homebrew, 5-15s for others)
- Phase 5: Descriptions (background, 10-60s) - Progressive enhancement

**3. Performance Optimization Breakthrough** (`src/managers/homebrew_fast.rs`):
- **OLD approach**: 5-7 minutes (83+ separate `brew info` calls)
- **NEW approach**: 1-3 seconds (single API call to `formulae.brew.sh/api/formula.json`)
- **Speedup**: 100-200x faster
- **Key insight**: HTTP API over CLI process spawning + parallel filtering with Rayon

**4. State Management**:
```rust
pub struct DepMgrApp {
    pub packages: Arc<RwLock<Vec<Package>>>,  // Shared package data
    pub runtime: tokio::runtime::Runtime,      // Async runtime
    pub is_scanning: Arc<AtomicBool>,         // Lock-free scanning flag
    pub updating_packages: Arc<RwLock<HashSet<String>>>,  // Track in-progress updates
    // ... UI state
}
```

## Key Dependencies

**External APIs**:
- Homebrew Formula API: `https://formulae.brew.sh/api/formula.json` (6,943 formulas)
- npm registry: `https://registry.npmjs.org/<package-name>` (description fetching)
- crates.io API: `https://crates.io/api/v1/crates/<name>` (Cargo metadata)
- PyPI API: `https://pypi.org/pypi/<package-name>/json` (pip metadata)

**CLI Dependencies** (must be installed for package manager support):
- `brew` (Homebrew)
- `npm` (Node.js)
- `cargo` (Rust)
- `pip3` (Python)

## Development Workflow

### Adding a New Package Manager

Follow this systematic approach (see `CONTRIBUTING.md` for full details):

1. **Update enum** in `src/models/package.rs`:
   ```rust
   pub enum PackageManager {
       // ... existing ...
       Yarn,  // Add variant
   }
   ```

2. **Implement methods** in same file:
   ```rust
   impl PackageManager {
       pub fn name(&self) -> &'static str {
           match self {
               PackageManager::Yarn => "yarn",
               // ...
           }
       }
   }
   ```

3. **Create manager module** `src/managers/yarn.rs`:
   ```rust
   pub async fn list_yarn_packages() -> Result<Vec<Package>>
   pub async fn check_outdated_yarn(packages: &mut [Package]) -> Result<()>
   pub async fn update_yarn_package(name: String) -> Result<()>
   pub async fn install_yarn_package(name: String) -> Result<()>
   pub async fn uninstall_yarn_package(name: String) -> Result<()>
   pub async fn add_yarn_descriptions(packages: Arc<RwLock<Vec<Package>>>)
   ```

4. **Add to exports** in `src/managers/mod.rs`:
   ```rust
   pub mod yarn;
   ```

5. **Update detector** in `src/managers/detector.rs`:
   ```rust
   if command_exists("yarn").await {
       managers.push(PackageManager::Yarn);
   }
   ```

6. **Add to scan flow** in `src/app.rs:start_scan()`:
   ```rust
   if available_managers.contains(&PackageManager::Yarn) {
       match crate::managers::yarn::list_yarn_packages().await {
           Ok(mut packages) => {
               let _ = crate::managers::yarn::check_outdated_yarn(&mut packages).await;
               let mut all_packages = packages_clone.write().await;
               all_packages.extend(packages);
           }
           Err(e) => eprintln!("[ERROR] Failed to list yarn packages: {}", e),
       }
   }
   ```

7. **Add to update dispatch** in `src/app.rs:update_package()`:
   ```rust
   let result = match manager {
       PackageManager::Yarn =>
           crate::managers::yarn::update_yarn_package(package_name.clone()).await,
       // ...
   };
   ```

**Reference implementations**: See `src/managers/npm.rs`, `src/managers/cargo.rs`, or `src/managers/pip.rs`

### Code Style Conventions

**Error Handling**:
```rust
use anyhow::{anyhow, Result};

pub async fn list_packages() -> Result<Vec<Package>> {
    let output = run_command("npm", &["list"]).await
        .map_err(|e| anyhow!("Failed to run npm list: {}", e))?;
    Ok(packages)
}
```

**Logging**:
```rust
println!("[DEBUG] Processing {} packages", count);
println!("[INFO] Successfully updated {}", name);
println!("[ERROR] Failed to fetch: {}", error);
```

**Comments** (explain WHY, not WHAT):
```rust
// Good: Explains reasoning
// Use parallel processing because API returns 6k+ formulas
let packages: Vec<Package> = formulas.par_iter().filter_map(...).collect();

// Bad: States the obvious
// Loop through formulas
for formula in formulas { }
```

**Async Patterns**:
```rust
// Spawn background tasks without blocking GUI
self.runtime.spawn(async move {
    let result = some_async_fn().await;
    *arc_locked_data.write().await = result;
});
```

## Platform Requirements

**Primary Platform**: macOS (Darwin)
- Fully tested and optimized for macOS
- Linux support planned (v1.0.0)
- Windows support planned (v1.0.0)

**System Requirements**:
- Rust 1.70+ (install via rustup.rs)
- At least one package manager (Homebrew, npm, cargo, pip)
- macOS 10.15+ recommended

**Build Profiles** (in `Cargo.toml`):
```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit for better optimization
```

## Performance Considerations

### Critical Performance Patterns

1. **Prefer HTTP API over CLI** (see `src/managers/homebrew_fast.rs:22-100`):
   - 100-500x faster for batch operations
   - Single API call replaces dozens of process spawns

2. **Use Rayon for CPU-bound parallelism** (see `src/managers/homebrew_fast.rs:69-87`):
   ```rust
   let packages: Vec<Package> = formulas
       .par_iter()  // Parallel iterator
       .filter_map(|formula| installed.get(&formula.name))
       .collect();
   ```

3. **Connection pooling** (see `src/utils/http_client.rs`):
   - Reuse HTTP connections (60-80% latency reduction)
   - 10 idle connections per host, 90s idle timeout

4. **Memory caching** (see `src/utils/cache.rs`):
   - DashMap with 1-hour TTL
   - Avoids redundant API calls within session

5. **Adaptive concurrency** (see `src/managers/homebrew_fast.rs:242-291`):
   - 8 concurrent description fetches
   - Batched API requests to avoid rate limiting

### Performance Benchmarks

| Operation | Old | New | Speedup |
|-----------|-----|-----|---------|
| List Homebrew | 5-7 min | 1-3 sec | 100-200x |
| Get descriptions | 3-5 min | <10 sec | 18-30x |
| Check outdated | 30-60 sec | Instant | ∞ |
| **Total first load** | **8-10 min** | **30-60 sec** | **10-20x** |
| Cached load | N/A | <100ms | Instant |

See `docs/PERFORMANCE.md` for detailed analysis.

## Worklog Management

**CRITICAL:** Maintain `docs/worklog.md` as a running AI development diary.

**Rules**:
- Always append new entries to the TOP of the file
- Never delete or update older entries - only add new ones
- Use UTC timestamps: `date -u +"%Y-%m-%d %H:%M:%S UTC"`
- Log all significant activities: progress, decisions, issues, context, business logic

**Entry Format**:
```
## YYYY-MM-DD HH:MM:SS UTC

**Project**: xyz
**Activity**: Brief description
**What**: What was accomplished
**Details**: Technical details, decisions made, issues encountered
**Next**: What needs follow-up (if applicable)

---
```

**When to Log**:
- Starting/completing development sessions
- Making architectural decisions
- Encountering blocking issues
- Implementing new features or major changes
- Debugging complex problems
- Performance optimizations
- Schema changes or migrations
- Progress updates, thoughts, context, business logic
- Issues, solutions, workarounds, or interesting findings

This allows future Claude instances to understand what has been done, what was learned, and what needs attention.

## Project-Specific Guidelines

### Understanding the Code Flow

For new Claude instances, understand the codebase in this order:

1. **Start**: `src/main.rs` - See GUI initialization and startup flow
2. **Core logic**: `src/app.rs` - Application state and async orchestration
3. **Performance showcase**: `src/managers/homebrew_fast.rs` - HTTP API optimization approach
4. **UI patterns**: `src/ui/dashboard.rs` - egui immediate-mode GUI patterns
5. **History**: `docs/worklog.md` - Development decisions and evolution

### Common Operations

**Reading packages in GUI code**:
```rust
let packages = self.packages.blocking_read();  // Safe blocking read in GUI thread
```

**Spawning background tasks**:
```rust
let packages_clone = Arc::clone(&self.packages);
self.runtime.spawn(async move {
    let result = fetch_data().await;
    *packages_clone.write().await = result;
});
```

**Lock-free flags**:
```rust
self.is_scanning.store(true, Ordering::Relaxed);
if self.is_scanning.load(Ordering::Relaxed) { }
```

### Data Flow Example

Startup sequence (`src/main.rs` → `src/app.rs`):
```
1. main.rs: Initialize eframe window
2. DepMgrApp::default(): Create application state
3. Temporary Tokio runtime: detect_available_managers()
4. start_scan(): Spawn async package scanning tasks
5. eframe event loop: GUI updates at 60 FPS
6. Background tasks: Update shared state via Arc<RwLock>
```

Update flow (`src/app.rs:update_package()`):
```
1. UI: Update button clicked
2. Mark package as updating
3. Spawn Tokio task
4. Execute package manager command (brew upgrade, npm update, etc.)
5. Update status message
6. Refresh package list
7. Clear updating flag
8. Clear status after 3 seconds
```

### Known Limitations

- No disk persistence (cache lost on restart)
- macOS focused (Linux/Windows support planned)
- Only 4 of 12 planned package managers implemented
- No dependency graph visualization
- Cargo outdated checking not implemented (needs `cargo-outdated` tool)

### Future Roadmap

**v0.2.0**: yarn/pnpm/gem support, disk cache, background refresh daemon
**v1.0.0**: Linux/Windows support, CLI mode, package search from UI, Homebrew cask support

See `README.md` for complete roadmap.

## Documentation

**Root Directory**:
- `README.md` - Project overview and quick start
- `CONTRIBUTING.md` - Development guidelines and contribution process

**Technical Documentation** (`/docs`):
- `ARCHITECTURE.md` - System design, component interactions, data flow
- `SETUP.md` - Detailed installation and configuration
- `API.md` - Internal API documentation and module reference
- `DEPLOYMENT.md` - Build instructions and distribution
- `TROUBLESHOOTING.md` - Common issues and solutions
- `PERFORMANCE.md` - Detailed performance analysis
- `worklog.md` - Complete development history (this is the AI development diary)

## Quick Reference

**Important file locations**:
- Package manager implementations: `src/managers/*.rs`
- Data models: `src/models/*.rs`
- GUI code: `src/ui/dashboard.rs`
- Utilities: `src/utils/*.rs`
- Entry point: `src/main.rs`
- Main app logic: `src/app.rs`

**Code references in comments**: Use `file_path:line_number` format (e.g., `src/app.rs:41-179`)

**Line counts** (approximate):
- Total Rust code: ~2,500 lines
- Largest files: `app.rs` (495), `dashboard.rs` (292), `homebrew_fast.rs` (~300), `project_scanner.rs` (226)
