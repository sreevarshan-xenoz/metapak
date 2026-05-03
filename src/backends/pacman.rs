//! Pacman backend for Arch Linux

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, parse_version, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct PacmanBackend;

impl PacmanBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for PacmanBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Pacman
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("pacman")
            .args(["-Ss", "--color", "never", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut packages = Vec::new();
        let mut lines = stdout.lines();

        while let Some(header) = lines.next() {
            if let Some(desc) = lines.next() {
                let parts: Vec<&str> = header.split_whitespace().collect();
                if parts.len() >= 2 {
                    let full_name = parts[0];
                    let version = parts[1];
                    let is_installed = header.contains("[installed]");

                    let name = full_name.split('/').nth(1).unwrap_or(full_name).to_string();

                    packages.push(create_package(
                        name,
                        version.to_string(),
                        desc.trim().to_string(),
                        PackageSource::Pacman,
                    ));
                }
            }
        }

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("pacman")
            .args(["-Qi", "--color", "never", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("pacman")
            .args(["-Q", "--color", "never"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
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
        // Try checkupdates first (doesn't require sudo)
        if let Ok(output) = Command::new("checkupdates").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let updates: Vec<OutdatedPackage> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
Some(OutdatedPackage::new(
                            parts[0].to_string(),
                            parts[1].to_string(),
                            parts.get(2).map(|s| s.to_string()).unwrap_or_else(|| "".to_string()),
                            "core".to_string(),
                        ))
                        } else {
                            None
                        }
                    })
                    .collect();
                return Ok(updates);
            }
        }

        // Fallback to pacman -Qu
        let output = Command::new("pacman")
            .args(["-Qu", "--color", "never"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let updates: Vec<OutdatedPackage> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(OutdatedPackage::new(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts.get(2).map(|s| s.to_string()).unwrap_or_else(|| "".to_string()),
                        "core".to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["-S".to_string(), "--noconfirm".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("sudo", vec!["-Syu".to_string(), "--noconfirm".to_string()])
    }
}

impl Default for PacmanBackend {
    fn default() -> Self {
        Self::new()
    }
}