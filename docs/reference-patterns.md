# Reference Patterns from klippy & dev-ports-tray

## Project Overviews

### klippy
- **Purpose**: macOS clipboard manager (menu bar app)
- **Structure**: Modular (`main.rs`, `clipboard.rs`, `storage.rs`)
- **Key Libraries**: `arboard`, `tray-icon`, `winit`, `global-hotkey`

### dev-ports-tray
- **Purpose**: Monitor and kill dev server processes (menu bar app)
- **Structure**: Single `main.rs` (more monolithic)
- **Key Libraries**: `anyhow`, `crossbeam-channel`, `nix`, `winit`, `tray-icon`

---

## Key Patterns & Learnings

### 1. Project Structure

**klippy** (Modular approach):
```
src/
├── main.rs          # Entry point, UI, event handling
├── clipboard.rs     # Clipboard operations
└── storage.rs       # Data storage logic
```

**dev-ports-tray** (Monolithic):
```
src/
└── main.rs          # Everything in one file
```

**For our project**: Prefer modular approach like klippy for better organization.

---

### 2. Command Execution Pattern

**dev-ports-tray** shows excellent pattern for executing shell commands:

```rust
use std::process::{Command, Stdio};

fn run_command_with_timeout(
    cmd: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<std::process::Output> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    
    // Poll with timeout
    loop {
        match child.try_wait() {
            Ok(Some(_status)) => return Ok(child.wait_with_output()?),
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    return Err(anyhow!("Command timed out"));
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(anyhow!("Error: {}", e)),
        }
    }
}
```

**Key takeaways**:
- Always use timeouts for external commands
- Capture stdout/stderr with `Stdio::piped()`
- Poll with `try_wait()` instead of blocking `wait()`
- Kill processes that timeout

**For our project**: Use this pattern for executing `brew`, `npm`, `cargo`, etc.

---

### 3. Error Handling

**dev-ports-tray** uses `anyhow`:
```rust
use anyhow::{anyhow, Result};

fn some_function() -> Result<()> {
    // Easy error propagation
    let output = run_command(...)?;
    Ok(())
}
```

**For our project**: Use `anyhow` for flexible error handling, `thiserror` for custom error types.

---

### 4. Threading & Async Patterns

**Both projects** use similar patterns:

**klippy**:
```rust
// Spawn background thread
thread::spawn(move || {
    loop {
        // Do work
        if changed {
            proxy.send_event(AppEvent::UpdateMenu);
        }
        thread::sleep(Duration::from_millis(500));
    }
});
```

**dev-ports-tray** (more sophisticated):
```rust
use crossbeam_channel as channel;

let (tx, rx) = channel::unbounded::<MonitorCmd>();

// Background thread with timeout-based select
thread::spawn(move || {
    loop {
        match rx.recv_timeout(remaining) {
            Ok(MonitorCmd::RescanNow) => perform_scan(),
            Ok(MonitorCmd::Shutdown) => break,
            Err(channel::RecvTimeoutError::Timeout) => perform_scan(),
            Err(channel::RecvTimeoutError::Disconnected) => break,
        }
    }
});
```

**For our project**: 
- Use `crossbeam-channel` for thread communication
- Use `tokio` for async operations (better for concurrent package manager queries)
- Consider async/await instead of threads for I/O-bound operations

---

### 5. Shared State Pattern

**Both projects** use `Arc<Mutex<>>`:

```rust
let storage = Arc::new(Mutex::new(ClipboardStorage::new()));
let storage_clone = Arc::clone(&storage);

thread::spawn(move || {
    if let Ok(mut storage) = storage_clone.lock() {
        // Modify storage
    }
});
```

**For our project**: Use `Arc<Mutex<>>` for shared state, or consider `Arc<RwLock<>>` if we have more reads than writes.

---

### 6. Event Loop Pattern

**Both projects** use `winit` event loop with custom events:

```rust
use winit::event_loop::{EventLoopBuilder, ControlFlow};

#[derive(Debug, Clone)]
enum AppEvent {
    UpdateSnapshot(Arc<ProcessSnapshot>),
    MenuAction,
    Quit,
}

let event_loop = EventLoopBuilder::with_user_event().build()?;
let proxy = event_loop.create_proxy();

// Send events from threads
proxy.send_event(AppEvent::UpdateSnapshot(snap))?;

// Handle events in main loop
event_loop.run(move |event, elwt| {
    elwt.set_control_flow(ControlFlow::Wait);
    match event {
        Event::UserEvent(AppEvent::UpdateSnapshot(snap)) => {
            // Update UI
        }
        _ => {}
    }
});
```

