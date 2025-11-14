use crate::models::{Package, PackageManager};
use crate::utils::run_command_with_timeout;
use anyhow::{anyhow, Result};
use std::time::Duration;

/// List globally installed npm packages
pub async fn list_npm_packages() -> Result<Vec<Package>> {
    println!("[NPM] Listing global packages");

    let output = run_command_with_timeout(
        "npm",
        &["list", "-g", "--depth=0", "--json"],
        Duration::from_secs(30),
    )
    .await?;

    if !output.status.success() {
        return Err(anyhow!("npm list failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)?;

    let mut packages = Vec::new();

    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
        for (name, info) in deps {
            if let Some(version) = info.get("version").and_then(|v| v.as_str()) {
                packages.push(Package {
                    name: name.clone(),
                    manager: PackageManager::Npm,
                    installed_version: version.to_string(),
                    latest_version: None,
                    is_outdated: false,
                    description: None,
                    used_in: vec![],
                    size: None,
                });
            }
        }
    }

    println!("[NPM] Found {} global packages", packages.len());
    Ok(packages)
}

/// Check for outdated npm packages
pub async fn check_outdated_npm(packages: &mut [Package]) -> Result<()> {
    println!("[NPM] Checking for outdated packages");

    let output = run_command_with_timeout(
        "npm",
        &["outdated", "-g", "--json"],
        Duration::from_secs(30),
    )
    .await?;

    // npm outdated returns exit code 1 when there are outdated packages, so we don't check status
    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
        if let Some(outdated) = json.as_object() {
            for pkg in packages.iter_mut() {
                if let Some(info) = outdated.get(&pkg.name) {
                    if let Some(latest) = info.get("latest").and_then(|v| v.as_str()) {
                        pkg.latest_version = Some(latest.to_string());
                        pkg.is_outdated = true;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Fetch descriptions for npm packages (parallel)
pub async fn add_npm_descriptions(
    packages: std::sync::Arc<tokio::sync::RwLock<Vec<crate::models::Package>>>,
) {
    use futures::{stream, StreamExt};

    println!("[NPM] Fetching package descriptions");

    let packages_read = packages.read().await;
    let npm_packages: Vec<String> = packages_read
        .iter()
        .filter(|p| p.manager == crate::models::PackageManager::Npm && p.description.is_none())
        .map(|p| p.name.clone())
        .collect();
    drop(packages_read);

    if npm_packages.is_empty() {
        return;
    }

    let total = npm_packages.len();
    println!("[NPM] Fetching descriptions for {} packages", total);

    const CONCURRENT_REQUESTS: usize = 8;
    let mut completed = 0;

    let mut stream = stream::iter(npm_packages)
        .map(|name| async move {
            let result = run_command_with_timeout(
                "npm",
                &["view", &name, "description"],
                Duration::from_secs(5),
            )
            .await;
            (name, result)
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((name, result)) = stream.next().await {
        if let Ok(output) = result {
            if output.status.success() {
                let desc = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !desc.is_empty() {
                    let mut packages_lock = packages.write().await;
                    if let Some(pkg) = packages_lock.iter_mut().find(|p| p.name == name) {
                        pkg.description = Some(desc);
                    }
                }
            }
        }

        completed += 1;
        if completed % 5 == 0 || completed == total {
            println!("[NPM] Descriptions: {}/{}", completed, total);
        }
    }

    println!("[NPM] Finished fetching descriptions");
}

/// Update an npm package
pub async fn update_npm_package(package_name: String) -> Result<()> {
    println!("[NPM] Updating: {}", package_name);

    let output = run_command_with_timeout(
        "npm",
        &["update", "-g", &package_name],
        Duration::from_secs(300),
    )
    .await?;

    if output.status.success() {
        println!("[NPM] Successfully updated: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to update {}: {}", package_name, stderr))
    }
}

/// Uninstall an npm package
pub async fn uninstall_npm_package(package_name: String) -> Result<()> {
    println!("[NPM] Uninstalling: {}", package_name);

    let output = run_command_with_timeout(
        "npm",
        &["uninstall", "-g", &package_name],
        Duration::from_secs(120),
    )
    .await?;

    if output.status.success() {
        println!("[NPM] Successfully uninstalled: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to uninstall {}: {}", package_name, stderr))
    }
}

/// Install an npm package
pub async fn install_npm_package(package_name: String) -> Result<()> {
    println!("[NPM] Installing: {}", package_name);

    let output = run_command_with_timeout(
        "npm",
        &["install", "-g", &package_name],
        Duration::from_secs(300),
    )
    .await?;

    if output.status.success() {
        println!("[NPM] Successfully installed: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to install {}: {}", package_name, stderr))
    }
}
