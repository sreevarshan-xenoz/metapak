//! Homebrew backend for macOS and Linux

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{CommandSpec, UniversalPackageManager, create_package};
use crate::errors::Result;
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::platform::PackageManager;

pub struct BrewBackend;

impl BrewBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for BrewBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Brew
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("brew")
            .args(["search", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Brew search output: just package names (one per line)
        let packages: Vec<Package> = stdout
            .lines()
            .filter(|line| !line.starts_with("==>"))
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                create_package(
                    line.trim().to_string(),
                    "?".to_string(),
                    String::new(),
                    PackageSource::Pacman,
                )
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let output = Command::new("brew")
            .args(["list", pkg_name])
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("brew")
            .args(["list", "--versions"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Format: package version version...
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let name = parts[0].to_string();
                    let version = parts.get(1).unwrap_or(&"?").to_string();
                    return Some(create_package(name, version, String::new(), PackageSource::Pacman));
                }
                None
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("brew")
            .args(["outdated", "--json"])
            .output()?;

        // Parse JSON output for updates
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // For now, return empty - JSON parsing needed
            // In production, use serde_json to parse
            let _ = stdout;
        }

        Ok(Vec::new())
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("brew", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("brew", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo("brew", vec!["upgrade".to_string()])
    }
}

impl Default for BrewBackend {
    fn default() -> Self {
        Self::new()
    }
}