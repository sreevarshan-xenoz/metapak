//! Zypper backend for openSUSE

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct ZypperBackend;

impl ZypperBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for ZypperBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Zypper
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("zypper")
            .args(["search", "--no-color", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Skip header lines
                if line.starts_with("S |") || line.starts_with("--") || line.trim().is_empty() {
                    return None;
                }
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let name = parts.get(1)?.trim().to_string();
                    if name.is_empty() {
                        return None;
                    }
                    Some(create_package(
                        name,
                        "?".to_string(),
                        parts.get(2).unwrap_or(&"").trim().to_string(),
                        PackageSource::Pacman,
                    ))
                } else {
                    None
                }
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
        let output = Command::new("zypper")
            .args(["list-installed", "--no-color"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                if line.starts_with("i |") {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 2 {
                        return Some(create_package(
                            parts.get(1)?.trim().to_string(),
                            parts.get(2).unwrap_or(&"?").trim().to_string(),
                            String::new(),
                            PackageSource::Pacman,
                        ));
                    }
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("zypper")
            .args(["list-updates", "--no-color"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .filter_map(|line| {
                if line.starts_with("v |") {
                    let parts: Vec<&str> = line.split('|').collect();
                    if parts.len() >= 4 {
                        return Some(OutdatedPackage::new(
                            parts.get(1)?.trim().to_string(),
                            parts.get(3).unwrap_or(&"?").trim().to_string(),
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

impl Default for ZypperBackend {
    fn default() -> Self {
        Self::new()
    }
}