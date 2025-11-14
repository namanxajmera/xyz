# Dependency Manager Tool - Brainstorming & Research

## Problem Statement

User has multiple package managers installed (Homebrew, npm, yarn, pnpm, cargo, pip, etc.) and faces these challenges:
1. Packages get outdated across different managers
2. Projects are deleted but dependencies remain installed globally
3. No unified view of what's installed where
4. No easy way to see which packages are used in which projects
5. No one-click update all functionality

## Goal

Create a Rust GUI application that:
- Aggregates information from all package managers
- Shows unified **visual** view of installed packages
- Identifies outdated packages with clear visual indicators
- Maps packages to projects that use them
- Provides **one-click** update capabilities (individual and bulk)
- Helps identify orphaned packages (installed but not used)
- **Why GUI?** Terminal commands already exist - we need visual organization, filtering, sorting, progress indicators, and easy bulk actions

---

## Package Managers to Support

### System-Level Package Managers
1. **Homebrew** (macOS)
   - Commands: `brew list`, `brew outdated`, `brew upgrade`, `brew info`
   - Both regular packages and casks
   - JSON output available: `brew list --json`

2. **MacPorts** (macOS alternative)
   - Commands: `port installed`, `port outdated`, `port upgrade`

3. **APT** (Linux - future consideration)
   - Commands: `apt list --installed`, `apt list --upgradable`

### Language-Specific Package Managers

4. **Node.js Ecosystem**
   - **npm** (global): `npm list -g --depth=0`, `npm outdated -g`, `npm update -g`
   - **yarn** (global): `yarn global list`, `yarn global outdated`
   - **pnpm** (global): `pnpm list -g --depth=0`, `pnpm outdated -g`
   - Project detection: Look for `package.json`, `yarn.lock`, `pnpm-lock.yaml`

5. **Rust**
   - **Cargo** (global): `cargo install --list`, `cargo install-update -l`
   - Project detection: Look for `Cargo.toml`

6. **Python**
   - **pip** (global): `pip list`, `pip list --outdated`
   - **pipx** (isolated apps): `pipx list`
   - Project detection: Look for `requirements.txt`, `pyproject.toml`, `Pipfile`, `poetry.lock`

7. **Ruby**
   - **gem** (global): `gem list`, `gem outdated`
   - Project detection: Look for `Gemfile`, `Gemfile.lock`

8. **Go**
   - **go install**: Check `$GOPATH/bin` or `$GOBIN`
   - Project detection: Look for `go.mod`, `go.sum`

9. **PHP**
   - **Composer** (global): `composer global show`, `composer global outdated`
   - Project detection: Look for `composer.json`, `composer.lock`

10. **Dart/Flutter**
    - **pub** (global): `pub global list`
    - Project detection: Look for `pubspec.yaml`

11. **Swift**
    - **Swift Package Manager**: Check for `Package.swift`
    - Project detection: Look for `Package.swift`

---

## Core Features

### 1. Discovery & Scanning
- **Detect available package managers** (check if commands exist)
- **Scan system** for installed packages per manager
- **Scan projects** in common directories (e.g., `~/Projects`, `~/Code`, `~/Development`)
- **Parse dependency files** to map packages to projects

### 2. Data Model

```rust
// Package Manager Type
enum PackageManager {
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
    // ... etc
}

// Package Information
struct Package {
    name: String,
    manager: PackageManager,
    installed_version: String,
    latest_version: Option<String>,
    is_outdated: bool,
    install_date: Option<DateTime>,
    size: Option<u64>, // disk space used
}

// Project Information
struct Project {
    path: PathBuf,
    name: String,
    package_managers: Vec<PackageManager>,
    dependencies: Vec<Dependency>,
    last_modified: DateTime,
}

// Dependency Mapping
struct Dependency {
    package_name: String,
    manager: PackageManager,
    version_constraint: String, // e.g., "^1.2.3", ">=2.0.0"
    is_dev: bool, // dev dependency vs production
}

// Usage Mapping
struct PackageUsage {
    package: Package,
    used_in_projects: Vec<Project>,
    is_orphaned: bool, // not used in any project
}
```

### 3. GUI Views & Features

#### Main Window - Dashboard View
**Left Sidebar:**
- Package Manager filters (Homebrew, npm, Cargo, etc.)
- Quick stats: Total packages, Outdated count, Orphaned count
- Project count

