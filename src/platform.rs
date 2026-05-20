//! Platform detection and package manager identification
//!
//! This module provides utilities to detect the current operating system
//! and identify available package managers on the system.

use std::process::Command;

/// Represents the operating system platform
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Platform {
    Linux,
    Macos,
    Windows,
    Unknown,
}

/// Represents different package manager types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManager {
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Apk,
    Brew,
    Winget,
    Chocolatey,
    Scoop,
    Flatpak,
    Snap,
    None,
}

impl PackageManager {
    pub fn name(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "pacman",
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Zypper => "zypper",
            PackageManager::Apk => "apk",
            PackageManager::Brew => "brew",
            PackageManager::Winget => "winget",
            PackageManager::Chocolatey => "chocolatey",
            PackageManager::Scoop => "scoop",
            PackageManager::Flatpak => "flatpak",
            PackageManager::Snap => "snap",
            PackageManager::None => "none",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PackageManager::Pacman => "Pacman",
            PackageManager::Apt => "APT",
            PackageManager::Dnf => "DNF",
            PackageManager::Zypper => "Zypper",
            PackageManager::Apk => "APK",
            PackageManager::Brew => "Homebrew",
            PackageManager::Winget => "Winget",
            PackageManager::Chocolatey => "Chocolatey",
            PackageManager::Scoop => "Scoop",
            PackageManager::Flatpak => "Flatpak",
            PackageManager::Snap => "Snap",
            PackageManager::None => "None",
        }
    }
}

/// Detects the current platform
pub fn detect_platform() -> Platform {
    #[cfg(target_os = "windows")]
    return Platform::Windows;

    #[cfg(target_os = "macos")]
    return Platform::Macos;

    #[cfg(target_os = "linux")]
    return Platform::Linux;

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return Platform::Unknown;
}

/// Detects the current Linux distribution from /etc/os-release
pub fn detect_linux_distro() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                return line.trim_start_matches("ID=").trim_matches('"').to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Detects available package managers on the current platform
pub fn detect_package_managers() -> Vec<PackageManager> {
    let mut managers = Vec::new();
    let platform = detect_platform();

    match platform {
        Platform::Linux => {
            if command_exists("pacman") {
                managers.push(PackageManager::Pacman);
            }
            if command_exists("apt") {
                managers.push(PackageManager::Apt);
            }
            if command_exists("dnf") {
                managers.push(PackageManager::Dnf);
            }
            if command_exists("zypper") {
                managers.push(PackageManager::Zypper);
            }
            if command_exists("apk") {
                managers.push(PackageManager::Apk);
            }
            if command_exists("brew") {
                managers.push(PackageManager::Brew);
            }
            if command_exists("flatpak") {
                managers.push(PackageManager::Flatpak);
            }
            if command_exists("snap") {
                managers.push(PackageManager::Snap);
            }
        }
        Platform::Macos => {
            if command_exists("brew") {
                managers.push(PackageManager::Brew);
            }
            if command_exists("port") {
                // macports - not adding by default
            }
        }
        Platform::Windows => {
            if command_exists("winget") {
                managers.push(PackageManager::Winget);
            }
            if command_exists("choco") {
                managers.push(PackageManager::Chocolatey);
            }
            if command_exists("scoop") {
                managers.push(PackageManager::Scoop);
            }
        }
        Platform::Unknown => {}
    }

    managers
}

/// Gets the default package manager for a platform
pub fn get_default_package_manager() -> PackageManager {
    let managers = detect_package_managers();
    managers.into_iter().next().unwrap_or(PackageManager::None)
}

/// Check if a command exists on the system
fn command_exists(cmd: &str) -> bool {
    // Try with "where" on Windows, "which" on Unix
    #[cfg(target_os = "windows")]
    {
        Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Get platform info string for display
pub fn get_platform_info() -> String {
    let platform = detect_platform();
    let managers = detect_package_managers();

    let platform_str = match platform {
        Platform::Linux => {
            let distro = detect_linux_distro();
            format!("Linux ({})", distro)
        }
        Platform::Macos => "macOS".to_string(),
        Platform::Windows => "Windows".to_string(),
        Platform::Unknown => "Unknown".to_string(),
    };

    let manager_names: Vec<&str> = managers.iter().map(|m| m.display_name()).collect();
    let manager_str = if manager_names.is_empty() {
        "none".to_string()
    } else {
        manager_names.join(", ")
    };

    format!("{} | Package Managers: {}", platform_str, manager_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = detect_platform();
        // Just ensure it returns a valid platform
        match platform {
            Platform::Linux | Platform::Macos | Platform::Windows | Platform::Unknown => {}
        }
    }

    #[test]
    fn test_package_manager_detection() {
        let _managers = detect_package_managers();
    }
}
