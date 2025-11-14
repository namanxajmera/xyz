pub mod cargo;
pub mod detector;
pub mod homebrew_fast;
pub mod npm;
pub mod pip;

pub use detector::detect_available_managers;