**Main Panel - Package Table:**
- Sortable columns: Name, Manager, Installed Version, Latest Version, Status, Used In Projects
- Checkboxes for bulk selection
- Color coding: Red for outdated, Gray for orphaned, Green for up-to-date
- Search/filter bar at top
- "Update Selected" button
- "Update All Outdated" button

**Features:**
- Click package name → Show details panel (right side)
- Details panel shows: Description, Size, Install date, Projects using it
- Progress bar during updates
- Toast notifications for update completion

#### Projects View
- List of detected projects
- Click project → Show dependencies
- Visual indicators for outdated dependencies
- "Update Project Dependencies" button per project

#### Orphaned Packages View
- Filtered list of packages not used in any project
- Bulk selection
- "Remove Selected" button with confirmation dialog
- Shows disk space that could be freed

#### Update Progress View
- Shows update progress for each package
- Real-time status updates
- Can cancel updates
- Shows errors if any occur

### 4. GUI-Specific Features

- **Visual Organization**: Tables, cards, sidebars - much better than terminal output
- **Filtering & Sorting**: Click column headers, search bar, filter dropdowns
- **Bulk Actions**: Select multiple packages, update all at once
- **Progress Indicators**: Visual progress bars during updates
- **Notifications**: Toast notifications for completed updates
- **One-Click Actions**: Buttons instead of typing commands
- **Visual Status**: Color coding, icons, badges
- **Responsive Layout**: Resizable windows, collapsible panels

---

## Architecture

### Project Structure
```
depmgr/
├── Cargo.toml
├── src/
│   ├── main.rs              # GUI application entry point
│   ├── app.rs               # Main application state and logic
│   ├── ui/                  # GUI components
│   │   ├── mod.rs
│   │   ├── dashboard.rs     # Main dashboard view
│   │   ├── package_table.rs # Package list table widget
│   │   ├── project_view.rs  # Project dependencies view
│   │   ├── update_dialog.rs # Update progress dialog
│   │   └── widgets.rs       # Reusable UI widgets
│   ├── managers/            # Package manager implementations
│   │   ├── mod.rs
│   │   ├── homebrew.rs
│   │   ├── npm.rs
│   │   ├── cargo.rs
│   │   ├── pip.rs
│   │   └── ...
│   ├── scanner/             # Project scanning logic
│   │   ├── mod.rs
│   │   ├── detector.rs      # Detect package managers
│   │   └── parser.rs        # Parse dependency files
│   ├── models/              # Data models
│   │   ├── mod.rs
│   │   ├── package.rs
│   │   ├── project.rs
│   │   └── usage.rs
│   ├── storage/             # Cache/storage
│   │   ├── mod.rs
│   │   └── cache.rs         # Cache scan results
│   └── utils/
│       ├── mod.rs
│       ├── command.rs       # Execute shell commands
│       └── format.rs        # Formatting helpers
└── tests/
```

### Key Design Decisions

1. **GUI Framework**: Use **egui** (immediate mode GUI) - perfect for tools/dashboards
   - Cross-platform, native feel
   - Great for tables, lists, buttons
   - Easy to build responsive UIs
   - Good performance
2. **Async/Await**: Use `tokio` for concurrent package manager queries
3. **Command Execution**: Use `tokio::process::Command` for async command execution
4. **Caching**: Cache scan results to avoid repeated expensive operations
5. **Config File**: `~/.config/depmgr/config.toml` for:
   - Project directories to scan
   - Ignore patterns
   - Update preferences
   - Package manager preferences
