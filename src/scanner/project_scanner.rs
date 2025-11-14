use crate::models::Package;
use std::path::PathBuf;
use walkdir::WalkDir;
use std::collections::HashMap;

// Scan projects and determine which Homebrew tools they actually use
pub fn scan_homebrew_tool_usage(packages: &mut [Package], scan_dirs: &[PathBuf]) {
    println!("[DEBUG] Scanning projects for Homebrew tool usage...");
    
    // Build a map of tool name -> projects using it
    let mut tool_usage: HashMap<String, Vec<String>> = HashMap::new();
    
    for base_dir in scan_dirs {
        if !base_dir.exists() {
            continue;
        }
        
        println!("[DEBUG] Scanning directory: {}", base_dir.display());
        
        // Walk through directories to find projects
        for entry in WalkDir::new(base_dir)
            .max_depth(4)
            .into_iter()
            .filter_entry(|e| {
                // Skip common directories we don't care about
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && 
                name != "node_modules" && 
                name != "target" && 
                name != "dist" &&
                name != "build" &&
                name != "__pycache__"
            })
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if !path.is_dir() {
                continue;
            }
            
            // Check for project indicator files and infer tool usage
            let project_path = path.to_string_lossy().to_string();
            
            // Node.js projects
            if path.join("package.json").exists() {
                tool_usage.entry("node".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("npm".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Rust projects
            if path.join("Cargo.toml").exists() {
                tool_usage.entry("rust".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("cargo".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Python projects
            if path.join("requirements.txt").exists() || 
               path.join("setup.py").exists() ||
               path.join("pyproject.toml").exists() ||
               path.join("Pipfile").exists() {
                tool_usage.entry("python".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("python3".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("pip".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Ruby projects
            if path.join("Gemfile").exists() {
                tool_usage.entry("ruby".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("gem".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("bundle".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Go projects
            if path.join("go.mod").exists() {
                tool_usage.entry("go".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Java projects
            if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
                tool_usage.entry("java".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("maven".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("gradle".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Git repositories
            if path.join(".git").exists() {
                tool_usage.entry("git".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Docker projects
            if path.join("Dockerfile").exists() || path.join("docker-compose.yml").exists() {
                tool_usage.entry("docker".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
                tool_usage.entry("docker-compose".to_string())
                    .or_insert_with(Vec::new)
                    .push(project_path.clone());
            }
            
            // Database tools - check for config files
            if path.join("package.json").exists() {
                // Read package.json to check for database dependencies
                if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
                    if content.contains("postgres") || content.contains("pg") {
                        tool_usage.entry("postgresql".to_string())
                            .or_insert_with(Vec::new)
                            .push(project_path.clone());
                    }
                    if content.contains("redis") {
                        tool_usage.entry("redis".to_string())
                            .or_insert_with(Vec::new)
                            .push(project_path.clone());
                    }
                    if content.contains("mongodb") || content.contains("mongoose") {
                        tool_usage.entry("mongodb".to_string())
                            .or_insert_with(Vec::new)
                            .push(project_path.clone());
                    }
                }
            }
        }
    }
    
    // Deduplicate project paths
    for projects in tool_usage.values_mut() {
        projects.sort();
        projects.dedup();
    }
    
    // Update packages with usage information
    for pkg in packages.iter_mut() {
        pkg.used_in.clear(); // Clear "System Tool" marker
        
        if let Some(projects) = tool_usage.get(&pkg.name) {
            pkg.used_in = projects.clone();
        }
    }
    
    let used_count = packages.iter().filter(|p| !p.used_in.is_empty()).count();
    let unused_count = packages.len() - used_count;
    
    println!("[DEBUG] Found {} tools used in projects, {} unused", used_count, unused_count);
    
    // Show some examples
    for pkg in packages.iter().take(5) {
        if !pkg.used_in.is_empty() {
            println!("[DEBUG] {} used in {} projects", pkg.name, pkg.used_in.len());
        }
    }
}

// Get common development directories to scan
pub fn get_scan_directories() -> Vec<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let home_path = PathBuf::from(home);
    
    vec![
        home_path.join("Desktop"),
        home_path.join("Documents"),
        home_path.join("projects"),
        home_path.join("dev"),
        home_path.join("Developer"),
        home_path.join("code"),
        home_path.join("workspace"),
    ]
}

