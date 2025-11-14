# Contributing Guidelines

**Development workflow and guidelines for contributing to DepMgr**

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Code Style](#code-style)
- [Development Workflow](#development-workflow)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Adding New Features](#adding-new-features)
- [Code Review Guidelines](#code-review-guidelines)

---

## Getting Started

We welcome contributions to DepMgr! Whether you're fixing bugs, adding features, or improving documentation, your help is appreciated.

**Ways to Contribute**:
- 🐛 Report bugs
- 💡 Suggest features
- 📝 Improve documentation
- 🔧 Fix issues
- ✨ Add new package manager support
- 🎨 Improve UI/UX

---

## Development Setup

### Prerequisites

**Required**:
- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **Git** for version control
- At least one package manager to test (Homebrew, npm, cargo, pip)

**Recommended**:
- **VS Code** with rust-analyzer extension
- **cargo-watch** for auto-reload: `cargo install cargo-watch`
- **cargo-edit** for dependency management: `cargo install cargo-edit`

### Clone and Build

```bash
# 1. Fork the repository on GitHub

# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/depmgr.git
cd depmgr

# 3. Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/depmgr.git

# 4. Build and run
cargo build
cargo run
```

### Development Tools

**Install Recommended Tools**:

```bash
# Code formatter
rustup component add rustfmt

# Linter
rustup component add clippy

# Auto-reload on file changes
cargo install cargo-watch

# Dependency management
cargo install cargo-edit
```

**IDE Setup**: See [`docs/SETUP.md`](docs/SETUP.md#ide-setup) for VS Code and IntelliJ configuration

---

## Code Style

### Rust Style Guidelines

**Follow Rust conventions**:
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common mistakes
- Follow naming conventions:
  - `snake_case` for functions, variables, modules
  - `PascalCase` for types, structs, enums
  - `SCREAMING_SNAKE_CASE` for constants

**Format Code**:

```bash
# Format all code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check
```

**Run Linter**:

```bash
# Check for issues
cargo clippy

# Auto-fix where possible
cargo clippy --fix
```

### Project-Specific Conventions

**1. Error Handling**:

```rust
// Use anyhow::Result for fallible functions
use anyhow::{anyhow, Result};

pub async fn list_packages() -> Result<Vec<Package>> {
    // ... implementation
}

// Provide context in errors
.map_err(|e| anyhow!("Failed to parse JSON: {}", e))?
```

**Code Reference**: Error handling pattern throughout codebase

**2. Async Functions**:

```rust
// Use async/await for I/O operations
pub async fn fetch_data() -> Result<Data> {
    let client = create_http_client();
    let response = client.get(url).send().await?;
    // ...
}
```

**Code Reference**: Async patterns in [`src/managers/`](src/managers/) modules

**3. Logging**:

```rust
// Use println! with prefixes for console output
println!("[DEBUG] Processing {} packages", count);
println!("[INFO] Successfully updated {}", name);
println!("[ERROR] Failed to fetch: {}", error);
```

**Code Reference**: Logging examples in [`src/app.rs`](src/app.rs) and manager modules

**4. Comments**:

```rust
// Good: Explain WHY, not WHAT
// Use parallel processing because API returns 6k+ formulas
let packages: Vec<Package> = formulas
    .par_iter()
    .filter_map(|f| ...)
    .collect();

// Bad: States the obvious
// Loop through formulas
for formula in formulas {
    // ...
}
```

---

## Development Workflow

### Branch Strategy

**Branch Naming**:
- `feature/add-yarn-support` - New features
- `fix/homebrew-timeout` - Bug fixes
- `docs/update-readme` - Documentation
- `refactor/optimize-cache` - Code refactoring

**Workflow**:

```bash
# 1. Sync with upstream
git fetch upstream
git checkout master
git merge upstream/master

# 2. Create feature branch
git checkout -b feature/add-yarn-support

# 3. Make changes and commit
git add .
git commit -m "Add yarn package manager support"

# 4. Push to your fork
git push origin feature/add-yarn-support

# 5. Create Pull Request on GitHub
```

### Commit Messages

**Format**:

```
<type>: <subject>

<body (optional)>

<footer (optional)>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `refactor`: Code refactoring
- `perf`: Performance improvement
- `test`: Tests
- `chore`: Build/tooling changes

**Examples**:

```
feat: Add yarn package manager support

Implements list, update, and description fetching for yarn.
Adds yarn.rs module following existing pattern.

Closes #42
```

```
fix: Fix timeout issue in Homebrew scanning

Reduced batch size from 20 to 10 packages to avoid timeouts
on slower connections.
```

```
docs: Update README with performance benchmarks

Added table showing 10-20x performance improvement with
new API-based approach.
```

### Running Tests

```bash
# Type checking (fast)
cargo check

# Linting
cargo clippy

# Formatting check
cargo fmt -- --check

# Run all checks
cargo check && cargo clippy && cargo fmt --check

# Run tests (when available)
cargo test
```

**Pre-Commit Checklist**:
- [ ] Code compiles: `cargo check`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`
- [ ] Application runs: `cargo run`
- [ ] No regressions in functionality

---

## Testing

### Manual Testing

**Test Checklist**:

- [ ] Application launches
- [ ] All package managers detected
- [ ] Packages load within 60 seconds
- [ ] Search works
- [ ] Filters work (outdated, manager selection)
- [ ] Update button works for outdated packages
- [ ] Remove button works
- [ ] Reinstall button works after removal
- [ ] Refresh button triggers rescan
- [ ] Column resizing works
- [ ] No crashes during normal use

### Testing New Package Managers

**If adding support for a new package manager**:

```bash
# 1. Verify manager is installed
yarn --version

# 2. Install test packages
yarn global add some-package

# 3. Test list function
# Should appear in DepMgr table

# 4. Test update function
# Click Update button, verify it works

# 5. Test remove function
# Click Remove button, verify it uninstalls

# 6. Test reinstall function
# Click Reinstall button, verify it installs
```

### Unit Tests (Future)

**Pattern for writing tests**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_packages() {
        let packages = list_homebrew_packages().await.unwrap();
        assert!(!packages.is_empty());
    }

    #[test]
    fn test_filter_packages() {
        let mut app = DepMgrApp::default();
        app.search_query = "wget".to_string();
        let filtered = app.filtered_packages();
        // assertions...
    }
}
```

**Note**: Comprehensive test suite is planned for v0.2.0

---

## Documentation

### Code Documentation

**Document public APIs**:

```rust
/// Fetches all Homebrew packages using the Formula API.
///
/// # Returns
///
/// * `Ok(Vec<Package>)` - List of installed Homebrew packages
/// * `Err(anyhow::Error)` - If API fetch or parsing fails
///
/// # Performance
///
/// - First call: 1-3 seconds (API fetch + parsing)
/// - Cached calls: <100ms (returns cached data)
///
/// # Example
///
/// ```rust
/// let packages = list_homebrew_packages_fast().await?;
/// println!("Found {} packages", packages.len());
/// ```
pub async fn list_homebrew_packages_fast() -> Result<Vec<Package>> {
    // implementation...
}
```

### Updating Documentation

**When to Update Docs**:
- Adding new features → Update [`README.md`](README.md)
- Changing architecture → Update [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- Adding APIs → Update [`docs/API.md`](docs/API.md)
- Finding bugs → Update [`docs/TROUBLESHOOTING.md`](docs/TROUBLESHOOTING.md)

**Documentation Standards**:
- Include code references with file paths
- Provide working examples
- Link related documentation
- Keep examples up-to-date with code

---

## Pull Request Process

### Before Submitting

**1. Self-Review**:
- [ ] Code compiles without warnings
- [ ] Follows style guidelines
- [ ] Includes comments where needed
- [ ] No debug code or print statements (except intentional logging)

**2. Test Your Changes**:
- [ ] Manual testing completed
- [ ] No regressions
- [ ] Works on your system

**3. Update Documentation**:
- [ ] README updated if needed
- [ ] Code comments added
- [ ] API docs updated if adding new APIs

### Pull Request Template

```markdown
## Description

Brief description of changes

## Type of Change

- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update

## Testing

- [ ] Tested locally
- [ ] No regressions
- [ ] Works with [list package managers tested]

## Checklist

- [ ] Code follows style guidelines
- [ ] Ran `cargo fmt`
- [ ] Ran `cargo clippy` (no warnings)
- [ ] Documentation updated
- [ ] Commit messages follow conventions

## Screenshots (if applicable)

[Add screenshots for UI changes]
```

### Review Process

**What Reviewers Look For**:
1. **Code Quality**: Follows Rust best practices
2. **Functionality**: Works as intended, no regressions
3. **Documentation**: Adequate comments and docs
4. **Style**: Consistent with codebase
5. **Performance**: No obvious performance issues

**Addressing Feedback**:
- Be responsive to review comments
- Make requested changes in new commits (don't force push)
- Explain your reasoning if you disagree

---

## Adding New Features

### Adding a New Package Manager

**Complete guide with code references**:

**1. Update PackageManager Enum** ([`src/models/package.rs:5-18`](src/models/package.rs)):

```rust
pub enum PackageManager {
    // ... existing ...
    Yarn,  // Add new variant
}
```

**2. Implement Methods** ([`src/models/package.rs:20-54`](src/models/package.rs)):

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

**3. Create Manager Module** (`src/managers/yarn.rs`):

```rust
use crate::models::{Package, PackageManager};
use anyhow::{anyhow, Result};

/// List globally installed yarn packages
pub async fn list_yarn_packages() -> Result<Vec<Package>> {
    // Implementation using `yarn global list --json`
}

/// Check which yarn packages are outdated
pub async fn check_outdated_yarn(packages: &mut [Package]) -> Result<()> {
    // Implementation using `yarn outdated --json`
}

/// Update a yarn package
pub async fn update_yarn_package(name: String) -> Result<()> {
    // Implementation using `yarn global upgrade <name>`
}

/// Install a yarn package
pub async fn install_yarn_package(name: String) -> Result<()> {
    // Implementation
}

/// Uninstall a yarn package
pub async fn uninstall_yarn_package(name: String) -> Result<()> {
    // Implementation
}

/// Fetch descriptions for yarn packages
pub async fn add_yarn_descriptions(packages: Arc<RwLock<Vec<Package>>>) {
    // Implementation
}
```

**4. Add to Module Exports** ([`src/managers/mod.rs`](src/managers/mod.rs)):

```rust
pub mod yarn;
```

**5. Add to Detector** ([`src/managers/detector.rs`](src/managers/detector.rs)):

```rust
if command_exists("yarn").await {
    managers.push(PackageManager::Yarn);
}
```

**6. Add to Scan Flow** ([`src/app.rs:41-179`](src/app.rs)):

```rust
// In start_scan() method:
if available_managers.contains(&PackageManager::Yarn) {
    println!("[DEBUG] Scanning yarn packages...");
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

**7. Add to Update Dispatch** ([`src/app.rs:286-302`](src/app.rs)):

```rust
let result = match manager {
    // ... existing ...
    PackageManager::Yarn => 
        crate::managers::yarn::update_yarn_package(package_name.clone()).await,
    // ...
};
```

**8. Test Thoroughly**:
- List packages
- Check outdated detection
- Update package
- Remove package
- Reinstall package

**Reference Implementation**: See [`src/managers/npm.rs`](src/managers/npm.rs) for complete example

### Adding UI Features

**Pattern**: Modify [`src/ui/dashboard.rs`](src/ui/dashboard.rs)

**Example: Add New Column**:

```rust
// 1. Add to TableBuilder
.column(Column::initial(100.0).at_least(60.0).resizable(true))

// 2. Add header
header.col(|ui| { ui.strong("New Column"); });

// 3. Add cell content
row.col(|ui| {
    ui.label(format!("{}", pkg.new_field));
});
```

**Reference**: Table implementation in [`src/ui/dashboard.rs:134-288`](src/ui/dashboard.rs)

---

## Code Review Guidelines

### For Contributors

**Before Requesting Review**:
- Self-review your changes
- Test thoroughly
- Write clear PR description
- Reference related issues

**During Review**:
- Respond to feedback promptly
- Ask questions if unclear
- Make requested changes
- Keep discussions constructive

### For Reviewers

**What to Check**:
1. **Functionality**: Does it work? Any edge cases?
2. **Code Quality**: Readable? Well-structured?
3. **Performance**: Any obvious bottlenecks?
4. **Style**: Follows conventions?
5. **Documentation**: Adequately documented?
6. **Tests**: Tested? Manual test results shared?

**Providing Feedback**:
- Be constructive and specific
- Explain *why* something should be changed
- Suggest alternatives
- Acknowledge good work

**Example Comments**:

✅ Good:
```
Consider using `par_iter()` here for parallel processing, 
since we're iterating over 1000+ items. See homebrew_fast.rs:70 
for reference.
```

❌ Bad:
```
This is slow.
```

---

## Project-Specific Guidelines

### Performance Considerations

**Always consider performance**:
- Use async for I/O operations
- Use Rayon for CPU-bound parallelism
- Cache expensive operations
- Batch API requests

**Reference**: [`docs/PERFORMANCE.md`](docs/PERFORMANCE.md) for optimization strategies

### State Management

**Follow patterns**:
- Use `Arc<RwLock<T>>` for shared async state
- Use `Arc<AtomicBool>` for simple flags
- Clone `Arc` (cheap) when spawning tasks

**Reference**: [`docs/ARCHITECTURE.md#state-management`](docs/ARCHITECTURE.md#state-management)

### Error Handling

**Best practices**:
- Return `Result<T>` for fallible functions
- Provide context with `.map_err()`
- Handle errors gracefully (don't panic)
- Log errors with `println!("[ERROR] ...")`

**Reference**: [`docs/ARCHITECTURE.md#error-handling`](docs/ARCHITECTURE.md#error-handling)

---

## Questions?

**Documentation**:
- [`README.md`](README.md) - Project overview
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - System design
- [`docs/API.md`](docs/API.md) - API reference
- [`docs/SETUP.md`](docs/SETUP.md) - Installation guide
- [`docs/worklog.md`](docs/worklog.md) - Development history

**Getting Help**:
- Open an issue for questions
- Check existing issues and PRs
- Review code for examples

---

**Thank you for contributing!** 🎉 Your help makes DepMgr better for everyone.

