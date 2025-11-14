pub mod detector;
pub mod homebrew;

pub use detector::detect_available_managers;
pub use homebrew::list_homebrew_packages;

