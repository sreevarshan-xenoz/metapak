//! Service layer for asynchronous operations
//! 
//! This module provides async implementations for package management operations
//! that can be awaited directly instead of using blocking tasks.

use crate::models::{Package, PackageSource};
use crate::errors::{AppError, Result};
use std::process::Command;
use serde;

/// Service for pacman-related operations
pub struct PacmanService;

impl PacmanService {
    /// Asynchronously searches for packages in pacman repositories
    pub async fn search(query: String) -> Result<Vec<Package>> {
        tokio::task::spawn_blocking(move || {
            let output = Command::new("pacman")
                .arg("-Ss")
                .arg(&query)
                .output()
                .map_err(|e| AppError::Pacman(format!("Failed to execute pacman search: {}", e)))?;

            if !output.status.success() {
                return Err(AppError::Pacman(format!("pacman search failed with status: {}", output.status)));
            }

            let stdout = String::from_utf8(output.stdout)
                .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in pacman output: {}", e)))?;

            let mut packages = Vec::new();
            let mut lines = stdout.lines();

            while let Some(header) = lines.next() {
                if let Some(desc) = lines.next() {
                    if let Some(pkg) = parse_pacman_entry(header, desc) {
                        packages.push(pkg);
                    }
                }
            }

            Ok(packages)
        }).await.map_err(|e| AppError::Other(format!("Join error: {}", e)))?
    }

    /// Asynchronously checks if a package is installed
    pub async fn is_installed(pkg_name: String) -> bool {
        tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .arg("-Qi")
                .arg(&pkg_name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }).await.unwrap_or(false)
    }

    /// Asynchronously checks for available updates
    pub async fn check_updates() -> Result<usize> {
        tokio::task::spawn_blocking(move || {
            // Try checkupdates first (from pacman-contrib)
            if let Ok(output) = Command::new("checkupdates").output() {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)
                        .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in checkupdates output: {}", e)))?;
                    return Ok(stdout.lines().count());
                }
            }

            // Fallback to pacman -Qu (checks against local DB, which might be stale but better than nothing if checkupdates missing)
            let output = Command::new("pacman")
                .arg("-Qu")
                .output()
                .map_err(|e| AppError::Pacman(format!("Failed to execute pacman -Qu: {}", e)))?;

            if output.status.success() {
                 let stdout = String::from_utf8(output.stdout)
                     .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in pacman -Qu output: {}", e)))?;
                 return Ok(stdout.lines().count());
            }

            // If it fails (e.g. no updates or error), return 0
            Ok(0)
        }).await.map_err(|e| AppError::Other(format!("Join error: {}", e)))?
    }
}

/// Helper function to parse pacman entries (moved from pacman module)
fn parse_pacman_entry(header: &str, desc: &str) -> Option<Package> {
    // Header format: repo/name version (groups) [installed]
    // Example: core/linux 6.6.1-arch1 (base) [installed]

    let parts: Vec<&str> = header.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let full_name = parts[0]; // repo/name
    let version = parts[1];
    let is_installed = header.contains("[installed]");

    let name = full_name.split('/').nth(1).unwrap_or(full_name).to_string();

    Some(Package {
        name,
        version: version.to_string(),
        description: desc.trim().to_string(),
        source: PackageSource::Pacman,
        is_installed,
        installed_size: None,
        download_size: None,
        groups: vec![],
        licenses: vec![],
        maintainers: vec![],
        keywords: vec![],
        url: None,
        depends_on: vec![],
        required_by: vec![],
        opt_depends: vec![],
        conflicts: vec![],
        replaces: vec![],
        provides: vec![],
    })
}

/// Service for AUR-related operations
pub struct AurService;

impl AurService {
    /// Asynchronously searches for packages in AUR
    pub async fn search(query: &str) -> Result<Vec<Package>> {
        let client = reqwest::Client::new();
        let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", query);

        let response = client.get(&url)
            .header("User-Agent", "arch-tui")
            .send()
            .await
            .map_err(|e| AppError::Aur(format!("Failed to send AUR request: {}", e)))?;

        let aur_response: AurResponse = response
            .json()
            .await
            .map_err(|e| AppError::Aur(format!("Failed to parse AUR response: {}", e)))?;

        let packages = aur_response.results.into_iter().map(|aur_pkg| {
            let mut all_deps = Vec::new();
            if let Some(depends) = aur_pkg.depends_on {
                all_deps.extend(depends);
            }
            if let Some(make_depends) = aur_pkg.make_depends {
                all_deps.extend(make_depends);
            }

            Package {
                name: aur_pkg.name,
                version: aur_pkg.version,
                description: aur_pkg.description.unwrap_or_default(),
                source: PackageSource::Aur,
                is_installed: false,
                installed_size: None,
                download_size: None,
                groups: vec![],
                licenses: aur_pkg.licenses.unwrap_or_default(),
                maintainers: aur_pkg.maintainer.map(|m| vec![m]).unwrap_or_default(),
                keywords: aur_pkg.keywords.unwrap_or_default(),
                url: aur_pkg.url,
                depends_on: all_deps,
                required_by: vec![],
                opt_depends: aur_pkg.opt_depends.unwrap_or_default(),
                conflicts: aur_pkg.conflicts.unwrap_or_default(),
                replaces: vec![],
                provides: aur_pkg.provides.unwrap_or_default(),
            }
        }).collect();

        Ok(packages)
    }
}

#[derive(serde::Deserialize, Debug)]
struct AurResponse {
    results: Vec<AurPackage>,
}

#[derive(serde::Deserialize, Debug)]
struct AurPackage {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "URL")]
    url: Option<String>,
    #[serde(rename = "Maintainer")]
    maintainer: Option<String>,
    #[serde(rename = "DependsOn")]
    depends_on: Option<Vec<String>>,
    #[serde(rename = "MakeDepends")]
    make_depends: Option<Vec<String>>,
    #[serde(rename = "OptDepends")]
    opt_depends: Option<Vec<String>>,
    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<String>>,
    #[serde(rename = "Licenses")]
    licenses: Option<Vec<String>>,
    #[serde(rename = "Keywords")]
    keywords: Option<Vec<String>>,
    #[serde(rename = "Provides")]
    provides: Option<Vec<String>>,
}