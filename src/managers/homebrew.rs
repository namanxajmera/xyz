use anyhow::{anyhow, Result};
use crate::models::{Package, PackageManager};
use crate::utils::run_command_with_timeout;
use serde_json::Value;
use std::time::Duration;

pub async fn list_homebrew_packages() -> Result<Vec<Package>> {
    println!("[DEBUG] Getting Homebrew package list with versions...");
    
    // Use 'brew list --versions' which is MUCH faster than 'brew info --json'
    // This gives us both names and versions in a simple text format: "package 1.2.3"
    let output = run_command_with_timeout(
        "brew",
        &["list", "--versions"],
        Duration::from_secs(15),
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("brew list --versions failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();
    
    // Parse format: "package-name 1.2.3" or "package-name 1.2.3 4.5.6" (multiple versions)
    for line in stdout.lines() {
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0];
            // Use the first version if multiple are installed
            let version = parts[1];
            
            packages.push(Package::new(
                name.to_string(),
                PackageManager::Homebrew,
                version.to_string(),
            ));
        } else if !parts.is_empty() {
            // Package without version info
            packages.push(Package::new(
                parts[0].to_string(),
                PackageManager::Homebrew,
                "unknown".to_string(),
            ));
        }
    }
    
    println!("[DEBUG] Found {} Homebrew packages", packages.len());
    Ok(packages)
}

// Get description for a single package
async fn get_package_description(package_name: String) -> Option<(String, String)> {
    match run_command_with_timeout(
        "brew",
        &["info", "--json=v2", &package_name],
        Duration::from_secs(10), // Increased from 5s to 10s
    )
    .await {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("[WARN] brew info failed for {}: exit code {}", 
                         package_name, output.status.code().unwrap_or(-1));
                return None;
            }
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            match serde_json::from_str::<Value>(&stdout) {
                Ok(json) => {
                    // Try formulae first
                    if let Some(formulae) = json.get("formulae").and_then(|v| v.as_array()) {
                        if let Some(formula) = formulae.first() {
                            if let Some(desc) = formula.get("desc").and_then(|v| v.as_str()) {
                                return Some((package_name, desc.to_string()));
                            }
                        }
                    }
                    
                    // Try casks
                    if let Some(casks) = json.get("casks").and_then(|v| v.as_array()) {
                        if let Some(cask) = casks.first() {
                            if let Some(desc) = cask.get("desc").and_then(|v| v.as_str()) {
                                return Some((package_name, desc.to_string()));
                            }
                        }
                    }
                    
                    eprintln!("[WARN] No description found in JSON for {}", package_name);
                }
                Err(e) => {
                    eprintln!("[WARN] Failed to parse JSON for {}: {}", package_name, e);
                }
            }
        }
        Err(e) => {
            eprintln!("[WARN] Command failed for {}: {}", package_name, e);
        }
    }
    
    None
}

// Get package descriptions with limited concurrency to prevent overwhelming Homebrew
pub async fn add_package_descriptions_parallel(
    packages: Vec<Package>,
    packages_clone: std::sync::Arc<tokio::sync::RwLock<Vec<Package>>>,
) {
    use futures::{stream, StreamExt};
    
    println!("[DEBUG] Getting package descriptions (max 8 concurrent)...");
    let package_names: Vec<String> = packages.iter().map(|p| p.name.clone()).collect();
    let total = package_names.len();
    
    const CONCURRENT_REQUESTS: usize = 8; // Limit to 8 concurrent brew commands
    
    let mut completed = 0;
    let mut failed = 0;
    
    // Create a stream that processes requests with limited concurrency
    let mut results = stream::iter(package_names)
        .map(|name| async move {
            let result = get_package_description(name.clone()).await;
            (name, result)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);
    
    // Process results as they come in
    while let Some((name, result)) = results.next().await {
        match result {
            Some((pkg_name, desc)) => {
                completed += 1;
                
                // Update the package immediately in shared state
                let mut pkgs = packages_clone.write().await;
                if let Some(pkg) = pkgs.iter_mut().find(|p| p.name == pkg_name) {
                    pkg.description = Some(desc);
                }
                
                if completed % 10 == 0 || completed == total {
                    println!("[DEBUG] Fetched descriptions: {}/{}", completed, total);
                }
            }
            None => {
                failed += 1;
            }
        }
    }
    
    println!("[DEBUG] All descriptions fetched: {}/{} (failed: {})", completed, total, failed);
}

// Separate function to check for outdated packages
// This can be called after displaying the initial list
pub async fn check_outdated_packages(mut packages: Vec<Package>) -> Result<Vec<Package>> {
    println!("[DEBUG] Checking for outdated packages...");
    
    // Check for outdated packages
    let outdated_output = run_command_with_timeout(
        "brew",
        &["outdated", "--json=v2"],
        Duration::from_secs(10),
    )
    .await;

    if let Ok(outdated_output) = outdated_output {
        if outdated_output.status.success() {
            let outdated_stdout = String::from_utf8_lossy(&outdated_output.stdout);
            if let Ok(outdated_json) = serde_json::from_str::<Value>(&outdated_stdout) {
                let outdated_names: Vec<String> = outdated_json
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|v| v.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect();

                println!("[DEBUG] Found {} outdated packages", outdated_names.len());

                for package in &mut packages {
                    if outdated_names.contains(&package.name) {
                        package.is_outdated = true;
                        // Try to get latest version from brew info
                        if let Ok(latest) = get_latest_version(&package.name).await {
                            *package = package.clone().with_latest_version(latest);
                        }
                    }
                }
            }
        }
    }

    Ok(packages)
}

async fn get_latest_version(package_name: &str) -> Result<String> {
    let output = run_command_with_timeout(
        "brew",
        &["info", "--json=v2", package_name],
        Duration::from_secs(5),
    )
    .await?;

    if !output.status.success() {
        return Err(anyhow!("brew info failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Value = serde_json::from_str(&stdout)?;

    // Try to get version from formulae or casks
    if let Some(formulae) = json.get("formulae").and_then(|v| v.as_array()) {
        if let Some(formula) = formulae.first() {
            if let Some(versions) = formula.get("versions").and_then(|v| v.as_object()) {
                if let Some(stable) = versions.get("stable").and_then(|v| v.as_str()) {
                    return Ok(stable.to_string());
                }
            }
        }
    }

    if let Some(casks) = json.get("casks").and_then(|v| v.as_array()) {
        if let Some(cask) = casks.first() {
            if let Some(version) = cask.get("version").and_then(|v| v.as_str()) {
                return Ok(version.to_string());
            }
        }
    }

    Err(anyhow!("Could not determine latest version"))
}

// Update a single package
pub async fn update_package(package_name: &str) -> Result<String> {
    println!("[DEBUG] Updating package: {}", package_name);
    
    let output = run_command_with_timeout(
        "brew",
        &["upgrade", package_name],
        Duration::from_secs(300), // 5 minutes for updates
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("brew upgrade failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

// Update all outdated packages
pub async fn update_all_packages() -> Result<String> {
    println!("[DEBUG] Updating all outdated packages...");
    
    let output = run_command_with_timeout(
        "brew",
        &["upgrade"],
        Duration::from_secs(600), // 10 minutes for bulk updates
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("brew upgrade all failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

