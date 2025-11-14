# Project Summary: Unified Package Manager Tool

## Vision

A Rust **GUI application** that provides a unified visual dashboard for managing packages across all your package managers (Homebrew, npm, yarn, pnpm, Cargo, pip, etc.). See everything in one place with tables, filters, and visual indicators. Identify outdated packages, find orphaned dependencies, and update with **one click** - no terminal commands needed!

## Key Features

### 1. **Unified Dashboard**
- See all installed packages across all package managers in one view
- Quick overview: "45 Homebrew packages (12 outdated), 23 npm packages (5 outdated)"
- Filter by manager, outdated status, or orphaned packages

### 2. **Project Mapping**
- Automatically scans your project directories
- Maps global packages to projects that use them
- Identifies orphaned packages (installed globally but not used anywhere)

### 3. **Update Management**
- Check for updates across all managers
- Update individual packages or bulk update
- One-click "update all" option
- Dry-run mode to preview changes

### 4. **Smart Detection**
- Auto-detects which package managers you have installed
- Scans common project directories (~/Projects, ~/Code, etc.)
- Parses dependency files (package.json, Cargo.toml, requirements.txt, etc.)

## Supported Package Managers

**System-Level:**
- Homebrew (macOS)
- MacPorts (macOS)

**Language-Specific:**
- npm, yarn, pnpm (Node.js)
- Cargo (Rust)
- pip, pipx (Python)
- gem (Ruby)
- Go modules
- Composer (PHP)
- pub (Dart/Flutter)
- Swift Package Manager

## GUI Features

- **Visual Dashboard**: Tables, sidebars, panels - organized and easy to scan
- **One-Click Updates**: Buttons instead of typing commands
- **Bulk Actions**: Select multiple packages, update all at once
- **Filtering & Sorting**: Click column headers, search bar, filter dropdowns
- **Progress Indicators**: Visual progress bars during updates
- **Notifications**: Toast notifications for completed updates
- **Color Coding**: Red for outdated, gray for orphaned, green for up-to-date
- **Project Mapping**: See which projects use which packages
- **Orphaned Detection**: Find unused packages taking up space

## Architecture Highlights

- **Modular Design**: Each package manager is a separate module
- **Async Operations**: Parallel queries for fast scanning
- **Caching**: Cache results to avoid repeated expensive operations
- **Configurable**: Config file for scan directories, preferences
- **Extensible**: Easy to add new package managers

## Implementation Plan

**Phase 1 (MVP):** Core functionality with Homebrew, npm, Cargo
**Phase 2:** Project scanning and dependency mapping
**Phase 3:** Update functionality
**Phase 4:** Interactive TUI and advanced features
**Phase 5:** Polish, optimization, documentation

## Technical Stack

- **Language**: Rust
- **GUI Framework**: egui + eframe (immediate mode GUI)
  - Perfect for dashboards and tools
  - Cross-platform (macOS, Windows, Linux)
  - Great table/list widgets
- **Async**: tokio (for concurrent package manager queries)
- **Serialization**: serde, toml, serde_json
- **File Operations**: walkdir, glob

## Next Steps

1. Review this brainstorming and provide feedback
2. Confirm feature priorities
3. Start implementation with MVP (Phase 1)
4. Iterate based on usage and feedback

---

See `docs/brainstorm.md` for detailed technical design and `docs/quick-reference.md` for command reference.

