//! Universal Package Manager Service
//!
//! This module provides a unified interface for package management
//! across different operating systems and package managers.

use async_trait::async_trait;
use std::sync::Arc;

pub mod apk;
pub mod apt;
pub mod brew;
pub mod cargo_backend;
pub mod chocolatey;
pub mod dnf;
pub mod flatpak;
pub mod npm;
pub mod pacman;
pub mod pip;
pub mod scoop;
pub mod snap;
pub mod snapshots;
pub mod winget;
pub mod zypper;

use crate::errors::Result;
use crate::models::{OutdatedPackage, Package, PackageSource};
use crate::platform::PackageManager;

/// Unified package manager trait
#[async_trait]
pub trait UniversalPackageManager: Send + Sync {
    /// Get the package manager name
    fn get_name(&self) -> PackageManager;

    /// Search for packages
    async fn search(&self, query: &str) -> Result<Vec<Package>>;

    /// Check if a package is installed
    async fn is_installed(&self, pkg_name: &str) -> bool;

    /// Get list of installed packages
    async fn list_installed(&self) -> Result<Vec<Package>>;

    /// Check for available updates
    async fn check_updates(&self) -> Result<Vec<OutdatedPackage>>;

    /// Build install command
    fn build_install_command(&self, packages: &[&str]) -> CommandSpec;

    /// Build remove command
    fn build_remove_command(&self, packages: &[&str]) -> CommandSpec;

    /// Build update command
    fn build_update_command(&self) -> CommandSpec;
}

/// Command specification for package operations
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub prog: String,
    pub args: Vec<String>,
    pub needs_sudo: bool,
}

impl CommandSpec {
    pub fn new(prog: &str, args: Vec<String>) -> Self {
        Self {
            prog: prog.to_string(),
            args,
            needs_sudo: true,
        }
    }

    pub fn no_sudo(prog: &str, args: Vec<String>) -> Self {
        Self {
            prog: prog.to_string(),
            args,
            needs_sudo: false,
        }
    }
}

/// Universal package service that routes to the appropriate backend
pub struct UniversalPackageService {
    backend: Arc<dyn UniversalPackageManager>,
}

impl UniversalPackageService {
    pub fn new(backend: Arc<dyn UniversalPackageManager>) -> Self {
        Self { backend }
    }

    pub fn from_system() -> Self {
        use crate::platform::detect_package_managers;

        let managers = detect_package_managers();
        let backend: Arc<dyn UniversalPackageManager> = match managers.first() {
            Some(PackageManager::Pacman) => Arc::new(pacman::PacmanBackend::new()),
            Some(PackageManager::Apt) => Arc::new(apt::AptBackend::new()),
            Some(PackageManager::Dnf) => Arc::new(dnf::DnfBackend::new()),
            Some(PackageManager::Zypper) => Arc::new(zypper::ZypperBackend::new()),
            Some(PackageManager::Apk) => Arc::new(apk::ApkBackend::new()),
            Some(PackageManager::Brew) => Arc::new(brew::BrewBackend::new()),
            Some(PackageManager::Winget) => Arc::new(winget::WingetBackend::new()),
            Some(PackageManager::Chocolatey) => Arc::new(chocolatey::ChocolateyBackend::new()),
            Some(PackageManager::Scoop) => Arc::new(scoop::ScoopBackend::new()),
            Some(PackageManager::Flatpak) => Arc::new(flatpak::FlatpakBackend::new()),
            Some(PackageManager::Snap) => Arc::new(snap::SnapBackend::new()),
            _ => Arc::new(pacman::PacmanBackend::new()), // Default to pacman
        };

        Self { backend }
    }

    pub fn get_backend_name(&self) -> PackageManager {
        self.backend.get_name()
    }

    pub async fn search(&self, query: &str) -> Result<Vec<Package>> {
        self.backend.search(query).await
    }

    pub async fn is_installed(&self, pkg_name: &str) -> bool {
        self.backend.is_installed(pkg_name).await
    }

    pub async fn list_installed(&self) -> Result<Vec<Package>> {
        self.backend.list_installed().await
    }

    pub async fn check_updates(&self) -> Result<Vec<OutdatedPackage>> {
        self.backend.check_updates().await
    }

    pub fn build_install_command(&self, packages: &[&str]) -> CommandSpec {
        self.backend.build_install_command(packages)
    }

    pub fn build_remove_command(&self, packages: &[&str]) -> CommandSpec {
        self.backend.build_remove_command(packages)
    }

    pub fn build_update_command(&self) -> CommandSpec {
        self.backend.build_update_command()
    }
}

/// Parse version string to sortable form
pub fn parse_version(version: &str) -> (u64, u64, u64) {
    let parts: Vec<&str> = version.split(['.', '-']).collect();
    let major: u64 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

/// Create a package from a name and version
pub fn create_package(
    name: String,
    version: String,
    description: String,
    source: PackageSource,
) -> Package {
    Package {
        name,
        version,
        description,
        source,
        is_installed: false,
        is_outdated: false,
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
        votes: None,
        popularity: None,
        first_submitted: None,
        last_updated: None,
        package_base_id: None,
        num_votes: None,
    }
}
