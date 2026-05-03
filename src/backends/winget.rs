//! Winget backend for Windows

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct WingetBackend;

impl WingetBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for WingetBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Winget
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("winget")
            .args(["search", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(2) // Skip header lines
            .filter_map(|line| {
                // Winget output format varies, try to parse
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let name = parts.first()?;
                    if *name == "Name" || name.is_empty() {
                        return None;
                    }
                    // Find version column (usually second to last)
                    let version = parts.get(parts.len() - 2).unwrap_or(&"?");
                    Some(create_package(
                        name.to_string(),
                        version.to_string(),
                        "Windows package".to_string(),
                        PackageSource::Winget,
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("winget")
            .args(["list", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("winget")
            .args(["list"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(2) // Skip header
            .filter_map(|line| {
                if line.trim().is_empty() {
                    return None;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Some(create_package(
                        parts[0].to_string(),
                        parts.get(parts.len() - 1).unwrap_or(&"?").to_string(),
                        String::new(),
                        PackageSource::Pacman,
                    ));
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("winget")
            .args(["upgrade", "--include-unknown"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .skip(2)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    return Some(OutdatedPackage::new(
                        parts[0].to_string(),
                        parts.get(1).map(|s| s.to_string()).unwrap_or_else(|| "?".to_string()),
                        parts.get(parts.len() - 1).unwrap_or(&"?").to_string(),
                        "winget".to_string(),
                    ));
                }
                None
            })
            .collect();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string(), "--accept-package-agreements".to_string(), "--accept-source-agreements".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("winget", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("winget", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo("winget", vec!["upgrade".to_string(), "--all".to_string()])
    }
}

impl Default for WingetBackend {
    fn default() -> Self {
        Self::new()
    }
}