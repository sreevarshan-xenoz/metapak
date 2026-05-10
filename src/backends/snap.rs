//! Snap backend for Linux

use async_trait::async_trait;
use std::process::Command;

use crate::backends::{create_package, CommandSpec, UniversalPackageManager};
use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

pub struct SnapBackend;

impl SnapBackend {
    pub fn new() -> Self {
        Self
    }

    fn is_available() -> bool {
        Command::new("which")
            .arg("snap")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl UniversalPackageManager for SnapBackend {
    fn get_name(&self) -> PackageManager {
        PackageManager::Snap
    }

    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("snap")
            .args(["find", query, "--format=json"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let results: serde_json::Value =
            serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
        let packages: Vec<Package> = results
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        let version = item.get("version")?.as_str().unwrap_or("?").to_string();
                        let summary = item.get("summary")?.as_str().unwrap_or("").to_string();

                        Some(create_package(name, version, summary, PackageSource::Snap))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        if !Self::is_available() {
            return false;
        }

        let output = Command::new("snap").args(["list", pkg_name]).output();

        output.map(|o| o.status.success()).unwrap_or(false)
    }

    async fn list_installed(&self) -> Result<Vec<Package>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("snap")
            .args(["list", "--format=json"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        let results: serde_json::Value =
            serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
        let packages: Vec<Package> = results
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let name = item.get("name")?.as_str()?.to_string();
                        let version = item.get("version")?.as_str().unwrap_or("?").to_string();
                        let notes = item.get("notes")?.as_str().unwrap_or("").to_string();
                        let description = if notes.contains("core") {
                            "Core snap".to_string()
                        } else {
                            String::new()
                        };

                        let mut pkg =
                            create_package(name, version, description, PackageSource::Snap);
                        pkg.is_installed = true;
                        Some(pkg)
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(packages)
    }

    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        if !Self::is_available() {
            return Ok(Vec::new());
        }

        let output = Command::new("snap")
            .args(["refresh", "--list", "--format=json"])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let results: serde_json::Value =
            serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
        let updates: Vec<OutdatedPackage> = results
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        Some(OutdatedPackage::new(
                            item.get("name")?.as_str()?.to_string(),
                            item.get("current")?.as_str().unwrap_or("?").to_string(),
                            item.get("version")?.as_str().unwrap_or("?").to_string(),
                            "snap".to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(updates)
    }

    fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        let args: Vec<String> = ["install"]
            .iter()
            .chain(packages.iter())
            .map(|s| s.to_string())
            .collect();
        CommandSpec::new("snap", args)
    }

    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        let args: Vec<String> = ["remove"]
            .iter()
            .chain(packages.iter())
            .map(|s| s.to_string())
            .collect();
        CommandSpec::new("snap", args)
    }

    fn build_update_command(&self) -> CommandSpec {
        CommandSpec::new("snap", vec!["refresh".to_string()])
    }
}
