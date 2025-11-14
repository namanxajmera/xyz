use anyhow::{anyhow, Result};
use crate::models::{Package, PackageManager};
use crate::utils::run_command_with_timeout;
use serde_json::Value;
use std::time::Duration;

pub async fn list_homebrew_packages() -> Result<Vec<Package>> {
    println!("[DEBUG] Getting Homebrew package list...");
    
    // Get list of installed packages (plain text)
    let output = run_command_with_timeout(
        "brew",
        &["list"],
        Duration::from_secs(10),
    )
    .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("brew list failed: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let package_names: Vec<&str> = stdout.lines().filter(|s| !s.is_empty()).collect();
    
    println!("[DEBUG] Found {} Homebrew packages", package_names.len());
    
    if package_names.is_empty() {
        return Ok(Vec::new());
    }

    // Get detailed info - batch packages to avoid timeout
    // Process in batches of 20 to avoid command line length limits and timeouts
    let mut packages = Vec::new();
    const BATCH_SIZE: usize = 20;
    
    for chunk in package_names.chunks(BATCH_SIZE) {
        println!("[DEBUG] Processing batch of {} packages...", chunk.len());
        
        // Build command args: ["info", "--json=v2", "pkg1", "pkg2", ...]
        let mut args = vec!["info", "--json=v2"];
        args.extend(chunk.iter().map(|s| *s));
        
        let info_output = run_command_with_timeout(
            "brew",
            &args,
            Duration::from_secs(60), // Increased timeout per batch
        )
        .await;

        match info_output {
            Ok(output) if output.status.success() => {
                let info_stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<Value>(&info_stdout) {
                    // Parse formulae
                    if let Some(formulae) = json.get("formulae").and_then(|v| v.as_array()) {
                        for formula in formulae {
                            if let Some(name) = formula.get("name").and_then(|v| v.as_str()) {
                                // Get installed version - check installed array first
                                let version = formula
                                    .get("installed")
                                    .and_then(|v| v.as_array())
                                    .and_then(|v| v.first())
                                    .and_then(|v| v.get("version"))
                                    .and_then(|v| v.as_str())
                                    .or_else(|| {
                                        formula.get("versions")
                                            .and_then(|v| v.get("stable"))
                                            .and_then(|v| v.as_str())
                                    })
                                    .unwrap_or("unknown")
                                    .to_string();

                                packages.push(Package::new(
                                    name.to_string(),
                                    PackageManager::Homebrew,
                                    version,
                                ));
                            }
                        }
                    }

                    // Parse casks
                    if let Some(casks) = json.get("casks").and_then(|v| v.as_array()) {
                        for cask in casks {
                            if let Some(name) = cask.get("token").and_then(|v| v.as_str())
                                .or_else(|| cask.get("name").and_then(|v| v.as_str())) {
                                let version = cask
                                    .get("installed")
                                    .and_then(|v| v.as_str())
                                    .or_else(|| cask.get("version").and_then(|v| v.as_str()))
                                    .unwrap_or("unknown")
                                    .to_string();

                                packages.push(Package::new(
                                    name.to_string(),
                                    PackageManager::Homebrew,
                                    version,
                                ));
                            }
                        }
                    }
                } else {
                    eprintln!("[WARN] Failed to parse JSON for batch");
                }
            }
            Err(e) => {
                eprintln!("[WARN] Failed to get detailed info for batch: {}. Using basic list.", e);
                // Fallback: just use package names with "unknown" version for this batch
                for name in chunk {
                    packages.push(Package::new(
                        name.to_string(),
                        PackageManager::Homebrew,
                        "unknown".to_string(),
                    ));
                }
            }
            _ => {
                // Fallback: just use package names with "unknown" version for this batch
                for name in chunk {
                    packages.push(Package::new(
                        name.to_string(),
                        PackageManager::Homebrew,
                        "unknown".to_string(),
                    ));
                }
            }
        }
    }
    
    println!("[DEBUG] Processed {} packages total", packages.len());

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

