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

impl Package {
    pub fn new(name: String, manager: PackageManager, installed_version: String) -> Self {
        Self {
            name,
            manager,
            installed_version,
            latest_version: None,
            is_outdated: false,
            size: None,
            description: None,
            used_in: Vec::new(),
        }
    }

    pub fn with_latest_version(mut self, latest_version: String) -> Self {
        self.latest_version = Some(latest_version.clone());
        self.is_outdated = self.installed_version != latest_version;
        self
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn is_unused(&self) -> bool {
        self.used_in.is_empty()
    }
}

