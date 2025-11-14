use crate::models::{Package, PackageManager};
use crate::utils::run_command_with_timeout;
use anyhow::{anyhow, Result};
use std::time::Duration;

/// List installed cargo packages
pub async fn list_cargo_packages() -> Result<Vec<Package>> {
    println!("[CARGO] Listing installed packages");

    let output =
        run_command_with_timeout("cargo", &["install", "--list"], Duration::from_secs(30)).await?;

    if !output.status.success() {
        return Err(anyhow!("cargo install --list failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut packages = Vec::new();

    for line in stdout.lines() {
        // Lines with packages look like: "package-name v1.2.3:"
        if let Some(stripped) = line.strip_suffix(':') {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0];
                let version = parts[1].trim_start_matches('v');

                packages.push(Package {
                    name: name.to_string(),
                    manager: PackageManager::Cargo,
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

    println!("[CARGO] Found {} installed packages", packages.len());
    Ok(packages)
}

/// Check for outdated cargo packages using cargo-outdated if available
pub async fn check_outdated_cargo(_packages: &mut [Package]) -> Result<()> {
    // Note: Checking for outdated cargo binaries is complex
    // Would need cargo-outdated or cargo-update crate
    // For now, we'll skip this check
    println!("[CARGO] Outdated check not implemented yet");
    Ok(())
}

/// Fetch descriptions for cargo packages from crates.io API
pub async fn add_cargo_descriptions(
    packages: std::sync::Arc<tokio::sync::RwLock<Vec<crate::models::Package>>>,
) {
    use crate::utils::http_client::create_http_client;
    use futures::{stream, StreamExt};

    println!("[CARGO] Fetching package descriptions from crates.io");

    let packages_read = packages.read().await;
    let cargo_packages: Vec<String> = packages_read
        .iter()
        .filter(|p| p.manager == crate::models::PackageManager::Cargo && p.description.is_none())
        .map(|p| p.name.clone())
        .collect();
    drop(packages_read);

    if cargo_packages.is_empty() {
        return;
    }

    let total = cargo_packages.len();
    println!("[CARGO] Fetching descriptions for {} packages", total);

    let client = create_http_client();

    const CONCURRENT_REQUESTS: usize = 8;
    let mut completed = 0;

    let mut stream = stream::iter(cargo_packages)
        .map(|name| {
            let client = client.clone();
            async move {
                let url = format!("https://crates.io/api/v1/crates/{}", name);
                let result = client
                    .get(&url)
                    .header("User-Agent", "depmgr/0.1.0")
                    .send()
                    .await;
                (name, result)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((name, result)) = stream.next().await {
        if let Ok(response) = result {
            if response.status().is_success() {
                if let Ok(json) = response.json::<serde_json::Value>().await {
                    if let Some(desc) = json
                        .get("crate")
                        .and_then(|c| c.get("description"))
                        .and_then(|d| d.as_str())
                    {
                        let mut packages_lock = packages.write().await;
                        if let Some(pkg) = packages_lock.iter_mut().find(|p| p.name == name) {
                            pkg.description = Some(desc.to_string());
                        }
                    }
                }
            }
        }

        completed += 1;
        if completed % 5 == 0 || completed == total {
            println!("[CARGO] Descriptions: {}/{}", completed, total);
        }
    }

    println!("[CARGO] Finished fetching descriptions");
}

/// Update a cargo package
pub async fn update_cargo_package(package_name: String) -> Result<()> {
    println!("[CARGO] Updating: {}", package_name);

    let output = run_command_with_timeout(
        "cargo",
        &["install", &package_name, "--force"],
        Duration::from_secs(600), // 10 minutes for compilation
    )
    .await?;

    if output.status.success() {
        println!("[CARGO] Successfully updated: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to update {}: {}", package_name, stderr))
    }
}

/// Uninstall a cargo package
pub async fn uninstall_cargo_package(package_name: String) -> Result<()> {
    println!("[CARGO] Uninstalling: {}", package_name);

    let output = run_command_with_timeout(
        "cargo",
        &["uninstall", &package_name],
        Duration::from_secs(60),
    )
    .await?;

    if output.status.success() {
        println!("[CARGO] Successfully uninstalled: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to uninstall {}: {}", package_name, stderr))
    }
}

/// Install a cargo package
pub async fn install_cargo_package(package_name: String) -> Result<()> {
    println!("[CARGO] Installing: {}", package_name);

    let output = run_command_with_timeout(
        "cargo",
        &["install", &package_name],
        Duration::from_secs(600), // 10 minutes for compilation
    )
    .await?;

    if output.status.success() {
        println!("[CARGO] Successfully installed: {}", package_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!("Failed to install {}: {}", package_name, stderr))
    }
}
