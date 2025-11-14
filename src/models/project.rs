use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::Dependency;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
    pub package_managers: Vec<crate::models::PackageManager>,
    pub dependencies: Vec<Dependency>,
    pub last_modified: DateTime<Utc>,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Self {
            path,
            name,
            package_managers: Vec::new(),
            dependencies: Vec::new(),
            last_modified: Utc::now(),
        }
    }
}