**For our project**: 
- If we build a TUI, we might use `ratatui` instead
- For CLI-only, we don't need winit
- But the event pattern is useful for async operations

---

### 7. Release Profile Optimizations

**Both projects** use aggressive optimizations:

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

**For our project**: Use same optimizations for production builds.

---

### 8. Parsing Command Output

**dev-ports-tray** shows good pattern for parsing `lsof`:

```rust
let stdout = String::from_utf8_lossy(&output.stdout);

for line in stdout.lines().skip(1) {  // Skip header
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 9 {
        continue;  // Skip malformed lines
    }
    
    // Parse specific columns
    let pid = i32::from_str(parts[1])?;
    // ...
}
```

**For our project**: 
- Parse JSON when available (`brew list --json`, `npm list --json`)
- Parse text output when JSON isn't available (`cargo install --list`)
- Handle malformed output gracefully

---

### 9. Menu State Management

**klippy** shows clean menu state pattern:

```rust
#[derive(Default)]
struct MenuState {
    clear_all: MenuId,
    quit: MenuId,
    item_ids: HashMap<MenuId, usize>,
}

let menu_state = Arc::new(Mutex::new(MenuState::default()));

// Store IDs when building menu
state.item_ids.insert(menu_item.id().clone(), index);

// Lookup IDs when handling events
if let Some(&index) = state.item_ids.get(&event.id) {
    // Handle action
}
```

**For our project**: If we build a TUI, similar state management patterns apply.

---

### 10. Process Management

**dev-ports-tray** shows good process termination:

```rust
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

fn terminate_process(pid: i32) -> Result<()> {
    let npid = Pid::from_raw(pid);
    
    // Try SIGTERM first
    kill(npid, Signal::SIGTERM)?;
    
    // Wait for graceful exit
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        match kill(npid, None) {
            Ok(_) => thread::sleep(Duration::from_millis(100)),
            Err(nix::errno::Errno::ESRCH) => return Ok(()), // Already dead
            Err(e) => return Err(anyhow!("Error: {}", e)),
        }
    }
    
    // Force kill if still alive
    kill(npid, Signal::SIGKILL)?;
    Ok(())
}
```

**For our project**: We might need this for killing processes during updates, or for cleaning up.

---

## Recommendations for Our Project

### Architecture Decisions

1. **CLI vs TUI**: Start with CLI (simpler), add TUI later if needed
2. **Async vs Threads**: Use `tokio` for async I/O (better for concurrent package manager queries)
3. **Error Handling**: `anyhow` for flexibility, `thiserror` for custom errors
4. **Command Execution**: Use timeout pattern from dev-ports-tray
5. **Parsing**: Prefer JSON when available, fallback to text parsing
6. **State Management**: Use `Arc<Mutex<>>` or `Arc<RwLock<>>` for shared state

### Dependencies to Consider

```toml
[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Async
tokio = { version = "1", features = ["full"] }

# Error handling
anyhow = "1"
thiserror = "1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# File operations
walkdir = "2"
glob = "0.3"

# Utilities
regex = "1"
chrono = "0.4"
indicatif = "0.17"  # Progress bars
colored = "2"

# Optional: TUI
ratatui = "0.26"
crossterm = "0.27"
```

### Code Organization

```
src/
├── main.rs              # CLI entry point
├── cli.rs               # Argument parsing
├── managers/            # Package manager implementations
│   ├── mod.rs
│   ├── homebrew.rs
│   ├── npm.rs
│   └── ...
├── scanner/             # Project scanning
│   ├── mod.rs
│   ├── detector.rs
│   └── parser.rs
├── models/              # Data models
│   ├── mod.rs
│   ├── package.rs
│   └── project.rs
├── storage/             # Caching
│   ├── mod.rs
│   └── cache.rs
└── utils/               # Utilities
    ├── mod.rs
    ├── command.rs       # Command execution with timeout
    └── format.rs
```

---

## Key Takeaways

1. **Keep it simple**: Both projects are focused and do one thing well
2. **Handle errors gracefully**: Use `anyhow` for easy error propagation
3. **Use timeouts**: Always timeout external commands
4. **Parse robustly**: Handle malformed output gracefully
5. **Optimize for release**: Use aggressive optimizations
6. **Modular structure**: Separate concerns into modules
7. **Async for I/O**: Use `tokio` for concurrent operations

