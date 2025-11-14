use serde::{Deserialize, Serialize};

use super::{Package, Project};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub package_name: String,
    pub manager: crate::models::PackageManager,
    pub version_constraint: String, // e.g., "^1.2.3", ">=2.0.0"
    pub is_dev: bool,
}

#[derive(Debug, Clone)]
pub struct PackageUsage {
    pub package: Package,
    pub used_in_projects: Vec<Project>,
    pub is_orphaned: bool,
}

impl PackageUsage {
    pub fn new(package: Package) -> Self {
        Self {
            package,
            used_in_projects: Vec::new(),
            is_orphaned: true,
        }
    }

    pub fn add_project(&mut self, project: Project) {
        self.used_in_projects.push(project);
        self.is_orphaned = false;
    }
}

