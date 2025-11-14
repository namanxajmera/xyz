use anyhow::{anyhow, Result};
use crate::models::{Package, PackageManager};
use crate::utils::http_client::create_http_client;
use crate::utils::cache::{get_cached, set_cached};
use serde::{Deserialize, Serialize};
use rayon::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
struct FormulaInfo {
    name: String,
    #[serde(default)]
    desc: Option<String>,
    versions: Versions,
}

#[derive(Debug, Deserialize, Serialize)]
struct Versions {
    stable: Option<String>,
}

/// BLAZINGLY FAST: Fetch ALL Homebrew packages in ONE API call
pub async fn list_homebrew_packages_fast() -> Result<Vec<Package>> {
    println!("[FAST] Fetching Homebrew packages via API...");
    
    // Check cache first (1 hour TTL)
    if let Some(cached_packages) = get_cached::<Vec<Package>>("homebrew_all_packages") {
        println!("[FAST] ✓ Loaded {} packages from cache (instant!)", cached_packages.len());
        return Ok(cached_packages);
    }
    
    let client = create_http_client();
    
    // Fetch ALL formulas in ONE request
    let url = "https://formulae.brew.sh/api/formula.json";
    let start = std::time::Instant::now();
    
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to fetch Homebrew API: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Homebrew API returned status: {}", response.status()));
    }
    
    let formulas: Vec<FormulaInfo> = response.json()
        .await
        .map_err(|e| anyhow!("Failed to parse Homebrew API response: {}", e))?;
    
    let fetch_time = start.elapsed();
    println!("[FAST] ✓ Fetched {} formulas in {:?}", formulas.len(), fetch_time);
    
    // Get locally installed packages (fast CLI command)
    let installed = get_installed_packages().await?;
    
    // Parallel parse: Filter to only installed packages
    let start_parse = std::time::Instant::now();
    let packages: Vec<Package> = formulas
        .par_iter()  // Rayon parallel iterator
        .filter_map(|formula| {
            // Only include if it's installed locally
            installed.get(&formula.name).map(|local_version| {
                Package {
                    name: formula.name.clone(),
                    manager: PackageManager::Homebrew,
                    installed_version: local_version.clone(),
                    latest_version: formula.versions.stable.clone(),
                    is_outdated: false, // Will check later
                    description: formula.desc.clone(),
                    used_in: vec![], // Will scan later
                    size: None, // Not available from API
                }
            })
        })
        .collect();
    
    let parse_time = start_parse.elapsed();
    println!("[FAST] ✓ Parsed {} installed packages in {:?}", packages.len(), parse_time);
    
    // Cache for 1 hour
    set_cached("homebrew_all_packages".to_string(), &packages, 3600);
    
    println!("[FAST] 🚀 Total time: {:?} (vs 5-7 minutes with old method!)", 
             fetch_time + parse_time);
    
    Ok(packages)
}

