//! Service layer for package management operations
//!
//! This module provides async implementations for package management operations
//! following the provider traits pattern for better testability and organization.

use async_trait::async_trait;
use dashmap::DashMap;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use crate::config::AppConfig;
use crate::errors::{AppError, Result};
use crate::models::{Package, PackageSource};
use crate::traits::{PackageProvider, UpdateProvider};

lazy_static::lazy_static! {
    /// Cache search results to avoid repeated queries
    static ref PACKAGE_CACHE: DashMap<String, CachedSearch> = DashMap::new();
}

/// Cached search entry with timestamp
#[derive(Clone)]
struct CachedSearch {
    packages: Vec<Package>,
    cached_at: std::time::Instant,
}

impl CachedSearch {
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
                return Ok(cached.packages.clone());
            }
        }

        let query = query.to_string();
        tokio::task::spawn_blocking(move || Self::search_blocking(&query))
            .await
            .map_err(|e| AppError::Other(format!("Join error: {}", e)))?
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
        })
        .await
        .unwrap_or(false)
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
            return Err(AppError::Pacman(format!(
                "pacman search failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| AppError::Pacman(format!("Invalid UTF-8 in pacman output: {}", e)))?;

        let mut packages = Vec::new();
        let mut lines = stdout.lines();

        while let Some(header) = lines.next() {
            if let Some(desc) = lines.next() {
                if let Some(pkg) = Self::parse_entry(header, desc) {
                    packages.push(pkg);
                }
            }
        }

        let cache_key = format!("pacman:{}", query);
        PACKAGE_CACHE.insert(
            cache_key,
            CachedSearch {
                packages: packages.clone(),
                cached_at: std::time::Instant::now(),
            },
        );

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
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .connect_timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
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
                return Ok(cached.packages.clone());
            }
        }

        let url = format!(
            "https://aur.archlinux.org/rpc/v5/search/{}",
            urlencoding::encode(query)
        );

        const MAX_RETRIES: usize = 3;
        let mut response = None;
        let mut last_error = None;
        for attempt in 0..MAX_RETRIES {
            match self
                .client
                .get(&url)
                .header("User-Agent", "arch-tui/0.1.0")
                .send()
                .await
            {
                Ok(resp) => {
                    response = Some(resp);
                    break;
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    if attempt + 1 < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_millis(250 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        let response = response.ok_or_else(|| {
            AppError::Aur(format!(
                "Failed to send AUR request after {} attempts: {}",
                MAX_RETRIES,
                last_error.unwrap_or_else(|| "unknown error".to_string())
            ))
        })?;

        if !response.status().is_success() {
            return Err(AppError::Aur(format!(
                "AUR request failed with status {}",
                response.status()
            )));
        }

        let aur_response: AurResponse = response
            .json()
            .await
            .map_err(|e| AppError::Aur(format!("Failed to parse AUR response: {}", e)))?;

        let packages: Vec<Package> = aur_response
            .results
            .into_iter()
            .map(|aur_pkg| {
                let mut all_deps = Vec::new();
                if let Some(depends) = aur_pkg.depends {
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
                }
            })
            .collect();

        PACKAGE_CACHE.insert(
            cache_key,
            CachedSearch {
                packages: packages.clone(),
                cached_at: std::time::Instant::now(),
            },
        );

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
        })
        .await
        .unwrap_or(false)
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
                    let stdout = String::from_utf8(output.stdout).map_err(|e| {
                        AppError::Pacman(format!("Invalid UTF-8 in checkupdates output: {}", e))
                    })?;
                    return Ok(stdout.lines().filter(|l| !l.is_empty()).count());
                }
            }

            // Fallback to pacman -Qu (checks against local DB)
            let output = Command::new("pacman")
                .arg("-Qu")
                .output()
                .map_err(|e| AppError::Pacman(format!("Failed to execute pacman -Qu: {}", e)))?;

            if output.status.success() {
                let stdout = String::from_utf8(output.stdout).map_err(|e| {
                    AppError::Pacman(format!("Invalid UTF-8 in pacman -Qu output: {}", e))
                })?;
                return Ok(stdout.lines().filter(|l| !l.is_empty()).count());
            }

            Ok(0)
        })
        .await
        .map_err(|e| AppError::Other(format!("Join error: {}", e)))?
    }
}

/// Package service that orchestrates multiple providers
pub struct PackageService {
    providers: Vec<Arc<dyn PackageProvider>>,
    update_provider: Arc<dyn UpdateProvider>,
}

#[derive(Debug, Clone, PartialEq)]
enum QuerySourceFilter {
    Any,
    Repo,
    Aur,
}

#[derive(Debug, Clone)]
struct QuerySpec {
    text: String,
    source: QuerySourceFilter,
    installed: Option<bool>,
    name_prefix: Option<String>,
    name_contains: Option<String>,
    dependency_contains: Option<String>,
}

