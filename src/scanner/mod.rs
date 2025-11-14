pub mod project_scanner;

pub use project_scanner::{scan_homebrew_tool_usage, get_scan_directories};

// Removed scan_package_usage - unused dead code. Using scan_homebrew_tool_usage instead.
