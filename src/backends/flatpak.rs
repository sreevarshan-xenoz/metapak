//! Flatpak backend for Linux

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct FlatpakBackend;

impl FlatpakBackend {
    pub fn new() -> Self {
        Self
    }

    fn is_available() -> bool {
        Command::new("which")
            .arg("flatpak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl UniversalPackageManager for FlatpakBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Flatpak
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("flatpak")
            .args(["search", query, "--columns=name,description,version,installed"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    let name = parts.get(0)?.trim().to_string();
                    let description = parts.get(1)?.trim().to_string();
                    let version = parts.get(2).map(|v| v.trim().to_string()).unwrap_or_else(|| "?".to_string());
                    let is_installed = parts.get(3).map(|s| s.trim() == "Installed").unwrap_or(false);

                    let mut pkg = create_package(name, version, description, PackageSource::Flatpak);
                    pkg.is_installed = is_installed;
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        if !Self::is_available() {
            return false;
        }

        let output = Command::new("flatpak")
            .args(["info", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("flatpak")
            .args(["list", "--columns=application,name,version,description"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    Some(create_package(
                        parts.get(0).map(|s| s.trim()).unwrap_or("").to_string(),
                        parts.get(2).unwrap_or(&"?").to_string(),
                        parts.get(3).map(|s| s.trim()).unwrap_or("").to_string(),
                        PackageSource::Flatpak,
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("flatpak")
            .args(["update", "--dry-run"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .filter_map(|line| {
                if line.contains("Update") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts.get(1)?.to_string();
                        return Some(OutdatedPackage {
                            name,
                            current_version: "?".to_string(),
                            new_version: "?".to_string(),
                        });
                    }
                }
                None
            })
            .collect();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let args: Vec<String> = ["install", "-y"]
            .iter()
            .chain(packages.iter())
            .map(|s| s.to_string())
            .collect();
        CommandSpec::new("flatpak", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let args: Vec<String> = ["uninstall", "-y"]
            .iter()
            .chain(packages.iter())
            .map(|s| s.to_string())
            .collect();
        CommandSpec::new("flatpak", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("flatpak", vec!["update", "-y".to_string()])
    }
}