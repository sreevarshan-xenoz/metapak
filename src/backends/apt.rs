//! APT backend for Debian/Ubuntu

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct AptBackend;

impl AptBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for AptBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Apt
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("apt-cache")
            .args(["search", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Format: package - description
                if let Some((name, desc)) = line.split_once(" - ") {
                    Some(create_package(
                        name.trim().to_string(),
                        "?".to_string(),
                        desc.trim().to_string(),
                        PackageSource::Pacman, // Using Pacman as generic Linux
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("dpkg")
            .args(["-l", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("dpkg")
            .args(["-l"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                if line.starts_with("ii ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        return Some(create_package(
                            parts[1].to_string(),
                            parts.get(2).unwrap_or(&"?").to_string(),
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
        // First update cache
        let _ = Command::new("sudo")
            .args(["apt-get", "update", "-qq"])
            .output();

        let output = Command::new("apt-list")
            .args(["--upgradable", "-qq"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let updates: Vec<OutdatedPackage> = stdout
                    .lines()
                    .filter_map(|line| {
                        // Format: package oldver -> newver
                        if let Some((name, rest)) = line.split_once(" ") {
                            let new_ver = rest.trim().split(" -> ").nth(1).unwrap_or("?");
                            Some(OutdatedPackage::new(name.trim().to_string(), "?".to_string(), new_ver.to_string(), "main".to_string()))
                        } else {
                            None
                        }
                    })
                    .collect();
                Ok(updates)
            }
            _ => Ok(Vec::new()),
        }
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
        CommandSpec::new("sudo", vec!["update".to_string(), "-y".to_string(), "upgrade".to_string()])
    }
}

impl Default for AptBackend {
    fn default() -> Self {
        Self::new()
    }
}