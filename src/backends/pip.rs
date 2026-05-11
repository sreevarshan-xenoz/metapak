//! Pip (Python) ecosystem backend

use async_trait::async_trait;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct PipBackend;

impl PipBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for PipBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::None // Ecosystem, not system
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        // pip search has been disabled on PyPI since 2021
        // Use pip index versions or a direct PyPI API call instead
        let url = format!(
            "https://pypi.org/simple/?q={}",
            urlencoding::encode(query)
        );

        // Fallback: use `pip install <query>==` to get version info
        // For now, do a simple pip index search
        let output = std::process::Command::new("pip")
            .args(["index", "versions", query])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // Format: package (version)
                // Available versions: ...
                let mut packages = Vec::new();
                for line in stdout.lines() {
                    if line.contains("(") && line.contains(")") {
                        if let Some(name_end) = line.find('(') {
                            let name = line[..name_end].trim();
                            let version = line[name_end..]
                                .trim_start_matches('(')
                                .trim_end_matches(')')
                                .trim();
                            packages.push(create_package(
                                name.to_string(),
                                version.to_string(),
                                String::new(),
                                PackageSource::Pip,
                            ));
                        }
                    }
                }
                Ok(packages)
            }
            _ => {
                // If pip index doesn't work, return the query as a potential package
                Ok(vec![create_package(
                    query.to_string(),
                    "?".to_string(),
                    "Search via pip install".to_string(),
                    PackageSource::Pip,
                )])
            }
        }
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        std::process::Command::new("pip")
            .args(["show", pkg_name])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = std::process::Command::new("pip")
            .args(["list", "--format=json"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
            let packages: Vec<Package> = results
                .iter()
                .filter_map(|item| {
                    let name = item.get("name")?.as_str()?;
                    let version = item.get("version")?.as_str()?;
                    let mut pkg = create_package(
                        name.to_string(),
                        version.to_string(),
                        String::new(),
                        PackageSource::Pip,
                    );
                    pkg.is_installed = true;
                    Some(pkg)
                })
                .collect();
            Ok(packages)
        } else {
            Ok(Vec::new())
        }
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = std::process::Command::new("pip")
            .args(["list", "--outdated", "--format=json"])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                    let updates: Vec<OutdatedPackage> = results
                        .iter()
                        .filter_map(|item| {
                            let name = item.get("name")?.as_str()?;
                            let current = item.get("version")?.as_str()?;
                            let latest = item.get("latest_version")?.as_str()?;
                            Some(OutdatedPackage::new(
                                name.to_string(),
                                current.to_string(),
                                latest.to_string(),
                                "pip".to_string(),
                            ))
                        })
                        .collect();
                    Ok(updates)
                } else {
                    Ok(Vec::new())
                }
            }
            _ => Ok(Vec::new()),
        }
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("pip", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string(), "-y".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("pip", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo(
            "pip",
            vec![
                "install".to_string(),
                "--upgrade".to_string(),
                "pip".to_string(),
            ],
        )
    }
}

impl Default for PipBackend {
    fn default() -> Self {
        Self::new()
    }
}
