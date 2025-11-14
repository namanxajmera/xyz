# Contributing

This is mostly for AI coding assistants. If you're a human and want to contribute, cool, but no formal process here.

## Setup

```bash
# Clone and build
git clone <repo>
cd xyz
cargo build

# Run it
cargo run
```

**Tools you might want**:
```bash
rustup component add rustfmt clippy
cargo install cargo-watch  # auto-reload on changes
```

## Code Style

Just run these before committing:

```bash
cargo fmt          # format code
cargo clippy --fix # fix obvious issues
```

**Rust conventions**:
- `snake_case` for functions/variables
- `PascalCase` for types/structs
- `SCREAMING_SNAKE_CASE` for constants

**Project patterns**:

Error handling:
```rust
use anyhow::{anyhow, Result};

pub async fn do_thing() -> Result<Data> {
    something().await
        .map_err(|e| anyhow!("Failed because: {}", e))?;
}
```

Logging:
```rust
println!("[DEBUG] Processing {} items", count);
println!("[ERROR] Failed to fetch: {}", error);
```

Async operations:
```rust
// Spawn background tasks
self.runtime.spawn(async move {
    let result = expensive_operation().await;
    *shared_state.write().await = result;
});
```

## Adding a Package Manager

If you want to add support for yarn/pnpm/gem/whatever:

**1. Create the module** (`src/managers/yarn.rs`):
```rust
use crate::models::{Package, PackageManager};
use anyhow::Result;

pub async fn list_yarn_packages() -> Result<Vec<Package>> {
    // Run: yarn global list --json
    // Parse output
    // Return packages
}

pub async fn check_outdated_yarn(packages: &mut [Package]) -> Result<()> {
    // Run: yarn global outdated
    // Update is_outdated field
}

pub async fn update_yarn_package(name: String) -> Result<()> {
    // Run: yarn global upgrade <name>
}

pub async fn install_yarn_package(name: String) -> Result<()> {
    // Run: yarn global add <name>
}

pub async fn uninstall_yarn_package(name: String) -> Result<()> {
    // Run: yarn global remove <name>
}

pub async fn add_yarn_descriptions(packages: Arc<RwLock<Vec<Package>>>) {
    // Fetch descriptions in parallel (8 concurrent)
    // Update UI incrementally
}
```

**2. Add to enum** (`src/models/package.rs`):
```rust
pub enum PackageManager {
    // ...
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

**3. Add to detector** (`src/managers/detector.rs`):
```rust
if command_exists("yarn").await {
    managers.push(PackageManager::Yarn);
}
```

**4. Add to scan** (`src/app.rs` in `start_scan()`):
```rust
if available_managers.contains(&PackageManager::Yarn) {
    match crate::managers::yarn::list_yarn_packages().await {
        Ok(mut packages) => {
            let _ = crate::managers::yarn::check_outdated_yarn(&mut packages).await;
            let mut all_packages = packages_clone.write().await;
            all_packages.extend(packages);
            
            // Fetch descriptions in background
            let packages_arc = Arc::clone(&packages_clone);
            tokio::spawn(async move {
                crate::managers::yarn::add_yarn_descriptions(packages_arc).await;
            });
        }
        Err(e) => eprintln!("[ERROR] Failed to list yarn packages: {}", e),
    }
}
```

**5. Add to operations** (`src/app.rs`):
```rust
// In update_package():
let result = match manager {
    // ...
    PackageManager::Yarn => 
        crate::managers::yarn::update_yarn_package(package_name.clone()).await,
};

// In reinstall_package():
PackageManager::Yarn => 
    crate::managers::yarn::install_yarn_package(pkg_name).await,

// In uninstall_package():
PackageManager::Yarn => 
    crate::managers::yarn::uninstall_yarn_package(pkg_name).await,
```

Look at `src/managers/npm.rs` or `src/managers/cargo.rs` for working examples.

## Documentation

Update these if you change stuff:
- `README.md` - if adding features
- `docs/ARCHITECTURE.md` - if changing design
- `docs/API.md` - if changing APIs
- `docs/worklog.md` - add entry for what you did

## Testing

No formal tests yet. Just:
1. Run the app: `cargo run`
2. Check it works
3. Make sure you didn't break existing stuff

## Common Commands

```bash
cargo check              # type checking (fast)
cargo clippy            # linting
cargo fmt               # formatting
cargo build --release   # optimized build
cargo run               # build and run
cargo watch -x run      # auto-reload on changes
```

## Performance

If you're optimizing, see `docs/PERFORMANCE.md` for techniques. Key points:
- Use HTTP APIs instead of CLI spawning (100-500x faster)
- Use Rayon for CPU-bound parallel work
- Use Tokio for I/O-bound async work
- Cache aggressively

## Questions?

- Read the code (it's well-documented)
- Check `docs/ARCHITECTURE.md` for system design
- Check `docs/worklog.md` for development history
- Check `docs/TROUBLESHOOTING.md` if something breaks
