use crate::models::{Package, PackageManager};
use crate::utils::run_command_with_timeout;
use anyhow::{anyhow, Result};
use std::time::Duration;

/// List globally installed pip packages
pub async fn list_pip_packages() -> Result<Vec<Package>> {
    println!("[PIP] Listing installed packages");

    let output =
        run_command_with_timeout("pip3", &["list", "--format=json"], Duration::from_secs(30))
            .await?;

    if !output.status.success() {
        return Err(anyhow!("pip3 list failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;

    let mut packages = Vec::new();

    for item in json {
        if let (Some(name), Some(version)) = (
            item.get("name").and_then(|n| n.as_str()),
            item.get("version").and_then(|v| v.as_str()),
        ) {
            packages.push(Package {
                name: name.to_string(),
                manager: PackageManager::Pip,
                installed_version: version.to_string(),
                latest_version: None,
                is_outdated: false,
                description: None,
                used_in: vec![],
                size: None,
            });
        }
    }

    println!("[PIP] Found {} installed packages", packages.len());
    Ok(packages)
}

/// Check for outdated pip packages
pub async fn check_outdated_pip(packages: &mut [Package]) -> Result<()> {
    println!("[PIP] Checking for outdated packages");

    let output = run_command_with_timeout(
        "pip3",
        &["list", "--outdated", "--format=json"],
        Duration::from_secs(60),
    )
    .await?;

    if !output.status.success() {
        return Ok(()); // Not a fatal error
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(json) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
        for item in json {
            if let (Some(name), Some(latest)) = (
                item.get("name").and_then(|n| n.as_str()),
                item.get("latest_version").and_then(|v| v.as_str()),
            ) {
                for pkg in packages.iter_mut() {
                    if pkg.name == name {
                        pkg.latest_version = Some(latest.to_string());
                        pkg.is_outdated = true;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fetch descriptions for pip packages
pub async fn add_pip_descriptions(
    packages: std::sync::Arc<tokio::sync::RwLock<Vec<crate::models::Package>>>,
) {
    use futures::{stream, StreamExt};

    println!("[PIP] Fetching package descriptions");

    let packages_read = packages.read().await;
    let pip_packages: Vec<String> = packages_read
        .iter()
        .filter(|p| p.manager == crate::models::PackageManager::Pip && p.description.is_none())
        .map(|p| p.name.clone())
        .collect();
    drop(packages_read);

    if pip_packages.is_empty() {
        return;
    }

    let total = pip_packages.len();
    println!("[PIP] Fetching descriptions for {} packages", total);

    const CONCURRENT_REQUESTS: usize = 8;
    let mut completed = 0;

    let mut stream = stream::iter(pip_packages)
        .map(|name| async move {
            let result =
                run_command_with_timeout("pip3", &["show", &name], Duration::from_secs(5)).await;
            (name, result)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((name, result)) = stream.next().await {
        if let Ok(output) = result {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse pip show output for Summary line
                for line in stdout.lines() {
                    if let Some(desc) = line.strip_prefix("Summary: ") {
                        let desc = desc.trim().to_string();
                        if !desc.is_empty() {
                            let mut packages_lock = packages.write().await;
                            if let Some(pkg) = packages_lock.iter_mut().find(|p| p.name == name) {
                                pkg.description = Some(desc);
                            }
                            break;
                        }
                    }
                }
            }
        }

        completed += 1;
        if completed % 5 == 0 || completed == total {
            println!("[PIP] Descriptions: {}/{}", completed, total);
        }
    }

    println!("[PIP] Finished fetching descriptions");
}

/// Update a pip package
pub async fn update_pip_package(package_name: String) -> Result<()> {
    println!("[PIP] Updating: {}", package_name);

    let output = run_command_with_timeout(
        "pip3",
        &["install", "--upgrade", &package_name],
        Duration::from_secs(300),
    )
    .await?;

    if output.status.success() {
        println!("[PIP] Successfully updated: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to update {}: {}", package_name, stderr))
    }
}

/// Uninstall a pip package
pub async fn uninstall_pip_package(package_name: String) -> Result<()> {
    println!("[PIP] Uninstalling: {}", package_name);

    let output = run_command_with_timeout(
        "pip3",
        &["uninstall", "-y", &package_name],
        Duration::from_secs(120),
    )
    .await?;

    if output.status.success() {
        println!("[PIP] Successfully uninstalled: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to uninstall {}: {}", package_name, stderr))
    }
}

/// Install a pip package
pub async fn install_pip_package(package_name: String) -> Result<()> {
    println!("[PIP] Installing: {}", package_name);

    let output = run_command_with_timeout(
        "pip3",
        &["install", &package_name],
        Duration::from_secs(300),
    )
    .await?;

    if output.status.success() {
        println!("[PIP] Successfully installed: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to install {}: {}", package_name, stderr))
    }
}
