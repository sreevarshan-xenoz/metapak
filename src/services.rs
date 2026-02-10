//! Service layer for package management operations
//!
//! This module provides async implementations for package management operations
//! following the provider traits pattern for better testability and organization.

use async_trait::async_trait;
use std::process::Command;
use std::sync::Arc;
use dashmap::DashMap;
use regex::Regex;

use crate::models::{Package, PackageSource};
use crate::errors::{AppError, Result};
use crate::traits::{PackageProvider, UpdateProvider};
use crate::config::AppConfig;

lazy_static::lazy_static! {
    /// Cache for package information to avoid repeated queries
    static ref PACKAGE_CACHE: DashMap<String, CachedPackage> = DashMap::new();
}

/// Cached package entry with timestamp
#[derive(Clone)]
struct CachedPackage {
    package: Package,
    cached_at: std::time::Instant,
}

impl CachedPackage {
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > std::time::Duration::from_secs(300) // 5 minutes
    }
}

/// Pacman package provider implementation
pub struct PacmanProvider;

impl PacmanProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl PackageProvider for PacmanProvider {
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        // Check cache first
        let cache_key = format!("pacman:{}", query);
        if let Some(cached) = PACKAGE_CACHE.get(&cache_key) {
            if !cached.is_expired() {
                tracing::debug!("Cache hit for pacman search: {}", query);
                return Ok(vec![cached.package.clone()]);
            }
        }

        let query = query.to_string();
        tokio::task::spawn_blocking(move || {
            Self::search_blocking(&query)
        }).await.map_err(|e| AppError::Other(format!("Join error: {}", e)))?
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        let pkg_name = pkg_name.to_string();
        tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .arg("-Qi")
                .arg(&pkg_name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }).await.unwrap_or(false)
    }

    fn source(&self) -> PackageSource {
        PackageSource::Pacman
    }

    fn name(&self) -> &'static str {
        "pacman"
    }
}

impl PacmanProvider {
    /// Blocking search implementation
    fn search_blocking(query: &str) -> Result<Vec<Package>> {
        let output = Command::new("pacman")
            .arg("-Ss")
            .arg(query)
            .output()
            .map_err(|e| AppError::Pacman(format!("Failed to execute pacman search: {}", e)))?;

        if !output.status.success() {
            // Check if it's just no results vs an actual error
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no results") || output.status.code() == Some(1) {
                return Ok(Vec::new());
            }
            return Err(AppError::Pacman(format!("pacman search failed: {}", stderr)));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in pacman output: {}", e)))?;

        let mut packages = Vec::new();
        let mut lines = stdout.lines();

        while let Some(header) = lines.next() {
            if let Some(desc) = lines.next() {
                if let Some(pkg) = Self::parse_entry(header, desc) {
                    // Cache the package
                    let cache_key = format!("pacman:{}", pkg.name);
                    PACKAGE_CACHE.insert(cache_key, CachedPackage {
                        package: pkg.clone(),
                        cached_at: std::time::Instant::now(),
                    });
                    packages.push(pkg);
                }
            }
        }

        Ok(packages)
    }

    /// Parse a pacman package entry from command output
    fn parse_entry(header: &str, desc: &str) -> Option<Package> {
        // Header format: repo/name version (groups) [installed]
        // Example: core/linux 6.6.1-arch1 (base) [installed]
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let full_name = parts[0]; // repo/name
        let version = parts[1];
        let is_installed = header.contains("[installed]") || header.contains("[Installed]");

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
}

/// AUR package provider implementation
pub struct AurProvider {
    client: reqwest::Client,
}

impl AurProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PackageProvider for AurProvider {
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        // Check cache first
        let cache_key = format!("aur:{}", query);
        if let Some(cached) = PACKAGE_CACHE.get(&cache_key) {
            if !cached.is_expired() {
                tracing::debug!("Cache hit for AUR search: {}", query);
                return Ok(vec![cached.package.clone()]);
            }
        }

        let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", 
            urlencoding::encode(query));

        let response = self.client.get(&url)
            .header("User-Agent", "arch-tui/0.1.0")
            .send()
            .await
            .map_err(|e| AppError::Aur(format!("Failed to send AUR request: {}", e)))?;

        let aur_response: AurResponse = response
            .json()
            .await
            .map_err(|e| AppError::Aur(format!("Failed to parse AUR response: {}", e)))?;

