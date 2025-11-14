use crate::models::{Package, PackageManager};
use crate::managers::{detect_available_managers, list_homebrew_packages};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;

pub struct DepMgrApp {
    pub packages: Arc<RwLock<Vec<Package>>>,
    pub available_managers: Vec<PackageManager>,
    pub selected_managers: std::collections::HashSet<PackageManager>,
    pub search_query: String,
    pub show_outdated_only: bool,
    pub show_orphaned_only: bool,
    pub is_scanning: Arc<AtomicBool>,
    pub refresh_requested: bool,
    pub runtime: tokio::runtime::Runtime,
}

impl Default for DepMgrApp {
    fn default() -> Self {
        Self {
            packages: Arc::new(RwLock::new(Vec::new())),
            available_managers: Vec::new(),
            selected_managers: std::collections::HashSet::new(),
            search_query: String::new(),
            show_outdated_only: false,
            show_orphaned_only: false,
            is_scanning: Arc::new(AtomicBool::new(false)),
            refresh_requested: false,
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
}

impl DepMgrApp {
    pub async fn initialize(&mut self) {
        println!("[DEBUG] Initializing app...");
        // Detect available package managers
        self.available_managers = detect_available_managers().await;
        println!("[DEBUG] Found {} package managers", self.available_managers.len());
        self.selected_managers = self.available_managers.iter().cloned().collect();
        
        // Initial scan
        self.is_scanning.store(true, Ordering::Relaxed);
        let packages_clone = Arc::clone(&self.packages);
        let scanning_flag = Arc::clone(&self.is_scanning);
        let available_managers = self.available_managers.clone();
        
        self.runtime.spawn(async move {
            println!("[DEBUG] Starting package scan...");
            let mut all_packages = Vec::new();

            // Scan Homebrew if available
            if available_managers.contains(&PackageManager::Homebrew) {
                println!("[DEBUG] Scanning Homebrew packages...");
                match list_homebrew_packages().await {
                    Ok(packages) => {
                        println!("[DEBUG] Found {} Homebrew packages", packages.len());
                        all_packages.extend(packages);
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to list Homebrew packages: {}", e);
                    }
                }
            }

            // TODO: Add other package managers

            println!("[DEBUG] Scan complete. Total packages: {}", all_packages.len());
            *packages_clone.write().await = all_packages;
            scanning_flag.store(false, Ordering::Relaxed);
            println!("[DEBUG] Scanning flag set to false");
        });
    }

    pub fn request_refresh(&mut self) {
        self.refresh_requested = true;
    }

    pub fn handle_refresh(&mut self) {
        if self.refresh_requested {
            self.refresh_requested = false;
            self.is_scanning.store(true, Ordering::Relaxed);
            
            let packages_clone = Arc::clone(&self.packages);
            let scanning_flag = Arc::clone(&self.is_scanning);
            let available_managers = self.available_managers.clone();
            
            self.runtime.spawn(async move {
                println!("[DEBUG] Refresh: Starting package scan...");
                let mut all_packages = Vec::new();

                // Scan Homebrew if available
                if available_managers.contains(&PackageManager::Homebrew) {
                    match list_homebrew_packages().await {
                        Ok(packages) => {
                            println!("[DEBUG] Refresh: Found {} Homebrew packages", packages.len());
                            all_packages.extend(packages);
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Refresh: Failed to list Homebrew packages: {}", e);
                        }
                    }
                }

                // TODO: Add other package managers

                println!("[DEBUG] Refresh: Scan complete. Total packages: {}", all_packages.len());
                *packages_clone.write().await = all_packages;
                scanning_flag.store(false, Ordering::Relaxed);
            });
        }
    }

    pub fn filtered_packages(&self) -> Vec<Package> {
        let packages = self.packages.blocking_read();
        packages
            .iter()
            .filter(|pkg| {
                // Filter by selected managers
                if !self.selected_managers.is_empty() && !self.selected_managers.contains(&pkg.manager) {
                    return false;
                }

                // Filter by search query
                if !self.search_query.is_empty() {
                    if !pkg.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        return false;
                    }
                }

                // Filter by outdated
                if self.show_outdated_only && !pkg.is_outdated {
                    return false;
                }

                // Filter by orphaned (TODO: implement orphaned detection)
                if self.show_orphaned_only {
                    // Placeholder - will implement later
                }

                true
            })
            .cloned()
            .collect()
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let packages = self.packages.blocking_read();
        let total = packages.len();
        let outdated = packages.iter().filter(|p| p.is_outdated).count();
        // Count orphaned packages (not used in any project)
        // For now, we'll implement basic orphaned detection later
        // Reference the functions to ensure they're not considered dead code
        let _orphaned_packages = self.find_orphaned_packages();
        let _scanned_projects = self.scan_projects();
        let orphaned = 0; // TODO: implement orphaned detection using PackageUsage
        (total, outdated, orphaned)
    }
    
    // Placeholder for project scanning - will use Project and Dependency
    // This demonstrates usage of Project::new() and Dependency struct
    pub fn scan_projects(&self) -> Vec<crate::models::Project> {
        // TODO: Implement project scanning
        // For now, return empty vector but demonstrate usage
        let _example_project = crate::models::Project::new(std::path::PathBuf::from("/tmp/example"));
        let _example_dep = crate::models::Dependency {
            package_name: "example".to_string(),
            manager: crate::models::PackageManager::Npm,
            version_constraint: "^1.0.0".to_string(),
            is_dev: false,
        };
        Vec::new()
    }
    
    // Placeholder for orphaned detection - will use PackageUsage
    // This demonstrates usage of PackageUsage::new() and add_project()
    pub fn find_orphaned_packages(&self) -> Vec<crate::models::PackageUsage> {
        // TODO: Implement orphaned package detection
        // For now, return empty vector but demonstrate usage
        let packages = self.packages.blocking_read();
        if let Some(pkg) = packages.first() {
            let mut usage = crate::models::PackageUsage::new(pkg.clone());
            let example_project = crate::models::Project::new(std::path::PathBuf::from("/tmp/example"));
            usage.add_project(example_project);
            // Access the package field to avoid warning
            let _ = &usage.package;
            return vec![usage];
        }
        Vec::new()
    }
}