/// Fast: Get locally installed package names and versions
async fn get_installed_packages() -> Result<std::collections::HashMap<String, String>> {
    use crate::utils::run_command_with_timeout;
    use std::time::Duration;
    
    let output = run_command_with_timeout(
        "brew",
        &["list", "--versions"],
        Duration::from_secs(15),
    )
    .await?;
    
    if !output.status.success() {
        return Err(anyhow!("brew list --versions failed"));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut installed = std::collections::HashMap::new();
    
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            installed.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    
    println!("[FAST] ✓ Found {} locally installed packages", installed.len());
    Ok(installed)
}

/// Fast: Check which packages are outdated using batch API
pub async fn check_outdated_packages_fast(packages: &mut [Package]) -> Result<()> {
    println!("[FAST] Checking for outdated packages...");
    let start = std::time::Instant::now();
    
    // Simple comparison: installed vs latest from API
    let mut outdated_count = 0;
    for pkg in packages.iter_mut() {
        if let (Some(latest), installed) = (&pkg.latest_version, &pkg.installed_version) {
            // Simple version comparison (you can enhance this)
            if latest != installed {
                pkg.is_outdated = true;
                outdated_count += 1;
            }
        }
    }
    
    let elapsed = start.elapsed();
    println!("[FAST] ✓ Found {} outdated packages in {:?}", outdated_count, elapsed);
    
    Ok(())
}

/// Get descriptions with adaptive concurrency (fallback for missing descriptions)
pub async fn add_missing_descriptions_fast(
    packages: Vec<Package>,
    packages_clone: std::sync::Arc<tokio::sync::RwLock<Vec<Package>>>,
) {
    use crate::utils::run_command_with_timeout;
    use std::time::Duration;
    use futures::{stream, StreamExt};
    
    // Only fetch for packages missing descriptions
    let missing: Vec<String> = packages.iter()
        .filter(|p| p.description.is_none())
        .map(|p| p.name.clone())
        .collect();
    
    if missing.is_empty() {
        println!("[FAST] ✓ All packages have descriptions!");
        return;
    }
    
    println!("[FAST] Fetching {} missing descriptions (adaptive concurrency)...", missing.len());
    
    const CONCURRENT_REQUESTS: usize = 8; // Higher since API already gave us most
    
    let total = missing.len();
    let mut completed = 0;
    
    let mut stream = stream::iter(missing)
        .map(|name| async move {
            let result = run_command_with_timeout(
                "brew",
                &["info", "--json=v2", &name],
                Duration::from_secs(10),
            ).await;
            
            (name.clone(), result)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);
    
    while let Some((name, result)) = stream.next().await {
        if let Ok(output) = result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    let desc = json.get("formulae")
                        .and_then(|v| v.as_array())
                        .and_then(|arr| arr.first())
                        .and_then(|f| f.get("desc"))
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    
                    if let Some(description) = desc {
                        let mut pkgs = packages_clone.write().await;
                        if let Some(pkg) = pkgs.iter_mut().find(|p| p.name == name) {
                            pkg.description = Some(description);
                        }
                    }
                }
            }
        }
        
        completed += 1;
        if completed % 5 == 0 || completed == total {
            println!("[FAST] Missing descriptions: {}/{}", completed, total);
        }
    }
}

/// Update a single package
pub async fn update_package(package_name: String) -> Result<()> {
    use crate::utils::run_command_with_timeout;
    use std::time::Duration;
    
    println!("[UPDATE] Updating {}...", package_name);
    
    let output = run_command_with_timeout(
        "brew",
        &["upgrade", &package_name],
        Duration::from_secs(300), // 5 minutes
    )
    .await?;
    
    if output.status.success() {
        println!("[UPDATE] ✓ Successfully updated {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to update {}: {}", package_name, stderr))
    }
}

/// Update all outdated packages
pub async fn update_all_packages() -> Result<()> {
    use crate::utils::run_command_with_timeout;
    use std::time::Duration;
    
    println!("[UPDATE] Updating all outdated packages...");
    
    let output = run_command_with_timeout(
        "brew",
        &["upgrade"],
        Duration::from_secs(600), // 10 minutes
    )
    .await?;
    
    if output.status.success() {
        println!("[UPDATE] ✓ Successfully updated all packages");
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to update packages: {}", stderr))
    }
}

/// Uninstall a package
pub async fn uninstall_package(package_name: String) -> Result<()> {
    use crate::utils::run_command_with_timeout;
    use std::time::Duration;
    
    println!("[REMOVE] Uninstalling {}...", package_name);
    
    let output = run_command_with_timeout(
        "brew",
        &["uninstall", &package_name],
        Duration::from_secs(120), // 2 minutes
    )
    .await?;
    
    if output.status.success() {
        println!("[REMOVE] ✓ Successfully uninstalled {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to uninstall {}: {}", package_name, stderr))
    }
}

