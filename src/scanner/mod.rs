pub mod project_scanner;

pub use project_scanner::{scan_homebrew_tool_usage, get_scan_directories};

use crate::models::Package;
use std::path::PathBuf;
use walkdir::WalkDir;

// Scan common development directories for project-level package usage (npm, cargo, etc.)
pub fn scan_package_usage(packages: &mut [Package], scan_dirs: &[PathBuf]) {
    println!("[DEBUG] Scanning for package usage in {} directories...", scan_dirs.len());
    
    for package in packages.iter_mut() {
        let mut found_in = Vec::new();
        
        for base_dir in scan_dirs {
            if !base_dir.exists() {
                continue;
            }
            
            // Walk through directories up to 3 levels deep
            for entry in WalkDir::new(base_dir)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                
                // Check for project indicator files
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    
                    // Check package.json for npm packages
                    if file_name == "package.json" && package.manager == crate::models::PackageManager::Npm {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains(&format!("\"{}\"", package.name)) {
                                if let Some(parent) = path.parent() {
                                    found_in.push(parent.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    
                    // Check Cargo.toml for cargo packages
                    if file_name == "Cargo.toml" && package.manager == crate::models::PackageManager::Cargo {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains(&package.name) {
                                if let Some(parent) = path.parent() {
                                    found_in.push(parent.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    
                    // Check requirements.txt for pip packages
                    if (file_name == "requirements.txt" || file_name == "Pipfile") 
                        && package.manager == crate::models::PackageManager::Pip {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains(&package.name) {
                                if let Some(parent) = path.parent() {
                                    found_in.push(parent.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    
                    // Check Gemfile for gem packages
                    if file_name == "Gemfile" && package.manager == crate::models::PackageManager::Gem {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            if content.contains(&package.name) {
                                if let Some(parent) = path.parent() {
                                    found_in.push(parent.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Deduplicate and sort
        found_in.sort();
        found_in.dedup();
        package.used_in = found_in;
    }
    
    let used_count = packages.iter().filter(|p| !p.used_in.is_empty()).count();
    let unused_count = packages.len() - used_count;
    println!("[DEBUG] Found {} packages in use, {} unused", used_count, unused_count);
}