        let packages: Vec<Package> = aur_response.results.into_iter()
            .map(|aur_pkg| {
                let mut all_deps = Vec::new();
                if let Some(depends) = aur_pkg.depends {
                    all_deps.extend(depends);
                }
                if let Some(make_depends) = aur_pkg.make_depends {
                    all_deps.extend(make_depends);
                }

                let pkg = Package {
                    name: aur_pkg.name,
                    version: aur_pkg.version,
                    description: aur_pkg.description.unwrap_or_default(),
                    source: PackageSource::Aur,
                    is_installed: false, // Will be updated later
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
                };

                // Cache the package
                let cache_key = format!("aur:{}", pkg.name);
                PACKAGE_CACHE.insert(cache_key, CachedPackage {
                    package: pkg.clone(),
                    cached_at: std::time::Instant::now(),
                });

                pkg
            })
            .collect();

        Ok(packages)
    }

    async fn is_installed(&self, pkg_name: &str) -> bool {
        // AUR packages are tracked by pacman once installed
        let pkg_name = pkg_name.to_string();
        tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .arg("-Qm")
                .arg(&pkg_name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }).await.unwrap_or(false)
    }

    fn source(&self) -> PackageSource {
        PackageSource::Aur
    }

    fn name(&self) -> &'static str {
        "aur"
    }
}

/// Update provider implementation
pub struct SystemUpdateProvider;

impl SystemUpdateProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl UpdateProvider for SystemUpdateProvider {
    async fn check_updates(&self) -> Result<usize> {
        tokio::task::spawn_blocking(move || {
            // Try checkupdates first (from pacman-contrib) - doesn't require sudo
            if let Ok(output) = Command::new("checkupdates").output() {
                if output.status.success() {
                    let stdout = String::from_utf8(output.stdout)
                        .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in checkupdates output: {}", e)))?;
                    return Ok(stdout.lines().filter(|l| !l.is_empty()).count());
                }
            }

            // Fallback to pacman -Qu (checks against local DB)
            let output = Command::new("pacman")
                .arg("-Qu")
                .output()
                .map_err(|e| AppError::Pacman(format!("Failed to execute pacman -Qu: {}", e)))?;

            if output.status.success() {
                let stdout = String::from_utf8(output.stdout)
                    .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in pacman -Qu output: {}", e)))?;
                return Ok(stdout.lines().filter(|l| !l.is_empty()).count());
            }

            Ok(0)
        }).await.map_err(|e| AppError::Other(format!("Join error: {}", e)))?
    }

    async fn update_system(&self) -> Result<()> {
        // This is handled by the command execution system
        Ok(())
    }
}

/// Package service that orchestrates multiple providers
pub struct PackageService {
    providers: Vec<Arc<dyn PackageProvider>>,
    update_provider: Arc<dyn UpdateProvider>,
}

impl PackageService {
    pub fn new() -> Self {
        Self {
            providers: vec![
                Arc::new(PacmanProvider::new()),
                Arc::new(AurProvider::new()),
            ],
            update_provider: Arc::new(SystemUpdateProvider::new()),
        }
    }

    /// Search across all providers concurrently
    pub async fn search_all(&self, query: &str) -> Result<Vec<Package>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut all_results = Vec::new();
        let mut tasks = Vec::new();

        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let query = query.to_string();
            let task = tokio::spawn(async move {
                provider.search(&query).await
            });
            tasks.push(task);
        }

        for task in tasks {
            match task.await {
                Ok(Ok(packages)) => all_results.extend(packages),
                Ok(Err(e)) => {
                    tracing::warn!("Provider search failed: {}", e);
                }
                Err(e) => {
                    tracing::error!("Task join error: {}", e);
                }
            }
        }

        // Update installation status for AUR packages
        let pacman = PacmanProvider::new();
        for pkg in &mut all_results {
            if pkg.source == PackageSource::Aur && !pkg.is_installed {
                pkg.is_installed = pacman.is_installed(&pkg.name).await;
            }
        }

        Ok(all_results)
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<usize> {
        self.update_provider.check_updates().await
    }

    /// Get package cache stats
    pub fn cache_stats(&self) -> (usize, usize) {
        let total = PACKAGE_CACHE.len();
        let expired = PACKAGE_CACHE.iter()
            .filter(|entry| entry.value().is_expired())
            .count();
        (total, expired)
    }

    /// Clear expired cache entries
    pub fn clear_expired_cache() {
        PACKAGE_CACHE.retain(|_, v| !v.is_expired());
    }
}

impl Default for PackageService {
    fn default() -> Self {
        Self::new()
    }
}

// AUR Response structures
#[derive(serde::Deserialize, Debug)]
struct AurResponse {
    #[serde(rename = "resultcount")]
    result_count: u32,
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
    #[serde(rename = "Depends")]
    depends: Option<Vec<String>>,
    #[serde(rename = "MakeDepends")]
    make_depends: Option<Vec<String>>,
    #[serde(rename = "OptDepends")]
    opt_depends: Option<Vec<String>>,
    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<String>>,
    #[serde(rename = "License")]
    licenses: Option<Vec<String>>,
    #[serde(rename = "Keywords")]
    keywords: Option<Vec<String>>,
    #[serde(rename = "Provides")]
    provides: Option<Vec<String>>,
}

