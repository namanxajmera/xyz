# API Documentation

**Internal API reference for DepMgr modules and functions**

---

## Table of Contents

- [Overview](#overview)
- [Module Structure](#module-structure)
- [Core APIs](#core-apis)
- [Package Manager APIs](#package-manager-apis)
- [Scanner APIs](#scanner-apis)
- [Utility APIs](#utility-apis)
- [Data Models](#data-models)
- [Type Reference](#type-reference)

---

## Overview

This document provides detailed API documentation for DepMgr's internal modules. This is primarily useful for:

- **AI Engineers**: Understanding code structure for contributions
- **Extension Developers**: Adding new package managers or features
- **Maintainers**: Reference for internal APIs and contracts

**Note**: This is not a public API (no crates.io publishing yet). All APIs are internal to the application.

---

## Module Structure

```
src/
├── main.rs              # Application entry point
├── app.rs               # Application state and business logic
├── models/              # Data structures
│   ├── mod.rs
│   ├── package.rs
│   ├── project.rs
│   └── usage.rs
├── managers/            # Package manager integrations
│   ├── mod.rs
│   ├── detector.rs
│   ├── homebrew_fast.rs
│   ├── npm.rs
│   ├── cargo.rs
│   └── pip.rs
├── scanner/             # Project directory scanning
│   ├── mod.rs
│   └── project_scanner.rs
├── ui/                  # GUI components
│   ├── mod.rs
│   └── dashboard.rs
└── utils/               # Shared utilities
    ├── mod.rs
    ├── cache.rs
    ├── http_client.rs
    └── command.rs
```

---

## Core APIs

### Application State (`app.rs`)

**File**: [`src/app.rs`](src/app.rs)

#### `DepMgrApp`

Main application state struct.

```rust
pub struct DepMgrApp {
    pub packages: Arc<RwLock<Vec<Package>>>,
    pub available_managers: Vec<PackageManager>,
    pub selected_managers: HashSet<PackageManager>,
    pub search_query: String,
    pub show_outdated_only: bool,
    pub show_orphaned_only: bool,
    pub is_scanning: Arc<AtomicBool>,
    pub refresh_requested: bool,
    pub runtime: tokio::runtime::Runtime,
    pub updating_packages: Arc<RwLock<HashSet<String>>>,
    pub update_status: Arc<RwLock<String>>,
    pub removed_packages: Arc<RwLock<HashSet<String>>>,
}
```

**Code Reference**: [`src/app.rs:6-19`](src/app.rs)

**Fields**:
- `packages`: Thread-safe shared list of all packages
- `available_managers`: Detected package managers on system
- `selected_managers`: User-selected managers for filtering
- `search_query`: Current search text
- `show_outdated_only`: Filter flag for outdated packages
- `show_orphaned_only`: Filter flag for orphaned packages
- `is_scanning`: Atomic flag indicating scan in progress
- `refresh_requested`: Flag for pending refresh
- `runtime`: Tokio async runtime
- `updating_packages`: Set of packages currently being updated
- `update_status`: Current update status message
- `removed_packages`: Packages removed in current session

#### Methods

##### `start_scan(&mut self)`

Initiates parallel scanning of all available package managers.

```rust
pub fn start_scan(&mut self)
```

**Behavior**:
1. Sets `is_scanning` to true
2. Spawns async task on tokio runtime
3. Scans each available manager in parallel
4. Updates `packages` incrementally as data arrives
5. Sets `is_scanning` to false when complete

**Code Reference**: [`src/app.rs:41-179`](src/app.rs)

**Example Usage**:
```rust
let mut app = DepMgrApp::default();
app.start_scan();  // Non-blocking
```

##### `filtered_packages(&self) -> Vec<Package>`

Returns filtered list of packages based on current filters.

```rust
pub fn filtered_packages(&self) -> Vec<Package>
```

**Filters Applied**:
- Selected managers (if any)
- Search query (case-insensitive substring match)
- Show outdated only
- Show orphaned only

**Code Reference**: [`src/app.rs:192-228`](src/app.rs)

**Example Usage**:
```rust
let filtered = app.filtered_packages();
for pkg in filtered {
    println!("{}: {}", pkg.name, pkg.installed_version);
}
```

##### `stats(&self) -> (usize, usize, usize)`

Returns statistics tuple: (total, outdated, unused).

```rust
pub fn stats(&self) -> (usize, usize, usize)
```

**Returns**: `(total_packages, outdated_count, unused_count)`

**Code Reference**: [`src/app.rs:230-240`](src/app.rs)

**Example Usage**:
```rust
let (total, outdated, unused) = app.stats();
println!("Total: {}, Outdated: {}, Unused: {}", total, outdated, unused);
```

##### `update_package(&mut self, package_name: String, manager: PackageManager)`

Updates a single package asynchronously.

```rust
pub fn update_package(&mut self, package_name: String, manager: PackageManager)
```

**Parameters**:
- `package_name`: Name of package to update
- `manager`: Package manager that owns this package

**Behavior**:
1. Adds package to `updating_packages` set
2. Shows spinner in UI
3. Spawns async task to run manager-specific update
4. Updates `packages` list on success
5. Removes from `updating_packages` set
6. Shows status message

**Code Reference**: [`src/app.rs:276-336`](src/app.rs)

**Example Usage**:
```rust
app.update_package("wget".to_string(), PackageManager::Homebrew);
```

##### `update_all_outdated(&mut self)`

Updates all outdated packages in bulk.

```rust
pub fn update_all_outdated(&mut self)
```

**Behavior**:
- Calls manager-specific bulk update function
- Shows progress in status area
- Refreshes package list on completion

**Code Reference**: [`src/app.rs:339-381`](src/app.rs)

##### `uninstall_package(&mut self, package_name: String, manager: PackageManager)`

Removes a package.

```rust
pub fn uninstall_package(&mut self, package_name: String, manager: PackageManager)
```

**Parameters**:
- `package_name`: Name of package to remove
- `manager`: Package manager that owns this package

**Behavior**:
1. Spawns async task to uninstall
2. On success, adds to `removed_packages` set
3. Package stays in UI with "Reinstall" button
4. Shows status message

**Code Reference**: [`src/app.rs:447-495`](src/app.rs)

##### `reinstall_package(&mut self, package_name: String, manager: PackageManager)`

Reinstalls a previously removed package.

```rust
pub fn reinstall_package(&mut self, package_name: String, manager: PackageManager)
```

**Parameters**:
- `package_name`: Name of package to reinstall
- `manager`: Package manager that owns this package

**Behavior**:
1. Spawns async task to install
2. On success, removes from `removed_packages` set
3. Button changes back to "Remove"
4. Shows status message

**Code Reference**: [`src/app.rs:397-444`](src/app.rs)

##### Helper Methods

```rust
pub fn is_updating(&self, package_name: &str) -> bool
pub fn is_removed(&self, package_name: &str) -> bool
pub fn get_update_status(&self) -> String
pub fn request_refresh(&mut self)
pub fn handle_refresh(&mut self)
```

**Code References**:
- `is_updating`: [`src/app.rs:383-387`](src/app.rs)
- `is_removed`: [`src/app.rs:389-391`](src/app.rs)
- `get_update_status`: [`src/app.rs:393-395`](src/app.rs)
- `request_refresh`: [`src/app.rs:181-183`](src/app.rs)
- `handle_refresh`: [`src/app.rs:185-190`](src/app.rs)

---

## Package Manager APIs

### Common Interface

All package manager modules implement these functions:

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

// Fetch descriptions in background
pub async fn add_<manager>_descriptions(packages: Arc<RwLock<Vec<Package>>>)
```

### Homebrew API (`homebrew_fast.rs`)

**File**: [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)

#### `list_homebrew_packages_fast() -> Result<Vec<Package>>`

Fetches all Homebrew packages using Formula API.

```rust
pub async fn list_homebrew_packages_fast() -> Result<Vec<Package>>
```

**Returns**: Vector of installed Homebrew packages with descriptions

**Behavior**:
1. Checks cache (returns immediately if cached)
2. Fetches all formulas from `https://formulae.brew.sh/api/formula.json`
3. Gets locally installed packages via `brew list --versions`
4. Filters API results to installed packages only
5. Caches results for 1 hour

**Performance**: 1-3 seconds (vs 5-7 minutes with old method)

**Code Reference**: [`src/managers/homebrew_fast.rs:22-100`](src/managers/homebrew_fast.rs)

**Example Usage**:
```rust
let packages = list_homebrew_packages_fast().await?;
println!("Found {} Homebrew packages", packages.len());
```

#### `check_outdated_packages_fast(packages: &mut [Package]) -> Result<()>`

Marks outdated packages by comparing installed vs latest versions.

```rust
pub async fn check_outdated_packages_fast(packages: &mut [Package]) -> Result<()>
```

**Parameters**:
- `packages`: Mutable slice of packages to check

**Behavior**:
- Compares `installed_version` vs `latest_version`
- Sets `is_outdated = true` for packages needing updates
- No CLI calls (data already from API)

**Performance**: Instant (no I/O)

**Code Reference**: [`src/managers/homebrew_fast.rs:102-150`](src/managers/homebrew_fast.rs)

#### `update_package(package_name: String) -> Result<()>`

Updates a single Homebrew package.

```rust
pub async fn update_package(package_name: String) -> Result<()>
```

**Parameters**:
- `package_name`: Name of package to update

**Command**: `brew upgrade <package_name>`

**Timeout**: 5 minutes

**Code Reference**: [`src/managers/homebrew_fast.rs:180-200`](src/managers/homebrew_fast.rs)

#### `update_all_packages() -> Result<()>`

Updates all outdated Homebrew packages.

```rust
pub async fn update_all_packages() -> Result<()>
```

**Command**: `brew upgrade`

**Timeout**: 10 minutes

**Code Reference**: [`src/managers/homebrew_fast.rs:152-178`](src/managers/homebrew_fast.rs)

#### `install_package(package_name: String) -> Result<()>`

Installs (or reinstalls) a Homebrew package.

```rust
pub async fn install_package(package_name: String) -> Result<()>
```

**Command**: `brew install <package_name>`

**Code Reference**: [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)

#### `uninstall_package(package_name: String) -> Result<()>`

Uninstalls a Homebrew package.

```rust
pub async fn uninstall_package(package_name: String) -> Result<()>
```

**Command**: `brew uninstall <package_name>`

**Code Reference**: [`src/managers/homebrew_fast.rs`](src/managers/homebrew_fast.rs)

### npm API (`npm.rs`)

**File**: [`src/managers/npm.rs`](src/managers/npm.rs)

#### `list_npm_packages() -> Result<Vec<Package>>`

Lists globally installed npm packages.

```rust
pub async fn list_npm_packages() -> Result<Vec<Package>>
```

**Command**: `npm list -g --depth=0 --json`

**Returns**: Vector of global npm packages

**Code Reference**: [`src/managers/npm.rs`](src/managers/npm.rs)

#### `check_outdated_npm(packages: &mut [Package]) -> Result<()>`

Checks for outdated npm packages.

```rust
pub async fn check_outdated_npm(packages: &mut [Package]) -> Result<()>
```

**Command**: `npm outdated -g --json`

**Updates**: `latest_version` and `is_outdated` fields

#### `update_npm_package(name: String) -> Result<()>`

Updates a global npm package.

```rust
pub async fn update_npm_package(name: String) -> Result<()>
```

**Command**: `npm update -g <name>`

#### `add_npm_descriptions(packages: Arc<RwLock<Vec<Package>>>)`

Fetches descriptions for npm packages in background.

```rust
pub async fn add_npm_descriptions(packages: Arc<RwLock<Vec<Package>>>)
```

**Command**: `npm view <package> description` (per package)

**Concurrency**: 8 parallel requests

**Code Reference**: [`src/managers/npm.rs`](src/managers/npm.rs)

### Cargo API (`cargo.rs`)

**File**: [`src/managers/cargo.rs`](src/managers/cargo.rs)

#### `list_cargo_packages() -> Result<Vec<Package>>`

Lists installed Cargo binaries.

```rust
pub async fn list_cargo_packages() -> Result<Vec<Package>>
```

**Command**: `cargo install --list`

**Parses**: Text output format (`package-name v1.2.3:`)

#### `check_outdated_cargo(packages: &mut [Package]) -> Result<()>`

Checks for outdated Cargo packages.

```rust
pub async fn check_outdated_cargo(packages: &mut [Package]) -> Result<()>
```

**Note**: Currently limited (needs `cargo-outdated` crate)

**TODO**: Integrate cargo-outdated or query crates.io API

#### `add_cargo_descriptions(packages: Arc<RwLock<Vec<Package>>>)`

Fetches descriptions from crates.io API.

```rust
pub async fn add_cargo_descriptions(packages: Arc<RwLock<Vec<Package>>>)
```

**API**: `https://crates.io/api/v1/crates/<package>`

**Concurrency**: 8 parallel requests

**Requires**: User-Agent header (crates.io policy)

**Code Reference**: [`src/managers/cargo.rs`](src/managers/cargo.rs)

### pip API (`pip.rs`)

**File**: [`src/managers/pip.rs`](src/managers/pip.rs)

#### `list_pip_packages() -> Result<Vec<Package>>`

Lists installed pip packages.

```rust
pub async fn list_pip_packages() -> Result<Vec<Package>>
```

**Command**: `pip3 list --format=json`

**Returns**: Vector of pip packages

#### `check_outdated_pip(packages: &mut [Package]) -> Result<()>`

Checks for outdated pip packages.

```rust
pub async fn check_outdated_pip(packages: &mut [Package]) -> Result<()>
```

**Command**: `pip3 list --outdated --format=json`

#### `update_pip_package(name: String) -> Result<()>`

Updates a pip package.

```rust
pub async fn update_pip_package(name: String) -> Result<()>
```

**Command**: `pip3 install --upgrade <name>`

#### `add_pip_descriptions(packages: Arc<RwLock<Vec<Package>>>)`

Fetches descriptions for pip packages.

```rust
pub async fn add_pip_descriptions(packages: Arc<RwLock<Vec<Package>>>)
```

**Command**: `pip3 show <package>` (per package)

**Parses**: "Summary: " line from output

**Concurrency**: 8 parallel requests

**Code Reference**: [`src/managers/pip.rs`](src/managers/pip.rs)

### Package Manager Detection (`detector.rs`)

**File**: [`src/managers/detector.rs`](src/managers/detector.rs)

#### `detect_available_managers() -> Vec<PackageManager>`

Detects which package managers are installed on the system.

```rust
pub async fn detect_available_managers() -> Vec<PackageManager>
```

**Method**: Runs `<manager> --version` for each potential manager

**Returns**: Vector of available `PackageManager` enums

**Example Usage**:
```rust
let managers = detect_available_managers().await;
println!("Found {} package managers", managers.len());
for mgr in managers {
    println!("  - {}", mgr.name());
}
```

**Code Reference**: [`src/managers/detector.rs`](src/managers/detector.rs)

---

## Scanner APIs

### Project Scanner (`scanner/project_scanner.rs`)

**File**: [`src/scanner/project_scanner.rs`](src/scanner/project_scanner.rs)

#### `get_scan_directories() -> Vec<PathBuf>`

Returns default directories to scan for projects.

```rust
pub fn get_scan_directories() -> Vec<PathBuf>
```

**Returns**: Vector of paths like `~/Desktop`, `~/projects`, etc.

**Code Reference**: [`src/scanner/mod.rs`](src/scanner/mod.rs)

#### `scan_homebrew_tool_usage(packages: &mut [Package], scan_dirs: &[PathBuf])`

Scans project directories to detect which tools are used where.

```rust
pub fn scan_homebrew_tool_usage(
    packages: &mut [Package],
    scan_dirs: &[PathBuf]
)
```

**Parameters**:
- `packages`: Mutable slice of packages to update
- `scan_dirs`: Directories to scan

**Behavior**:
1. Walks directories up to 4 levels deep
2. Detects project types by marker files:
   - `package.json` → uses `node`, `npm`
   - `Cargo.toml` → uses `rust`, `cargo`
   - `requirements.txt` → uses `python`, `pip`
   - `Gemfile` → uses `ruby`, `gem`
   - `.git` → uses `git`
   - `Dockerfile` → uses `docker`
3. Updates `used_in` field for matching packages

**Performance**: 2-5 seconds for typical project directories

**Code Reference**: [`src/scanner/project_scanner.rs:7-226`](src/scanner/project_scanner.rs)

**Example Usage**:
```rust
let scan_dirs = get_scan_directories();
scan_homebrew_tool_usage(&mut packages, &scan_dirs);

for pkg in packages {
    if pkg.used_in.is_empty() {
        println!("{} is unused", pkg.name);
    } else {
        println!("{} used in {} projects", pkg.name, pkg.used_in.len());
    }
}
```

---

## Utility APIs

### Cache (`utils/cache.rs`)

**File**: [`src/utils/cache.rs`](src/utils/cache.rs)

#### `get_cached<T>(key: &str) -> Option<T>`

Retrieves cached value if not expired.

```rust
pub fn get_cached<T: DeserializeOwned>(key: &str) -> Option<T>
```

**Parameters**:
- `key`: Cache key (e.g., "homebrew_all_packages")

**Returns**: `Some(value)` if cached and not expired, `None` otherwise

**Example Usage**:
```rust
if let Some(packages) = get_cached::<Vec<Package>>("homebrew_all_packages") {
    return Ok(packages);  // Cache hit!
}
```

#### `set_cached<T>(key: String, value: &T, ttl_seconds: u64)`

Stores value in cache with TTL.

```rust
pub fn set_cached<T: Serialize>(key: String, value: &T, ttl_seconds: u64)
```

**Parameters**:
- `key`: Cache key
- `value`: Value to cache (must be serializable)
- `ttl_seconds`: Time-to-live in seconds

**Example Usage**:
```rust
set_cached("homebrew_all_packages".to_string(), &packages, 3600);  // 1 hour
```

**Code Reference**: [`src/utils/cache.rs`](src/utils/cache.rs)

### HTTP Client (`utils/http_client.rs`)

**File**: [`src/utils/http_client.rs:1-13`](src/utils/http_client.rs)

#### `create_http_client() -> Client`

Creates a high-performance HTTP client with connection pooling.

```rust
pub fn create_http_client() -> Client
```

**Configuration**:
- Pool size: 10 connections per host
- Idle timeout: 90 seconds
- Request timeout: 30 seconds
- Gzip compression: Enabled

**Returns**: Configured `reqwest::Client`

**Example Usage**:
```rust
let client = create_http_client();
let response = client
    .get("https://api.example.com/data")
    .send()
    .await?;
```

### Command Runner (`utils/command.rs`)

**File**: [`src/utils/command.rs`](src/utils/command.rs)

#### Command Execution Pattern

**Standard Pattern** used throughout codebase:

```rust
use tokio::process::Command;
use tokio::time::{timeout, Duration};

async fn run_command_example() -> Result<String> {
    let child = Command::new("brew")
        .arg("list")
        .arg("--versions")
        .output();
    
    let output = timeout(Duration::from_secs(30), child)
        .await
        .map_err(|_| anyhow!("Command timed out"))?
        .map_err(|e| anyhow!("Failed to execute: {}", e))?;
    
    if !output.status.success() {
        return Err(anyhow!("Command failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

**Pattern Used In**:
- All package manager list functions
- All update/install/uninstall functions
- Description fetching

---

## Data Models

### Package (`models/package.rs`)

**File**: [`src/models/package.rs:62-72`](src/models/package.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub manager: PackageManager,
    pub installed_version: String,
    pub latest_version: Option<String>,
    pub is_outdated: bool,
    pub size: Option<u64>,
    pub description: Option<String>,
    pub used_in: Vec<String>,
}
```

**Fields**:
- `name`: Package identifier (e.g., "wget")
- `manager`: Which package manager owns it
- `installed_version`: Currently installed version string
- `latest_version`: Available version (None if up-to-date)
- `is_outdated`: Computed flag for filtering
- `size`: Disk space in bytes (optional)
- `description`: Human-readable purpose (optional)
- `used_in`: Project paths that use this package

### PackageManager (`models/package.rs`)

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
```rust
impl PackageManager {
    pub fn name(&self) -> &'static str      // Display name
    pub fn command(&self) -> &'static str   // CLI command
}
```

### Project (`models/project.rs`)

**File**: [`src/models/project.rs`](src/models/project.rs)

```rust
#[derive(Debug, Clone)]
pub struct Project {
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectType {
    Node,
    Rust,
    Python,
    Ruby,
    Go,
    Unknown,
}

pub fn new(path: PathBuf) -> Self  // Constructor
```

### Dependency (`models/project.rs`)

```rust
#[derive(Debug, Clone)]
pub struct Dependency {
    pub package_name: String,
    pub manager: PackageManager,
    pub version_constraint: String,
    pub is_dev: bool,
}
```

### PackageUsage (`models/usage.rs`)

**File**: [`src/models/usage.rs`](src/models/usage.rs)

```rust
#[derive(Debug, Clone)]
pub struct PackageUsage {
    pub package: Package,
    pub projects: Vec<Project>,
}

pub fn new(package: Package) -> Self
pub fn add_project(&mut self, project: Project)
```

---

## Type Reference

### Common Result Type

```rust
use anyhow::Result;

// All fallible functions return Result<T>
pub async fn list_packages() -> Result<Vec<Package>>
```

### Async Patterns

**Tokio Async**:
```rust
#[tokio::main]
async fn main() {
    let packages = list_packages().await.unwrap();
}
```

**Spawning Background Tasks**:
```rust
tokio::spawn(async move {
    let result = expensive_operation().await;
    // Process result...
});
```

### Shared State

**Arc + RwLock Pattern**:
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

let packages: Arc<RwLock<Vec<Package>>> = Arc::new(RwLock::new(Vec::new()));

// Clone Arc (cheap):
let packages_clone = Arc::clone(&packages);

// Write access (async):
*packages.write().await = new_packages;

// Read access (async):
let packages_ref = packages.read().await;

// Read access (blocking, from GUI thread):
let packages_ref = packages.blocking_read();
```

**Atomic Bool Pattern**:
```rust
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

let is_scanning: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

// Set:
is_scanning.store(true, Ordering::Relaxed);

// Get:
let scanning = is_scanning.load(Ordering::Relaxed);
```

---

## Example: Adding a New Package Manager

**Complete example for adding yarn support:**

**1. Update PackageManager enum**:

```rust
// src/models/package.rs
pub enum PackageManager {
    // ... existing ...
    Yarn,
}

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

**2. Create yarn.rs module**:

```rust
// src/managers/yarn.rs
use crate::models::{Package, PackageManager};
use anyhow::{anyhow, Result};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn list_yarn_packages() -> Result<Vec<Package>> {
    let cmd = Command::new("yarn")
        .arg("global")
        .arg("list")
        .arg("--json")
        .output();
    
    let output = timeout(Duration::from_secs(30), cmd)
        .await??;
    
    if !output.status.success() {
        return Err(anyhow!("yarn global list failed"));
    }
    
    // Parse JSON output...
    let packages = parse_yarn_output(&output.stdout)?;
    Ok(packages)
}

pub async fn update_yarn_package(name: String) -> Result<()> {
    let cmd = Command::new("yarn")
        .arg("global")
        .arg("upgrade")
        .arg(&name)
        .output();
    
    let output = timeout(Duration::from_secs(300), cmd)
        .await??;
    
    if !output.status.success() {
        return Err(anyhow!("yarn upgrade failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

// ... implement other functions ...
```

**3. Add to detector**:

```rust
// src/managers/detector.rs
if command_exists("yarn").await {
    managers.push(PackageManager::Yarn);
}
```

**4. Add to app scanning**:

```rust
// src/app.rs (in start_scan method)
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

**5. Add to update dispatch**:

```rust
// src/app.rs (in update_package method)
let result = match manager {
    // ... existing ...
    PackageManager::Yarn => 
        crate::managers::yarn::update_yarn_package(package_name.clone()).await,
    // ...
};
```

---

## Further Reading

- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System design and component interactions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development workflow and code style
- **[README.md](README.md)** - Project overview and features

---

**API documentation complete.** For questions or clarifications, consult the source code with references provided above.

