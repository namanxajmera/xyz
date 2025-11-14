pub mod project_scanner;

pub use project_scanner::{get_scan_directories, scan_homebrew_tool_usage};

// Removed scan_package_usage - unused dead code. Using scan_homebrew_tool_usage instead.
