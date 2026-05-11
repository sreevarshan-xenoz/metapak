//! APK backend for Alpine Linux

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct ApkBackend;

impl ApkBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for ApkBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Apk
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("apk").args(["search", "-v", query]).output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Format: package-name-version - description
                if let Some((name_ver, desc)) = line.split_once(" - ") {
                    // Split name from version (last hyphen before a digit)
                    let name = name_ver
                        .rfind(|c: char| c == '-' && name_ver[..].ends_with(|d: char| d.is_ascii_digit()))
                        .map(|pos| &name_ver[..pos])
                        .unwrap_or(name_ver);
                    Some(create_package(
                        name.to_string(),
                        "?".to_string(),
                        desc.trim().to_string(),
                        PackageSource::Pacman, // Generic Linux source
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        Command::new("apk")
            .args(["info", "-e", pkg_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("apk").args(["info", "-v"]).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                Some(create_package(
                    line.to_string(),
                    "?".to_string(),
                    String::new(),
                    PackageSource::Pacman,
                ))
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("apk")
            .args(["version", "-v", "-l", "<"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let updates: Vec<OutdatedPackage> = stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            Some(OutdatedPackage::new(
                                parts[0].to_string(),
                                parts.get(1).unwrap_or(&"?").to_string(),
                                parts.get(2).unwrap_or(&"?").to_string(),
                                "main".to_string(),
                            ))
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
        let mut args = vec!["apk".to_string(), "add".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["apk".to_string(), "del".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::new("sudo", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("sudo", vec!["apk".to_string(), "upgrade".to_string()])
    }
}

impl Default for ApkBackend {
    fn default() -> Self {
        Self::new()
    }
}
