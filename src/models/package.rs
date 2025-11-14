use serde::{Deserialize, Serialize};
use std::fmt;

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

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Homebrew => "Homebrew",
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Cargo => "Cargo",
            PackageManager::Pip => "pip",
            PackageManager::Pipx => "pipx",
            PackageManager::Gem => "gem",
            PackageManager::Go => "Go",
            PackageManager::Composer => "Composer",
            PackageManager::Pub => "pub",
            PackageManager::Swift => "Swift",
        }
    }

    pub fn command(&self) -> &'static str {
        match self {
            PackageManager::Homebrew => "brew",
            PackageManager::Npm => "npm",
            PackageManager::Yarn => "yarn",
            PackageManager::Pnpm => "pnpm",
            PackageManager::Cargo => "cargo",
            PackageManager::Pip => "pip",
            PackageManager::Pipx => "pipx",
            PackageManager::Gem => "gem",
            PackageManager::Go => "go",
            PackageManager::Composer => "composer",
            PackageManager::Pub => "pub",
            PackageManager::Swift => "swift",
        }
    }
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub manager: PackageManager,
    pub installed_version: String,
    pub latest_version: Option<String>,
    pub is_outdated: bool,
    pub size: Option<u64>, // disk space in bytes
    pub description: Option<String>, // what the package does
    pub used_in: Vec<String>, // directories/projects using this package
}

// Removed unused helper methods - dead code cleanup

