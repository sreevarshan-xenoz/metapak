//! Chocolatey backend for Windows

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct ChocolateyBackend;

impl ChocolateyBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for ChocolateyBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Chocolatey
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("choco")
            .args(["search", query, "--limit-output"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Format: name|version|...
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    return Some(create_package(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        String::new(),
                        PackageSource::Chocolatey,
                    ));
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("choco")
            .args(["list", "--local-only", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("choco")
            .args(["list", "--local-only", "--limit-output"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                if line.is_empty() {
                    return None;
                }
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    return Some(create_package(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        String::new(),
                        PackageSource::Chocolatey,
                    ));
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("choco").args(["outdated"]).output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .filter_map(|line| {
                if line.contains('|') {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 3 {
                        return Some(OutdatedPackage::new(
                            parts[0].trim().to_string(),
                            parts[1].trim().to_string(),
                            parts[2].trim().to_string(),
                            "chocolatey".to_string(),
                        ));
                    }
                }
                None
            })
            .collect();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string(), "-y".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("choco", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string(), "-y".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("choco", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("choco", vec!["upgrade".to_string(), "-y".to_string()])
    }
}

impl Default for ChocolateyBackend {
    fn default() -> Self {
        Self::new()
    }
}