impl QuerySpec {
    fn parse(input: &str) -> Self {
        let mut text_tokens = Vec::new();
        let mut spec = Self {
            text: String::new(),
            source: QuerySourceFilter::Any,
            installed: None,
            name_prefix: None,
            name_contains: None,
            dependency_contains: None,
        };

        for token in input.split_whitespace() {
            if let Some((k, v)) = token.split_once(':') {
                match k {
                    "source" => {
                        spec.source = match v {
                            "aur" => QuerySourceFilter::Aur,
                            "repo" | "pacman" => QuerySourceFilter::Repo,
                            _ => QuerySourceFilter::Any,
                        };
                    }
                    "installed" => match v {
                        "true" | "yes" | "1" => spec.installed = Some(true),
                        "false" | "no" | "0" => spec.installed = Some(false),
                        _ => {}
                    },
                    "name" => {
                        if let Some(rest) = v.strip_prefix('^') {
                            if !rest.is_empty() {
                                spec.name_prefix = Some(rest.to_lowercase());
                            }
                        } else if !v.is_empty() {
                            spec.name_contains = Some(v.to_lowercase());
                        }
                    }
                    "dep" | "depends" => {
                        if !v.is_empty() {
                            spec.dependency_contains = Some(v.to_lowercase());
                        }
                    }
                    _ => text_tokens.push(token.to_string()),
                }
            } else {
                text_tokens.push(token.to_string());
            }
        }

        spec.text = text_tokens.join(" ").trim().to_string();
        spec
    }

    fn matches(&self, pkg: &Package) -> bool {
        if self.source == QuerySourceFilter::Aur && !matches!(pkg.source, PackageSource::Aur) {
            return false;
        }
        if self.source == QuerySourceFilter::Repo && !matches!(pkg.source, PackageSource::Pacman) {
            return false;
        }
        if let Some(installed) = self.installed {
            if pkg.is_installed != installed {
                return false;
            }
        }
        if let Some(prefix) = &self.name_prefix {
            if !pkg.name.to_lowercase().starts_with(prefix) {
                return false;
            }
        }
        if let Some(contains) = &self.name_contains {
            if !pkg.name.to_lowercase().contains(contains) {
                return false;
            }
        }
        if let Some(dep) = &self.dependency_contains {
            let hit = pkg
                .depends_on
                .iter()
                .any(|d| d.to_lowercase().contains(dep));
            if !hit {
                return false;
            }
        }
        true
    }
}

pub fn command_display(spec: &CommandSpec) -> String {
    format!("{} {}", spec.prog, spec.args.join(" "))
}

pub fn plan_package_transaction(packages: &[Package], config: &AppConfig) -> Vec<CommandSpec> {
    let (removes, installs): (Vec<_>, Vec<_>) = packages.iter().partition(|p| p.is_installed);
    let helper = AurHelperCommand::new(config);
    let mut commands = Vec::new();

    if !removes.is_empty() {
        let names: Vec<&str> = removes.iter().map(|p| p.name.as_str()).collect();
        commands.push(helper.remove_command(&names));
    }

    if !installs.is_empty() {
        let names: Vec<&str> = installs.iter().map(|p| p.name.as_str()).collect();
        let use_aur_helper = installs
            .iter()
            .any(|p| matches!(p.source, PackageSource::Aur));
        if use_aur_helper {
            commands.push(helper.install_command(&names));
        } else {
            let mut args = vec![
                "pacman".to_string(),
                "-S".to_string(),
                "--noconfirm".to_string(),
            ];
            args.extend(names.into_iter().map(|n| n.to_string()));
            commands.push(CommandSpec {
                prog: "sudo".to_string(),
                args,
            });
        }
    }

    commands
}

