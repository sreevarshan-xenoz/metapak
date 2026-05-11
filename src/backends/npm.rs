//! NPM ecosystem backend for Node.js packages

use async_trait::async_trait;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct NpmBackend;

impl NpmBackend {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UniversalPackageManager for NpmBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::None // Ecosystem, not system
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        let output = std::process::Command::new("npm")
            .args(["search", "--json", query])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse JSON array of results
        if let Ok(results) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
            let packages: Vec<Package> = results
                .iter()
                .take(50)
                .filter_map(|item| {
                    let name = item.get("name")?.as_str()?;
                    let version = item
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("?");
                    let description = item
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    Some(create_package(
                        name.to_string(),
                        version.to_string(),
                        description.to_string(),
                        PackageSource::Npm,
                    ))
                })
                .collect();
            Ok(packages)
        } else {
            Ok(Vec::new())
        }
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        std::process::Command::new("npm")
            .args(["ls", pkg_name, "--depth=0", "--json"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        let output = std::process::Command::new("npm")
            .args(["ls", "--global", "--depth=0", "--json"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                let packages: Vec<Package> = deps
                    .iter()
                    .map(|(name, info)| {
                        let version = info
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        let mut pkg = create_package(
                            name.to_string(),
                            version.to_string(),
                            String::new(),
                            PackageSource::Npm,
                        );
                        pkg.is_installed = true;
                        pkg
                    })
                    .collect();
                return Ok(packages);
            }
        }
        Ok(Vec::new())
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        let output = std::process::Command::new("npm")
            .args(["outdated", "--global", "--json"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    if let Some(obj) = json.as_object() {
                        let updates: Vec<OutdatedPackage> = obj
                            .iter()
                            .filter_map(|(name, info)| {
                                let current = info.get("current")?.as_str()?;
                                let wanted = info.get("wanted")?.as_str()?;
                                Some(OutdatedPackage::new(
                                    name.to_string(),
                                    current.to_string(),
                                    wanted.to_string(),
                                    "npm".to_string(),
                                ))
                            })
                            .collect();
                        return Ok(updates);
                    }
                }
                Ok(Vec::new())
            }
            Err(_) => Ok(Vec::new()),
        }
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["install".to_string(), "--global".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("npm", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["uninstall".to_string(), "--global".to_string()];
        args.extend(packages.iter().map(|s| s.to_string()));
        CommandSpec::no_sudo("npm", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::no_sudo(
            "npm",
            vec!["update".to_string(), "--global".to_string()],
        )
    }
}

impl Default for NpmBackend {
    fn default() -> Self {
        Self::new()
    }
}
