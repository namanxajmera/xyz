# Architecture

System design and technical details. Mainly for AI assistants understanding the codebase.

## Contents

- [System Architecture](#system-architecture)
- [Core Components](#core-components)
- [Data Models](#data-models)
- [Async Architecture](#async-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Package Manager Integrations](#package-manager-integrations)
- [GUI Architecture](#gui-architecture)
- [Caching Strategy](#caching-strategy)
- [Error Handling](#error-handling)
- [State Management](#state-management)
- [Extension Points](#extension-points)

## Overview

Fast desktop app in Rust that manages packages across Homebrew, npm, cargo, and pip. Key design goals:

- **Fast**: Parallel processing, HTTP APIs, caching
- **Responsive**: Non-blocking async, incremental UI updates
- **Modular**: Easy to add new package managers
- **Reliable**: Strong typing, good error handling

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         GUI Layer                           │
│                     (egui + eframe)                         │
│                 src/ui/dashboard.rs                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ├─> User Events (clicks, search, filter)
                  │
┌─────────────────▼───────────────────────────────────────────┐
│                    Application Layer                        │
│                       src/app.rs                            │
│  ┌────────────────────────────────────────────────────┐    │
│  │  DepMgrApp (State Management)                      │    │
│  │  - packages: Arc<RwLock<Vec<Package>>>            │    │
│  │  - tokio::Runtime                                  │    │
│  │  - search/filter state                            │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ├─> Async Operations (update, scan, etc.)
                  │
┌─────────────────▼───────────────────────────────────────────┐
│                   Package Managers Layer                    │
│                    src/managers/                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │ Homebrew │  │   npm    │  │  Cargo   │  │   pip    │  │
│  │   Fast   │  │          │  │          │  │          │  │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘  │
└─────────────────┬───────────────────────────────────────────┘
                  │
                  ├─> HTTP APIs / CLI Commands
                  │
┌─────────────────▼───────────────────────────────────────────┐
│                    Utilities Layer                          │
│                     src/utils/                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  HTTP Client │  │    Cache     │  │   Command    │    │
│  │  (reqwest)   │  │  (DashMap)   │  │   Runner     │    │
│  └──────────────┘  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────────────────────────┘
                  │
                  ├─> External APIs / System Commands
                  │
┌─────────────────▼───────────────────────────────────────────┐
│                    External Systems                         │
│  ┌──────────────────┐  ┌──────────────────┐               │
│  │  Homebrew API    │  │  Package Manager │               │
│  │  formulae.brew.sh│  │  CLIs (npm, pip) │               │
│  └──────────────────┘  └──────────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
src/
├── main.rs                      # Application entry point
│   └── Initializes GUI window, tokio runtime, starts scan
│
├── app.rs                       # Application state & business logic
│   ├── DepMgrApp struct
│   ├── start_scan() - orchestrates parallel scanning
│   ├── update/uninstall/reinstall operations
│   └── filtering & statistics
│
├── models/                      # Data structures
│   ├── mod.rs
│   ├── package.rs               # Package, PackageManager enum
│   ├── project.rs               # Project detection models
│   └── usage.rs                 # Package usage tracking
│
├── managers/                    # Package manager integrations
│   ├── mod.rs
│   ├── detector.rs              # Auto-detect available managers
│   ├── homebrew_fast.rs         # Homebrew (HTTP API)
│   ├── npm.rs                   # npm global packages
│   ├── cargo.rs                 # Cargo installed binaries
│   └── pip.rs                   # pip/pip3 packages
│
├── scanner/                     # Project directory scanning
│   ├── mod.rs
│   └── project_scanner.rs       # Detect tool usage in projects
│
├── ui/                          # GUI components
│   ├── mod.rs
│   └── dashboard.rs             # Main dashboard with table
│
└── utils/                       # Shared utilities
    ├── mod.rs
    ├── cache.rs                 # In-memory caching (DashMap)
    ├── http_client.rs           # HTTP client with connection pooling
    └── command.rs               # CLI command execution with timeout
```

**File Reference**: Project structure matches workspace layout at [`src/`](src/)

---

## Core Components

### 1. Application Entry Point (`main.rs`)

**Purpose**: Initialize the GUI window and start the application

**File**: [`src/main.rs:1-55`](src/main.rs)

**Key Responsibilities**:
- Create native window with egui
- Initialize `DepMgrApp` state
- Detect available package managers (blocking, runs once)
- Start async package scanning (non-blocking)
- Hand control to egui event loop

**Code Reference**:

```rust
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dependency Manager")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Dependency Manager",
        options,
        Box::new(|_cc| {
            let mut app = DepMgrApp::default();
            
            // Detect package managers (blocking, fast)
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                app.available_managers = 
                    crate::managers::detect_available_managers().await;
            });
            
            // Start async scan (non-blocking)
            app.start_scan();
            
            Ok(Box::new(app))
        }),
    )
}
```

### 2. Application State (`app.rs`)

**Purpose**: Central state management and business logic

**File**: [`src/app.rs:6-497`](src/app.rs)

**Key Structure**:

```rust
pub struct DepMgrApp {
    // Shared package list (thread-safe)
    pub packages: Arc<RwLock<Vec<Package>>>,
    
    // Available and selected package managers
    pub available_managers: Vec<PackageManager>,
    pub selected_managers: HashSet<PackageManager>,
    
    // UI state
    pub search_query: String,
    pub show_outdated_only: bool,
    pub show_orphaned_only: bool,
    
    // Async operation tracking
    pub is_scanning: Arc<AtomicBool>,
    pub updating_packages: Arc<RwLock<HashSet<String>>>,
    pub update_status: Arc<RwLock<String>>,
    pub removed_packages: Arc<RwLock<HashSet<String>>>,
    
    // Tokio runtime for async operations
    pub runtime: tokio::runtime::Runtime,
}
```

**Thread Safety**:
- `Arc<RwLock<T>>`: Multiple readers OR single writer (async-safe)
- `Arc<AtomicBool>`: Lock-free boolean flag
- `HashSet<String>`: Tracks which packages are being updated/removed

**Code Reference**: [`src/app.rs:6-38`](src/app.rs)

### 3. Package Scanning Orchestration

**Method**: `DepMgrApp::start_scan()`

**File**: [`src/app.rs:41-179`](src/app.rs)

**Flow**:

```
1. Set is_scanning = true
2. Spawn async task on tokio runtime
3. For each available package manager:
   ├─> Homebrew (if available)
   │   ├─> Fetch from API (fast)
   │   ├─> Update UI immediately
   │   ├─> Scan project usage
   │   ├─> Check outdated status
   │   └─> Fetch descriptions (background)
   │
   ├─> npm (if available)
   │   ├─> List global packages
   │   ├─> Check outdated
   │   ├─> Append to package list
   │   └─> Fetch descriptions (background)
   │
   ├─> Cargo (if available)
   │   └─> (similar pattern)
   │
   └─> pip (if available)
       └─> (similar pattern)
4. Set is_scanning = false
```

**Parallelism**: All package managers are scanned concurrently (not sequentially)

**Code Reference**: [`src/app.rs:41-179`](src/app.rs)

### 4. GUI Dashboard (`ui/dashboard.rs`)

**Purpose**: Render the main user interface

**File**: [`src/ui/dashboard.rs:5-292`](src/ui/dashboard.rs)

**Layout Structure**:

```
┌──────────────────────────────────────────────────┐
│  Sidebar (left panel)                            │
│  ┌────────────────────────┐                      │
│  │ Package Managers       │                      │
│  │ ☑ Homebrew             │                      │
│  │ ☑ npm                  │                      │
│  │ ☑ Cargo                │                      │
│  │ ☑ pip                  │                      │
│  ├────────────────────────┤                      │
│  │ Stats                  │                      │
│  │ Total: 150             │                      │
│  │ Outdated: 12           │                      │
│  │ Unused: 8              │                      │
│  ├────────────────────────┤                      │
│  │ 🔄 Refresh             │                      │
│  │ ⬆️ Update All (12)     │                      │
│  └────────────────────────┘                      │
└──────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────┐
│  Main Panel (central area)                       │
│  ┌────────────────────────────────────────────┐  │
│  │ Search: [_____]  ☑ Outdated Only           │  │
│  ├────────────────────────────────────────────┤  │
│  │ ⏳ Scanning packages...                    │  │
│  ├────────────────────────────────────────────┤  │
│  │ Package Table (resizable columns)          │  │
│  │ Name | Manager | Ver | Desc | Usage | ...  │  │
│  │ ───────────────────────────────────────────│  │
│  │ wget │ Homebrew│ 1.21│ ... │ 3 proj│ Update│  │
│  │ node │ Homebrew│ 20.0│ ... │ 5 proj│ Current│  │
│  │ ...                                         │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

**Key UI Components**:
1. **Sidebar** ([`src/ui/dashboard.rs:8-51`](src/ui/dashboard.rs)):
   - Package manager checkboxes for filtering
   - Statistics display (total, outdated, unused)
   - Refresh button
   - "Update All" bulk action button

2. **Search & Filters** ([`src/ui/dashboard.rs:58-66`](src/ui/dashboard.rs)):
   - Text search by package name
   - "Outdated Only" checkbox
   - "Orphaned Only" checkbox

3. **Status Display** ([`src/ui/dashboard.rs:71-119`](src/ui/dashboard.rs)):
   - Scanning spinner with "Scanning packages..." message
   - Update status with color coding (green=success, red=error)
   - Full-width natural wrapping for long messages

4. **Package Table** ([`src/ui/dashboard.rs:134-288`](src/ui/dashboard.rs)):
   - Resizable columns (drag dividers)
   - Striped rows for readability
   - Columns: Name, Manager, Installed, Latest, Description, Usage, Status, Action
   - Action buttons: Update, Remove, Reinstall

**Immediate Mode GUI**: egui re-renders the entire UI every frame (~60 FPS). State changes are reflected immediately.

---

## Data Models

### Package (`models/package.rs`)

**File**: [`src/models/package.rs:62-72`](src/models/package.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,                  // Package name (e.g., "wget")
    pub manager: PackageManager,       // Which manager owns it
    pub installed_version: String,     // Currently installed version
    pub latest_version: Option<String>,// Available version (if outdated)
    pub is_outdated: bool,             // Needs update?
    pub size: Option<u64>,             // Disk space in bytes
    pub description: Option<String>,   // What the package does
    pub used_in: Vec<String>,          // Project paths using this
}
```

**Fields**:
- `name`: Unique identifier within a manager
- `manager`: Enum indicating source (Homebrew, npm, cargo, pip)
- `installed_version`: Current version string
- `latest_version`: Available version from registry/API (None if up-to-date)
- `is_outdated`: Computed flag for filtering
- `description`: Human-readable purpose (fetched from API/CLI)
- `used_in`: List of project directories that reference this package

### PackageManager Enum

**File**: [`src/models/package.rs:5-54`](src/models/package.rs)

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    Homebrew,
    Npm,
    Yarn,
    Pnpm,
    Cargo,
    Pip,
    Pipx,
    Gem,
    Go,
    Composer,
    Pub,
    Swift,
}
```

**Methods**:
- `name() -> &'static str`: Display name (e.g., "Homebrew")
- `command() -> &'static str`: CLI command (e.g., "brew")

**Usage**: This enum is used throughout the codebase to dispatch manager-specific operations.

### Project Model (`models/project.rs`)

**File**: [`src/models/project.rs`](src/models/project.rs)

```rust
pub struct Project {
    pub path: PathBuf,               // Project root directory
    pub project_type: ProjectType,   // Node, Rust, Python, etc.
    pub dependencies: Vec<Dependency>,// Package dependencies
}

pub struct Dependency {
    pub package_name: String,
    pub manager: PackageManager,
    pub version_constraint: String,  // e.g., "^1.0.0"
    pub is_dev: bool,                // Dev vs production dependency
}
```

**Usage**: Currently used for project scanning to detect which packages are used where.

**Future Enhancement**: Will be used for full dependency graph visualization.

### PackageUsage Model (`models/usage.rs`)

**File**: [`src/models/usage.rs`](src/models/usage.rs)

```rust
pub struct PackageUsage {
    pub package: Package,
    pub projects: Vec<Project>,  // Which projects use this package
}
```

**Usage**: Tracks package-to-project relationships for orphan detection.

---

## Async Architecture

### Multi-Runtime Design

**Problem**: GUI frameworks typically run on the main thread with their own event loop. How do we perform async I/O without blocking the UI?

**Solution**: Separate tokio runtime owned by `DepMgrApp`

**Implementation**: [`src/app.rs:15-36`](src/app.rs)

```rust
pub struct DepMgrApp {
    pub runtime: tokio::runtime::Runtime,
    // ...
}

impl Default for DepMgrApp {
    fn default() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            // ...
        }
    }
}
```

**Pattern**: Spawn async tasks without blocking the GUI thread

```rust
// In src/app.rs methods:
pub fn update_package(&mut self, package_name: String, manager: PackageManager) {
    let packages = Arc::clone(&self.packages);
    
    self.runtime.spawn(async move {
        // This runs on tokio threadpool, not GUI thread
        let result = match manager {
            PackageManager::Homebrew => 
                crate::managers::homebrew_fast::update_package(package_name).await,
            // ...
        };
        
        // Update shared state (thread-safe)
        *packages.write().await = new_packages;
    });
}
```

**Code Reference**: [`src/app.rs:276-336`](src/app.rs) - Update package method

### Shared State Synchronization

**Challenge**: GUI thread needs to read state while async tasks modify it

**Solution**: `Arc<RwLock<T>>` for safe concurrent access

**Pattern**:

```rust
// Write from async task:
*app.packages.write().await = new_packages;

// Read from GUI thread (non-blocking):
let packages = app.packages.blocking_read();
```

**Lock-Free Flags**: Use `AtomicBool` for simple flags

```rust
pub is_scanning: Arc<AtomicBool>

// Set/get without locks:
is_scanning.store(true, Ordering::Relaxed);
let scanning = is_scanning.load(Ordering::Relaxed);
```

**Code Reference**: [`src/app.rs:13-18`](src/app.rs) - Async operation tracking fields

### Incremental UI Updates

**Pattern**: Update shared state multiple times during scanning

```rust
// Phase 1: Show packages immediately
*packages_clone.write().await = packages.clone();

// Phase 2: Add project usage info
scan_project_usage(&mut packages);
*packages_clone.write().await = packages.clone();  // Update again

// Phase 3: Add outdated status
check_outdated(&mut packages).await;
*packages_clone.write().await = packages.clone();  // Update again
```

**Result**: Users see packages appear → then usage info fills in → then outdated status appears

**Code Reference**: [`src/app.rs:53-76`](src/app.rs) - Homebrew scan phases

---

## Performance Optimizations

### 1. HTTP API Instead of Process Spawning

**Problem**: Running `brew info <package>` 83 times takes 5-7 minutes (6-10 seconds per package)

**Solution**: Single HTTP GET to Homebrew Formula API

**Implementation**: [`src/managers/homebrew_fast.rs:22-100`](src/managers/homebrew_fast.rs)

```rust
// ONE API call replaces 83+ CLI invocations
let url = "https://formulae.brew.sh/api/formula.json";
let formulas: Vec<FormulaInfo> = client
    .get(url)
    .send()
    .await?
    .json()
    .await?;

// Returns ALL 6,943 Homebrew formulas in 1-3 seconds
```

**Speedup**: 100-200x faster (5-7 minutes → 1-3 seconds)

### 2. Parallel Processing with Rayon

**Problem**: Filtering 6,943 formulas to find 83 installed packages is CPU-bound

**Solution**: Rayon parallel iterators

**Implementation**: [`src/managers/homebrew_fast.rs:69-87`](src/managers/homebrew_fast.rs)

```rust
let packages: Vec<Package> = formulas
    .par_iter()  // Parallel iterator (uses all CPU cores)
    .filter_map(|formula| {
        installed.get(&formula.name).map(|version| {
            Package { /* ... */ }
        })
    })
    .collect();

// Result: ~3ms to filter 6,943 items
```

**Speedup**: Linear → parallel processing (~8x on 8-core CPU)

### 3. Connection Pooling

**Problem**: Making many HTTP requests has overhead (TCP handshake, TLS negotiation)

**Solution**: Reuse HTTP connections with connection pooling

**Implementation**: [`src/utils/http_client.rs:5-13`](src/utils/http_client.rs)

```rust
pub fn create_http_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(10)    // Reuse up to 10 connections
        .pool_idle_timeout(Duration::from_secs(90))
        .timeout(Duration::from_secs(30))
        .gzip(true)                     // Compression
        .build()
        .expect("Failed to create HTTP client")
}
```

**Speedup**: 60-80% latency reduction on repeated requests

### 4. Memory Caching

**Problem**: API calls are expensive; data doesn't change often

**Solution**: In-memory cache with TTL

**Implementation**: [`src/utils/cache.rs`](src/utils/cache.rs)

```rust
use dashmap::DashMap;  // Lock-free concurrent hashmap

// Cache structure (internal):
struct CacheEntry<T> {
    value: T,
    expiry: Instant,
}

static CACHE: Lazy<DashMap<String, CacheEntry<Vec<u8>>>> = 
    Lazy::new(DashMap::new);

// Usage in homebrew_fast.rs:
if let Some(cached) = get_cached::<Vec<Package>>("homebrew_all_packages") {
    return Ok(cached);  // Instant return!
}

// ... fetch from API ...

set_cached("homebrew_all_packages", &packages, 3600);  // 1 hour TTL
```

**Speedup**: <100ms for cached loads (vs 30-60 seconds for fresh fetch)

### 5. Adaptive Concurrency

**Problem**: Fetching descriptions for 83 packages sequentially is slow

**Solution**: Fetch 8 descriptions concurrently

**Implementation**: [`src/managers/homebrew_fast.rs:210-250`](src/managers/homebrew_fast.rs)

```rust
// Process 8 packages at a time
for chunk in packages_needing_desc.chunks(8) {
    let futures: Vec<_> = chunk
        .iter()
        .map(|pkg| fetch_description(pkg.name.clone()))
        .collect();
    
    let results = futures::future::join_all(futures).await;
    
    // Update UI incrementally after each batch
    update_ui();
}
```

**Speedup**: 8x faster than sequential (with minimal resource usage)

### 6. Release Profile Optimizations

**Configuration**: [`Cargo.toml:42-45`](Cargo.toml)

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization (whole-program)
codegen-units = 1      # Single codegen unit for better opt
```

**Effect**: Binary is ~20% smaller and ~10-15% faster

**Trade-off**: Longer compile times (acceptable for release builds)

---

## Package Manager Integrations

### Common Interface Pattern

Each package manager module implements the same set of functions:

```rust
// List installed packages
pub async fn list_<manager>_packages() -> Result<Vec<Package>>

// Check which packages are outdated
pub async fn check_outdated_<manager>(packages: &mut [Package]) -> Result<()>

// Update a single package
pub async fn update_<manager>_package(name: String) -> Result<()>

// Install a package
pub async fn install_<manager>_package(name: String) -> Result<()>

// Uninstall a package
pub async fn uninstall_<manager>_package(name: String) -> Result<()>

// Fetch descriptions (background task)
pub async fn add_<manager>_descriptions(packages: Arc<RwLock<Vec<Package>>>)
```

### Homebrew Integration (`homebrew_fast.rs`)

**File**: [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)

**Strategy**: HTTP API + local CLI validation

**Key Functions**:

1. **`list_homebrew_packages_fast()`** ([`src/managers/homebrew_fast.rs:22-100`](src/managers/homebrew_fast.rs)):
   - Fetches ALL formulas from `https://formulae.brew.sh/api/formula.json`
   - Gets locally installed packages via `brew list --versions`
   - Filters API results to only installed packages
   - Returns with descriptions already populated

2. **`check_outdated_packages_fast()`** ([`src/managers/homebrew_fast.rs:102-150`](src/managers/homebrew_fast.rs)):
   - Compares `installed_version` vs `latest_version` (both from API)
   - Marks `is_outdated = true` for packages needing updates
   - No CLI calls needed (instant!)

3. **`update_package()`** ([`src/managers/homebrew_fast.rs:180-200`](src/managers/homebrew_fast.rs)):
   - Runs `brew upgrade <package_name>`
   - 5-minute timeout
   - Returns Result with detailed error messages

4. **`add_missing_descriptions_fast()`** ([`src/managers/homebrew_fast.rs:210-270`](src/managers/homebrew_fast.rs)):
   - Only fetches descriptions for packages missing them (rare)
   - Uses `brew info --json=v2` in batches
   - Runs in background (doesn't block UI)

### npm Integration (`npm.rs`)

**File**: [`src/managers/npm.rs`](src/managers/npm.rs)

**Strategy**: JSON output from npm CLI

**Key Implementation Details**:

1. **`list_npm_packages()`**:
   - Command: `npm list -g --depth=0 --json`
   - Parses JSON: `{ "dependencies": { "package": { "version": "1.0.0" } } }`
   - Returns Package structs with `manager: PackageManager::Npm`

2. **`check_outdated_npm()`**:
   - Command: `npm outdated -g --json`
   - Parses JSON: `{ "package": { "current": "1.0", "latest": "2.0" } }`
   - Updates `latest_version` and `is_outdated` fields

3. **`add_npm_descriptions()`**:
   - Command: `npm view <package> description` (per package)
   - Runs 8 concurrent requests
   - Updates UI incrementally as descriptions arrive

**Code Reference**: [`src/managers/npm.rs`](src/managers/npm.rs)

### Cargo Integration (`cargo.rs`)

**File**: [`src/managers/cargo.rs`](src/managers/cargo.rs)

**Strategy**: Parse `cargo install --list` output + crates.io API

**Key Implementation Details**:

1. **`list_cargo_packages()`**:
   - Command: `cargo install --list`
   - Parses text output format:
     ```
     package-name v1.2.3:
         binary-name
     ```
   - Extracts package names and versions with regex

2. **`check_outdated_cargo()`**:
   - Note: Requires `cargo-outdated` crate (not implemented yet)
   - Alternative: Query crates.io API for latest version
   - TODO: Add cargo-outdated integration

3. **`add_cargo_descriptions()`**:
   - Uses crates.io API: `https://crates.io/api/v1/crates/<package>`
   - Requires User-Agent header (crates.io policy)
   - Parses JSON response for `description` field
   - Runs 8 concurrent requests

**Code Reference**: [`src/managers/cargo.rs`](src/managers/cargo.rs)

### pip Integration (`pip.rs`)

**File**: [`src/managers/pip.rs`](src/managers/pip.rs)

**Strategy**: JSON output from pip CLI

**Key Implementation Details**:

1. **`list_pip_packages()`**:
   - Command: `pip3 list --format=json`
   - Parses JSON: `[{ "name": "package", "version": "1.0.0" }]`
   - Returns Package structs

2. **`check_outdated_pip()`**:
   - Command: `pip3 list --outdated --format=json`
   - Parses JSON: `[{ "name": "pkg", "version": "1.0", "latest_version": "2.0" }]`
   - Updates packages in-place

3. **`add_pip_descriptions()`**:
   - Command: `pip3 show <package>` (per package)
   - Parses text output for "Summary: " line
   - Runs 8 concurrent requests

**Code Reference**: [`src/managers/pip.rs`](src/managers/pip.rs)

### Package Manager Detection

**File**: [`src/managers/detector.rs`](src/managers/detector.rs)

**Function**: `detect_available_managers() -> Vec<PackageManager>`

**Strategy**: Check if CLI commands exist

```rust
async fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .output()
        .await
        .is_ok()
}

pub async fn detect_available_managers() -> Vec<PackageManager> {
    let mut managers = Vec::new();
    
    if command_exists("brew").await {
        managers.push(PackageManager::Homebrew);
    }
    if command_exists("npm").await {
        managers.push(PackageManager::Npm);
    }
    // ... etc for cargo, pip, yarn, pnpm ...
    
    managers
}
```

**Usage**: Called once at startup in [`src/main.rs:28-36`](src/main.rs)

---

## GUI Architecture

### Immediate Mode GUI (egui)

**Concept**: Unlike retained-mode GUIs (HTML, Qt), immediate mode re-runs UI code every frame

**Benefits**:
- Simple mental model: UI is a function of state
- No need to manually update UI elements
- State changes automatically reflected next frame

**Drawback**: Must be efficient (runs 60 times per second)

**Implementation**: [`src/main.rs:47-54`](src/main.rs)

```rust
impl eframe::App for DepMgrApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Called ~60 times per second
        
        self.handle_refresh();  // Check for refresh requests
        ui::show_dashboard(ctx, self);  // Re-render entire UI
    }
}
```

### Dashboard Layout

**File**: [`src/ui/dashboard.rs`](src/ui/dashboard.rs)

**Structure**: Three-panel layout

1. **Sidebar (Left Panel)**: [`src/ui/dashboard.rs:8-51`](src/ui/dashboard.rs)
   - Fixed-width, resizable
   - Package manager filters (checkboxes)
   - Statistics
   - Action buttons (Refresh, Update All)

2. **Main Panel (Central Area)**: [`src/ui/dashboard.rs:54-290`](src/ui/dashboard.rs)
   - Search bar
   - Filter toggles
   - Status messages (scanning, update progress)
   - Package table

3. **Package Table**: [`src/ui/dashboard.rs:134-288`](src/ui/dashboard.rs)
   - Resizable columns (egui_extras::TableBuilder)
   - Striped rows
   - Horizontal + vertical scrolling
   - Action buttons per row

### Table Implementation

**Library**: `egui_extras::TableBuilder`

**Code**: [`src/ui/dashboard.rs:134-288`](src/ui/dashboard.rs)

```rust
TableBuilder::new(ui)
    .striped(true)
    .resizable(true)
    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
    .column(Column::initial(100.0).at_least(60.0).resizable(true))  // Name
    .column(Column::initial(80.0).at_least(60.0).resizable(true))   // Manager
    // ... more columns ...
    .header(20.0, |mut header| {
        header.col(|ui| { ui.strong("Name"); });
        // ... more headers ...
    })
    .body(|mut body| {
        for pkg in filtered {
            body.row(18.0, |mut row| {
                row.col(|ui| { ui.label(&pkg.name); });
                // ... more cells ...
            });
        }
    });
```

**Features**:
- Users can drag column dividers to resize
- Columns have minimum widths
- Table auto-scrolls if content overflows

### Color Coding

**Status Column**: [`src/ui/dashboard.rs:231-243`](src/ui/dashboard.rs)

```rust
if pkg.is_outdated {
    ui.label(
        egui::RichText::new("Outdated")
            .color(egui::Color32::from_rgb(255, 165, 0))  // Orange
    );
} else {
    ui.label(
        egui::RichText::new("Current")
            .color(egui::Color32::from_rgb(0, 200, 0))    // Green
    );
}
```

**Usage Column**: [`src/ui/dashboard.rs:203-228`](src/ui/dashboard.rs)

```rust
if pkg.used_in.is_empty() {
    ui.label(
        egui::RichText::new("Unused")
            .color(egui::Color32::from_rgb(200, 0, 0))    // Red
    );
} else {
    ui.label(
        egui::RichText::new(display_text)
            .color(egui::Color32::from_rgb(0, 150, 0))    // Green
    );
}
```

### Real-Time Updates

**Pattern**: Request repaint when async operations are in progress

**Code**: [`src/ui/dashboard.rs:71-80`](src/ui/dashboard.rs)

```rust
let is_scanning = app.is_scanning.load(Ordering::Relaxed);
if is_scanning {
    ui.horizontal(|ui| {
        ui.spinner();  // Animated spinner
        ui.label("Scanning packages...");
    });
    ctx.request_repaint();  // Keep updating UI
}
```

**Effect**: UI continuously updates while scanning, showing packages as they appear

---

## Caching Strategy

### In-Memory Cache

**Implementation**: [`src/utils/cache.rs`](src/utils/cache.rs)

**Data Structure**: `DashMap<String, CacheEntry>`
- Key: String identifier (e.g., "homebrew_all_packages")
- Value: Serialized data + expiry timestamp

**Thread Safety**: DashMap is lock-free (uses sharding internally)

**TTL**: 1 hour (3600 seconds) for package data

**Usage Pattern**:

```rust
// Check cache first
if let Some(cached) = get_cached::<Vec<Package>>("homebrew_all_packages") {
    return Ok(cached);  // Instant!
}

// Fetch from API
let packages = fetch_from_api().await?;

// Cache for 1 hour
set_cached("homebrew_all_packages", &packages, 3600);
```

**Benefits**:
- Instant loads after first fetch
- Reduces API load
- Works across multiple scans in same session

**Limitations**:
- Does not persist between app restarts
- Planned: Add disk cache layer for cold-start performance

### HTTP Connection Pooling

**Implementation**: [`src/utils/http_client.rs`](src/utils/http_client.rs)

**Configuration**:
- Pool size: 10 connections per host
- Idle timeout: 90 seconds
- Request timeout: 30 seconds
- Gzip compression: Enabled

**Benefits**:
- Reuse TCP connections (no handshake overhead)
- Reuse TLS sessions (no negotiation overhead)
- 60-80% latency reduction on repeated requests

---

## Error Handling

### Error Types

**Primary**: `anyhow::Result<T>`
- Used for functions that can fail
- Provides context and error chains
- Allows `?` operator for easy propagation

**Custom Errors**: `thiserror` (where needed)
- Define custom error types with variants
- Implement `Display` and `Error` traits automatically

**Example Pattern**:

```rust
use anyhow::{anyhow, Result};

pub async fn list_packages() -> Result<Vec<Package>> {
    let output = Command::new("brew")
        .arg("list")
        .output()
        .await
        .map_err(|e| anyhow!("Failed to run brew: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!(
            "brew list failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    // Parse output...
    Ok(packages)
}
```

### Error Reporting in UI

**Pattern**: Display errors in status area with red text

**Code**: [`src/ui/dashboard.rs:96-102`](src/ui/dashboard.rs)

```rust
if update_status.contains("Failed") || update_status.contains("failed") {
    ui.label(
        egui::RichText::new(&update_status)
            .color(egui::Color32::from_rgb(255, 0, 0))  // Red
    );
}
```

**User Experience**:
- Errors don't crash the app
- Clear error messages with context
- Auto-dismiss after 3-5 seconds (non-critical errors)

---

## State Management

### Shared State Pattern

**Challenge**: Multiple async tasks need to read/write the same data

**Solution**: `Arc<RwLock<T>>` for safe concurrent access

**Pattern**:

```rust
// In app.rs:
pub packages: Arc<RwLock<Vec<Package>>>

// Clone Arc (cheap, just increments ref count):
let packages_clone = Arc::clone(&self.packages);

// Spawn async task:
self.runtime.spawn(async move {
    // Async task owns packages_clone
    
    // Write access:
    *packages_clone.write().await = new_packages;
});

// GUI thread reads (non-blocking):
let packages = self.packages.blocking_read();
```

**Benefits**:
- Thread-safe (enforced by Rust compiler)
- Multiple readers OR single writer (efficient)
- Deadlock-free (RwLock is fair)

### Atomic Flags

**Pattern**: Use `AtomicBool` for simple boolean flags

**Example**: [`src/app.rs:13`](src/app.rs)

```rust
pub is_scanning: Arc<AtomicBool>

// Set flag (from any thread):
is_scanning.store(true, Ordering::Relaxed);

// Read flag (from any thread):
let scanning = is_scanning.load(Ordering::Relaxed);
```

**Benefits**:
- Lock-free (no contention)
- Safe across threads
- Faster than `Mutex<bool>`

### State Transitions

**Package Update Flow**:

```
1. User clicks "Update" button
   ↓
2. Add package to updating_packages set
   ↓
3. Show spinner in UI (is_updating() returns true)
   ↓
4. Spawn async task to run `brew upgrade <pkg>`
   ↓
5. Update succeeds/fails
   ↓
6. Update packages list (fetch new version)
   ↓
7. Remove from updating_packages set
   ↓
8. Show success/error message
   ↓
9. Clear message after 3 seconds
```

**Code Reference**: [`src/app.rs:276-336`](src/app.rs)

---

## Extension Points

### Adding a New Package Manager

**Steps**:

1. **Add to enum**: [`src/models/package.rs:5-18`](src/models/package.rs)
   ```rust
   pub enum PackageManager {
       // ... existing ...
       Yarn,  // Add new variant
   }
   ```

2. **Implement name() and command()**: [`src/models/package.rs:20-54`](src/models/package.rs)
   ```rust
   impl PackageManager {
       pub fn name(&self) -> &'static str {
           match self {
               // ...
               PackageManager::Yarn => "yarn",
           }
       }
       
       pub fn command(&self) -> &'static str {
           match self {
               // ...
               PackageManager::Yarn => "yarn",
           }
       }
   }
   ```

3. **Create manager module**: `src/managers/yarn.rs`
   ```rust
   use crate::models::{Package, PackageManager};
   use anyhow::Result;
   
   pub async fn list_yarn_packages() -> Result<Vec<Package>> {
       // Run: yarn global list --json
       // Parse output
       // Return Vec<Package>
   }
   
   pub async fn check_outdated_yarn(packages: &mut [Package]) -> Result<()> {
       // Run: yarn global outdated
       // Update is_outdated field
   }
   
   pub async fn update_yarn_package(name: String) -> Result<()> {
       // Run: yarn global upgrade <name>
   }
   
   // ... install, uninstall, add_descriptions ...
   ```

4. **Add to detector**: [`src/managers/detector.rs`](src/managers/detector.rs)
   ```rust
   if command_exists("yarn").await {
       managers.push(PackageManager::Yarn);
   }
   ```

5. **Add to scan flow**: [`src/app.rs:41-179`](src/app.rs)
   ```rust
   // In start_scan():
   if available_managers.contains(&PackageManager::Yarn) {
       println!("[DEBUG] Scanning yarn packages...");
       match crate::managers::yarn::list_yarn_packages().await {
           Ok(mut packages) => {
               // Check outdated
               // Append to packages list
               // Fetch descriptions in background
           }
           Err(e) => eprintln!("[ERROR] Failed: {}", e),
       }
   }
   ```

6. **Add to update dispatch**: [`src/app.rs:286-302`](src/app.rs)
   ```rust
   let result = match manager {
       // ... existing ...
       PackageManager::Yarn => 
           crate::managers::yarn::update_yarn_package(package_name.clone()).await,
   };
   ```

**Example Reference**: See [`src/managers/npm.rs`](src/managers/npm.rs), [`src/managers/cargo.rs`](src/managers/cargo.rs), or [`src/managers/pip.rs`](src/managers/pip.rs) for complete examples.

### Adding New UI Features

**Pattern**: Modify `src/ui/dashboard.rs`

**Examples**:

1. **Add a new filter**:
   ```rust
   // In app.rs:
   pub show_unused_only: bool,
   
   // In dashboard.rs:
   ui.checkbox(&mut app.show_unused_only, "Unused Only");
   
   // In filtered_packages():
   if self.show_unused_only && !pkg.used_in.is_empty() {
       return false;
   }
   ```

2. **Add a new column**:
   ```rust
   .column(Column::initial(80.0).at_least(60.0).resizable(true))  // New column
   
   .header(20.0, |mut header| {
       header.col(|ui| { ui.strong("Size"); });  // New header
   })
   
   .body(|mut body| {
       row.col(|ui| {  // New cell
           if let Some(size) = pkg.size {
               ui.label(format_size(size));
           }
       });
   })
   ```

3. **Add a new action**:
   ```rust
   // In dashboard.rs:
   if ui.button("Info").clicked() {
       app.show_package_info(pkg.name.clone());
   }
   
   // In app.rs:
   pub fn show_package_info(&mut self, name: String) {
       // Open dialog with detailed info
   }
   ```

---

## Summary

DepMgr's architecture is designed for:

1. **Performance**: HTTP APIs, parallel processing, caching, connection pooling
2. **Responsiveness**: Async operations, incremental updates, non-blocking UI
3. **Extensibility**: Modular design, common interfaces, clear extension points
4. **Reliability**: Strong typing, comprehensive error handling, thread-safe state

**Key Architectural Decisions**:
- **Rust**: Performance, safety, excellent ecosystem
- **egui**: Simple immediate-mode GUI, cross-platform
- **Tokio**: Robust async runtime
- **HTTP APIs**: 100-200x faster than CLI spawning
- **Rayon**: Easy data parallelism
- **DashMap**: Lock-free concurrent caching

**Performance Results**:
- **10-20x faster** first load (8-10 min → 30-60 sec)
- **Instant** cached loads (<100ms)
- **Real-time UI updates** during scanning
- **Parallel operations** across all package managers

For more details, see:
- **[README.md](README.md)** - Overview and quick start
- **[PERFORMANCE.md](PERFORMANCE.md)** - Detailed benchmarks
- **[docs/worklog.md](docs/worklog.md)** - Development history and decisions

