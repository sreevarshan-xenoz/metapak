//! Cargo (Rust) ecosystem backend

use async_trait::async_trait;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct CargoBackend;

impl CargoBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for CargoBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::None // Ecosystem, not system
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = std::process::Command::new("cargo")
            .args(["search", query, "--limit", "50"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Format: name = "version"    # description
                let parts: Vec<&str> = line.splitn(2, '#').collect();
                let name_ver = parts.first()?;
                let description = parts.get(1).map(|s| s.trim()).unwrap_or("");

                let nv_parts: Vec<&str> = name_ver.splitn(2, '=').collect();
                let name = nv_parts.first()?.trim();
                let version = nv_parts.get(1)
                    .map(|v| v.trim().trim_matches('"'))
                    .unwrap_or("?");

                if name.is_empty() {
                    return None;
                }

                Some(create_package(
                    name.to_string(),
                    version.to_string(),
                    description.to_string(),
                    PackageSource::Cargo,
                ))
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        std::process::Command::new("cargo")
            .args(["install", "--list"])
            .output()
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout.lines().any(|line| {
                    !line.starts_with(' ') && line.contains(pkg_name)
                })
            })
            .unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = std::process::Command::new("cargo")
            .args(["install", "--list"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = stdout
            .lines()
            .filter_map(|line| {
                // Lines that don't start with space are package names
                if line.starts_with(' ') || line.trim().is_empty() {
                    return None;
                }
                // Format: name v0.1.0:
                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts.first()?;
                let version = parts.get(1)
                    .map(|v| v.trim_start_matches('v').trim_end_matches(':'))
                    .unwrap_or("?");

                let mut pkg = create_package(
                    name.to_string(),
                    version.to_string(),
                    String::new(),
                    PackageSource::Cargo,
                );
                pkg.is_installed = true;
                Some(pkg)
            })
            .collect();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        // cargo doesn't have a built-in outdated command without cargo-outdated
        // Return empty for now
        Ok(Vec::new())
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("cargo", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("cargo", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo(
            "cargo",
            vec!["install".to_string(), "--list".to_string()],
        )
    }
}

impl Default for CargoBackend {
    fn default() -> Self {
        Self::new()
    }
}
