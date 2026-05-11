//! Scoop backend for Windows

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct ScoopBackend;

impl ScoopBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for ScoopBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Scoop
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = Command::new("scoop").args(["search", query]).output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(2) // Skip header lines
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(create_package(
                        parts[0].to_string(),
                        parts.get(1).unwrap_or(&"?").to_string(),
                        String::new(),
                        PackageSource::Pacman, // Generic source
                    ))
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        Command::new("scoop")
            .args(["info", pkg_name])
            .output()
            .map(|o| {
                o.status.success()
                    && String::from_utf8_lossy(&o.stdout).contains("Installed:")
            })
            .unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = Command::new("scoop").args(["list"]).output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .skip(2) // Skip header
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mut pkg = create_package(
                        parts[0].to_string(),
                        parts[1].to_string(),
                        String::new(),
                        PackageSource::Pacman,
                    );
                    pkg.is_installed = true;
                    Some(pkg)
                } else {
                    None
                }
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = Command::new("scoop").args(["status"]).output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let updates: Vec<OutdatedPackage> = stdout
                    .lines()
                    .skip(2) // Skip header
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            Some(OutdatedPackage::new(
                                parts[0].to_string(),
                                parts[1].to_string(),
                                parts[2].to_string(),
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
        let mut args = vec!["install".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("scoop", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("scoop", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo("scoop", vec!["update".to_string(), "*".to_string()])
    }
}

impl Default for ScoopBackend {
    fn default() -> Self {
        Self::new()
    }
}
