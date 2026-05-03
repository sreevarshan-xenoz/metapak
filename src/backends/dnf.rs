//! DNF backend for Fedora/RHEL

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct DnfBackend;

impl DnfBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for DnfBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Dnf
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("dnf")
            .args(["search", "--no Color", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                if line.contains(" : ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if !parts.is_empty() {
                        // DNF search format: name.summary
                        let name = parts.first()?.split('.').next()?;
                        return Some(create_package(
                            name.to_string(),
                            "?".to_string(),
                            line.split(" : ").nth(1).unwrap_or("").to_string(),
                            PackageSource::Pacman,
                        ));
                    }
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("rpm")
            .args(["-q", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("dnf")
            .args(["list", "--installed", "--no Color"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(1) // Skip header
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(create_package(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        String::new(),
                        PackageSource::Pacman,
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("dnf")
            .args(["check-update", "--no Color"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] != "Last" && parts[0] != "Loaded" {
                    Some(OutdatedPackage::new(
                        parts[0].to_string(),
                        parts.get(1).unwrap_or(&"?").to_string(),
                        parts.get(2).map(|s| s.to_string()).unwrap_or_else(|| "?".to_string()),
                        "fedora".to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string(), "-y".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["remove".to_string(), "-y".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("sudo", vec!["update".to_string(), "-y".to_string()])
    }
}

impl Default for DnfBackend {
    fn default() -> Self {
        Self::new()
    }
}