/// Command builder for safe command execution
pub struct SafeCommandBuilder {
    program: String,
    args: Vec<String>,
}

impl SafeCommandBuilder {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            args: Vec::new(),
        }
    }

    /// Sanitize and add an argument
    pub fn arg(mut self, arg: &str) -> Self {
        let sanitized = Self::sanitize(arg);
        self.args.push(sanitized);
        self
    }

    /// Sanitize and add multiple arguments
    pub fn args(mut self, args: &[&str]) -> Self {
        for arg in args {
            self.args.push(Self::sanitize(arg));
        }
        self
    }

    /// Build the command string for display
    pub fn build_display(&self) -> String {
        format!("{} {}", self.program, self.args.join(" "))
    }

    /// Execute the command
    pub fn execute(&self) -> Result<std::process::Output> {
        Command::new(&self.program)
            .args(&self.args)
            .output()
            .map_err(|e| AppError::Command(format!("Failed to execute '{}': {}", self.program, e)))
    }

    /// Sanitize input to prevent command injection
    fn sanitize(input: &str) -> String {
        // Remove dangerous characters that could be used for injection
        let pattern = r##"[;&|<>$`"\n\r\x00]"##;
        let dangerous = Regex::new(pattern).unwrap();
        dangerous.replace_all(input, "").to_string()
    }
}

/// AUR helper command builder
pub struct AurHelperCommand {
    helper: String,
}

impl AurHelperCommand {
    const SUDO_PACMAN: &'static str = "sudo pacman";
    const NOCONFIRM: &'static str = "--noconfirm";
    
    pub fn new(config: &AppConfig) -> Self {
        let helper = Self::detect_helper(&config.aur_helper);
        Self { helper }
    }

    fn detect_helper(configured: &str) -> String {
        match configured {
            "paru" if Self::command_exists("paru") => "paru".to_string(),
            "yay" if Self::command_exists("yay") => "yay".to_string(),
            "auto" | _ => {
                if Self::command_exists("paru") {
                    "paru".to_string()
                } else if Self::command_exists("yay") {
                    "yay".to_string()
                } else {
                    Self::SUDO_PACMAN.to_string()
                }
            }
        }
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Build install command
    pub fn install_command(&self, packages: &[&str]) -> String {
        let no_confirm = Self::NOCONFIRM;
        if self.helper == Self::SUDO_PACMAN {
            format!("sudo pacman -S {no_confirm} {}", packages.join(" "))
        } else {
            format!("{} -S {no_confirm} {}", self.helper, packages.join(" "))
        }
    }

    /// Build remove command
    pub fn remove_command(&self, packages: &[&str]) -> String {
        let no_confirm = Self::NOCONFIRM;
        format!("sudo pacman -Rns {no_confirm} {}", packages.join(" "))
    }

    /// Build update command
    pub fn update_command(&self) -> String {
        let no_confirm = Self::NOCONFIRM;
        if self.helper == Self::SUDO_PACMAN {
            format!("sudo pacman -Syu {no_confirm}")
        } else {
            format!("{} -Syu {no_confirm}", self.helper)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_input() {
        assert_eq!(SafeCommandBuilder::sanitize("firefox"), "firefox");
        assert_eq!(SafeCommandBuilder::sanitize("rm -rf /"), "rm -rf /");
        assert_eq!(SafeCommandBuilder::sanitize("test; rm -rf /"), "test rm -rf /");
        assert_eq!(SafeCommandBuilder::sanitize("test|cat /etc/passwd"), "testcat /etc/passwd");
        assert_eq!(SafeCommandBuilder::sanitize("test`whoami`"), "testwhoami");
    }

    #[test]
    fn test_parse_pacman_entry() {
        let header = "core/linux 6.6.1-arch1 (base) [installed]";
        let desc = "The Linux kernel and modules";
        
        let pkg = PacmanProvider::parse_entry(header, desc);
        assert!(pkg.is_some());
        
        let pkg = pkg.unwrap();
        assert_eq!(pkg.name, "linux");
        assert_eq!(pkg.version, "6.6.1-arch1");
        assert!(pkg.is_installed);
    }

    #[test]
    fn test_parse_pacman_entry_not_installed() {
        let header = "community/firefox 120.0-1";
        let desc = "Standalone web browser";
        
        let pkg = PacmanProvider::parse_entry(header, desc);
        assert!(pkg.is_some());
        assert!(!pkg.unwrap().is_installed);
    }

    #[test]
    fn test_safe_command_builder() {
        let cmd = SafeCommandBuilder::new("pacman")
            .arg("-S")
            .args(&["firefox", "vlc"]);
        
        assert_eq!(cmd.build_display(), "pacman -S firefox vlc");
    }
}