pub fn plan_rollback_transaction(
    originally_installed: &[String],
    originally_removed: &[String],
    config: &AppConfig,
) -> Vec<CommandSpec> {
    let helper = AurHelperCommand::new(config);
    let mut commands = Vec::new();

    // Undo installs by removing them
    if !originally_installed.is_empty() {
        let names: Vec<&str> = originally_installed.iter().map(String::as_str).collect();
        commands.push(helper.remove_command(&names));
    }

    // Undo removals by re-installing them
    if !originally_removed.is_empty() {
        let names: Vec<&str> = originally_removed.iter().map(String::as_str).collect();
        commands.push(helper.install_command(&names));
    }

    commands
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
        let spec = QuerySpec::parse(query);
        if spec.text.trim().is_empty()
            && spec.installed.is_none()
            && spec.source == QuerySourceFilter::Any
            && spec.name_prefix.is_none()
            && spec.name_contains.is_none()
            && spec.dependency_contains.is_none()
        {
            return Ok(Vec::new());
        }

        let base_query = if spec.text.is_empty() {
            query.trim()
        } else {
            spec.text.as_str()
        };
        let mut all_results = Vec::new();
        let mut tasks = Vec::new();

        for provider in &self.providers {
            let provider = Arc::clone(provider);
            let query = base_query.to_string();
            let task = tokio::spawn(async move { provider.search(&query).await });
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

        // Deduplicate by source+name and keep richer entries if duplicates exist.
        let mut deduped: HashMap<(String, String), Package> = HashMap::new();
        for pkg in all_results {
            let source = match pkg.source {
                PackageSource::Pacman => "pacman".to_string(),
                PackageSource::Aur => "aur".to_string(),
            };
            let key = (source, pkg.name.clone());
            deduped
                .entry(key)
                .and_modify(|existing| {
                    if !existing.is_installed && pkg.is_installed {
                        existing.is_installed = true;
                    }
                    if existing.description.is_empty() && !pkg.description.is_empty() {
                        existing.description = pkg.description.clone();
                    }
                })
                .or_insert(pkg);
        }

        let query_lc = base_query.to_lowercase();
        let mut results: Vec<Package> = deduped.into_values().filter(|p| spec.matches(p)).collect();
        results.sort_by(|a, b| {
            let a_name = a.name.to_lowercase();
            let b_name = b.name.to_lowercase();
            let a_rank = relevance_rank(&a_name, &query_lc);
            let b_rank = relevance_rank(&b_name, &query_lc);
            a_rank
                .cmp(&b_rank)
                .then_with(|| a_name.cmp(&b_name))
                .then_with(|| match (&a.source, &b.source) {
                    (PackageSource::Pacman, PackageSource::Aur) => std::cmp::Ordering::Less,
                    (PackageSource::Aur, PackageSource::Pacman) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                })
        });

        Ok(results)
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<usize> {
        self.update_provider.check_updates().await
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

fn relevance_rank(name: &str, query: &str) -> u8 {
    if name == query {
        0
    } else if name.starts_with(query) {
        1
    } else if name.contains(query) {
        2
    } else {
        3
    }
}

// AUR Response structures
#[derive(serde::Deserialize, Debug)]
struct AurResponse {
    #[serde(rename = "resultcount")]
    _result_count: u32,
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

    /// Sanitize input to prevent command injection
    fn sanitize(input: &str) -> String {
        // Remove dangerous characters that could be used for injection
        let pattern = r##"[;&|<>$`"\n\r\x00]"##;
        match Regex::new(pattern) {
            Ok(dangerous) => dangerous.replace_all(input, "").to_string(),
            Err(_) => input.to_string(),
        }
    }
}

/// Structured command specification (program + arguments)
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub prog: String,
    pub args: Vec<String>,
}

impl CommandSpec {
    fn new(prog: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            prog: prog.into(),
            args,
        }
    }
}

/// AUR helper command builder
pub struct AurHelperCommand {
    helper: HelperKind,
}

#[derive(Debug, Clone, PartialEq)]
enum HelperKind {
    Paru,
    Yay,
    Pacman,
}

impl AurHelperCommand {
    const NOCONFIRM: &'static str = "--noconfirm";

    pub fn new(config: &AppConfig) -> Self {
        let helper = Self::detect_helper(&config.aur_helper);
        Self { helper }
    }

    fn detect_helper(configured: &str) -> HelperKind {
        match configured {
            "paru" if Self::command_exists("paru") => HelperKind::Paru,
            "yay" if Self::command_exists("yay") => HelperKind::Yay,
            "auto" | _ => {
                if Self::command_exists("paru") {
                    HelperKind::Paru
                } else if Self::command_exists("yay") {
                    HelperKind::Yay
                } else {
                    HelperKind::Pacman
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
    pub fn install_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec!["-S".to_string(), Self::NOCONFIRM.to_string()];
        args.extend(packages.iter().map(|p| p.to_string()));

        match self.helper {
            HelperKind::Paru => CommandSpec::new("paru", args),
            HelperKind::Yay => CommandSpec::new("yay", args),
            HelperKind::Pacman => {
                let mut pacman_args = vec!["pacman".to_string()];
                pacman_args.extend(args);
                CommandSpec::new("sudo", pacman_args)
            }
        }
    }

    /// Build remove command
    pub fn remove_command(&self, packages: &[&str]) -> CommandSpec {
        let mut args = vec![
            "pacman".to_string(),
            "-Rns".to_string(),
            Self::NOCONFIRM.to_string(),
        ];
        args.extend(packages.iter().map(|p| p.to_string()));
        CommandSpec::new("sudo", args)
    }

    /// Build update command
    pub fn update_command(&self) -> CommandSpec {
        let args = vec!["-Syu".to_string(), Self::NOCONFIRM.to_string()];
        match self.helper {
            HelperKind::Paru => CommandSpec::new("paru", args),
            HelperKind::Yay => CommandSpec::new("yay", args),
            HelperKind::Pacman => {
                let mut pacman_args = vec!["pacman".to_string()];
                pacman_args.extend(args);
                CommandSpec::new("sudo", pacman_args)
            }
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
        assert_eq!(
            SafeCommandBuilder::sanitize("test; rm -rf /"),
            "test rm -rf /"
        );
        assert_eq!(
            SafeCommandBuilder::sanitize("test|cat /etc/passwd"),
            "testcat /etc/passwd"
        );
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
