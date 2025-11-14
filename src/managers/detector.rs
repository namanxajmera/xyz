use crate::models::PackageManager;
use crate::utils::command_exists;

pub async fn detect_available_managers() -> Vec<PackageManager> {
    let mut available = Vec::new();

    // Check each package manager using the command() method
    let managers_to_check = vec![
        PackageManager::Homebrew,
        PackageManager::Npm,
        PackageManager::Yarn,
        PackageManager::Pnpm,
        PackageManager::Cargo,
        PackageManager::Pip,
        PackageManager::Pipx,
        PackageManager::Gem,
        PackageManager::Go,
        PackageManager::Composer,
        PackageManager::Pub,
        PackageManager::Swift,
    ];

    for manager in managers_to_check {
        let cmd = manager.command();
        if command_exists(cmd).await {
            available.push(manager);
        }
    }

    available
}