6. **Window Management**: Use `eframe` (egui's framework) for window handling

---

## Rust Libraries to Consider

### GUI Framework (Primary)
- **egui** + **eframe**: Immediate mode GUI framework
  - Perfect for tools and dashboards
  - Cross-platform (macOS, Windows, Linux)
  - Great table/list widgets
  - Easy to build responsive UIs
  - Good performance
  - Active development, popular in Rust community
  - Alternative: **Iced** (declarative, Elm-inspired) or **Tauri** (web-based frontend)

### GUI Alternatives
- **Iced**: Declarative GUI framework
  - More structured than egui
  - Good for complex UIs
  - Cross-platform
- **Tauri**: Web frontend (HTML/CSS/JS) + Rust backend
  - Modern web UI capabilities
  - Larger bundle size
  - Good if you want web-like UI

### Async Runtime
- **tokio**: Async runtime for concurrent operations
  - Needed for parallel package manager queries
  - Process execution

### Serialization
- **serde**: Serialization framework
- **toml**: TOML parsing (for config, Cargo.toml, etc.)
- **serde_json**: JSON parsing (npm, brew JSON outputs)

### File System & Paths
- **walkdir**: Recursive directory traversal
- **glob**: Pattern matching for file discovery
- **dirs**: Standard directories (home, config, etc.)

### Error Handling
- **anyhow**: Flexible error handling
- **thiserror**: Custom error types

### Utilities
- **regex**: Pattern matching for version parsing
- **chrono**: Date/time handling
- **indicatif**: Progress bars
- **colored**: Terminal colors

---

## Implementation Phases

### Phase 1: MVP - Core GUI & Basic Functionality
1. Set up egui + eframe window
2. Basic dashboard layout (sidebar + main panel)
3. Detect available package managers
4. List installed packages in table view
5. Check for outdated packages
6. Support: Homebrew, npm, Cargo
7. Basic filtering and sorting

### Phase 2: Project Scanning & Mapping
1. Scan directories for projects
2. Parse dependency files
3. Map packages to projects
4. Identify orphaned packages
5. Show "Used In Projects" column
6. Support: npm, yarn, pnpm, Cargo, pip
7. Projects view tab

### Phase 3: Update Functionality
1. Update individual packages (button click)
2. Bulk selection and update
3. Update progress dialog with progress bars
4. Update confirmation dialogs
5. Error handling and notifications
6. "Update All Outdated" button

### Phase 4: Enhanced GUI Features
1. Caching system (avoid re-scanning)
2. Config file support
3. Settings window
4. More package managers
5. Search/filter improvements
6. Package details panel
7. Orphaned packages view

### Phase 5: Polish & Optimization
1. Better error messages and dialogs
2. Toast notifications
3. Keyboard shortcuts
4. Logging
5. Documentation
6. Performance optimizations
7. App icon and branding

---

## Technical Challenges

### 1. Command Execution
- Different package managers have different command syntax
- Some require sudo (handle gracefully)
- Error handling for missing commands
- Timeout handling for slow operations

### 2. Version Parsing
- Different version formats (semver, date-based, etc.)
- Comparing versions accurately
- Handling pre-release versions

### 3. Project Detection
- Finding projects efficiently
- Handling nested projects
- Ignoring node_modules, target/, etc.
- Performance on large directory trees

### 4. Dependency Mapping
- Matching global packages to project dependencies
- Handling version constraints (^, ~, >=, etc.)
- Dev vs production dependencies
- Workspace/monorepo handling

### 5. Cross-Platform Support
- Initially macOS-focused (Homebrew)
- Linux support (APT, yum, etc.)
- Windows support (chocolatey, scoop) - future

---

## Configuration File Example

```toml
# ~/.config/depmgr/config.toml

[scan]
# Directories to scan for projects
project_dirs = [
    "~/Projects",
    "~/Code",
    "~/Development",
]

# Patterns to ignore
ignore_patterns = [
    "**/node_modules/**",
    "**/target/**",
    "**/.git/**",
    "**/venv/**",
    "**/__pycache__/**",
]

[managers]
# Enable/disable specific managers
homebrew = true
npm = true
yarn = true
pnpm = true
cargo = true
pip = true

[updates]
# Auto-update behavior
confirm_before_update = true
dry_run_by_default = false

[cache]
# Cache settings
enabled = true
ttl_hours = 24
```

---

## Open Questions

1. **Storage**: SQLite database vs JSON cache file?
   - SQLite: Better for queries, relationships
   - JSON: Simpler, human-readable

2. **Real-time vs Cached**: Always scan or cache results?
   - Cache with TTL for performance
   - Force refresh flag

3. **Update Strategy**: 
   - Always update to latest?
   - Respect semver constraints?
   - Update all or selective?

4. **Project Detection**:
   - How deep to scan?
   - Handle monorepos?
   - Git-based detection?

5. **UI Preference**:
   - CLI-only or TUI?
   - Both options?

---

## Next Steps

1. Set up Rust project structure
2. Implement package manager detection
3. Start with Homebrew implementation (simplest)
4. Add npm support
5. Build basic CLI interface
6. Iterate based on usage

---

## Research Notes

### Homebrew JSON Output
```bash
brew list --json=v2
# Returns JSON array of package objects
```

### npm JSON Output
```bash
npm list -g --json --depth=0
# Returns nested JSON structure
```

### Cargo Install List
```bash
cargo install --list
# Plain text, needs parsing
```

### Project Detection Strategy
- Look for lock files: `package-lock.json`, `yarn.lock`, `Cargo.lock`
- Look for manifest files: `package.json`, `Cargo.toml`, `requirements.txt`
- Check modification dates to determine active projects

