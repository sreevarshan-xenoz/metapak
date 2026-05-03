//! Service layer for package management operations
//!
//! This module provides async implementations for package management operations
//! following the provider traits pattern for better testability and organization.

use async_trait::async_trait;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use crate::config::AppConfig;
use crate::errors::{AppError, Result};
use crate::models::{Package, PackageSource, OutdatedPackage};
use crate::search::EnhancedSearch;
use crate::traits::{PackageProvider, UpdateProvider};

/// Cache search results to avoid repeated queries
static PACKAGE_CACHE: Lazy<DashMap<String, CachedSearch>> = Lazy::new(|| DashMap::new());

/// Configurable cache TTL in seconds
static CACHE_TTL_SECS: Lazy<std::sync::atomic::AtomicU64> =
    Lazy::new(|| std::sync::atomic::AtomicU64::new(300));

/// Circuit breaker for AUR API
static AUR_CIRCUIT_BREAKER: Lazy<CircuitBreaker> = Lazy::new(|| CircuitBreaker::new());

/// Circuit breaker for AUR API calls to prevent flooding when service is down
pub struct CircuitBreaker {
    failure_count: std::sync::atomic::AtomicU32,
    last_failure: std::sync::atomic::AtomicU64,
    state: std::sync::atomic::AtomicU32,
}

impl CircuitBreaker {
    pub const FAILURE_THRESHOLD: u32 = 5;
    pub const RECOVERY_SECS: u64 = 30;
    pub const STATE_CLOSED: u32 = 0;
    pub const STATE_OPEN: u32 = 1;
    pub const STATE_HALF_OPEN: u32 = 2;

    pub fn new() -> Self {
        Self {
            failure_count: std::sync::atomic::AtomicU32::new(0),
            last_failure: std::sync::atomic::AtomicU64::new(0),
            state: std::sync::atomic::AtomicU32::new(Self::STATE_CLOSED),
        }
    }

    pub fn is_available(&self) -> bool {
        let state = self.state.load(std::sync::atomic::Ordering::SeqCst);
        match state {
            Self::STATE_CLOSED => true,
            Self::STATE_OPEN => {
                let last_fail = self.last_failure.load(std::sync::atomic::Ordering::SeqCst);
                let elapsed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
                    - last_fail;
                if elapsed > Self::RECOVERY_SECS {
                    self.state.store(Self::STATE_HALF_OPEN, std::sync::atomic::Ordering::SeqCst);
                    true
                } else {
                    false
                }
            }
            Self::STATE_HALF_OPEN => true,
            _ => false,
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, std::sync::atomic::Ordering::SeqCst);
        self.state.store(Self::STATE_CLOSED, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.last_failure.store(now, std::sync::atomic::Ordering::SeqCst);

        if count >= Self::FAILURE_THRESHOLD {
            self.state.store(Self::STATE_OPEN, std::sync::atomic::Ordering::SeqCst);
            tracing::warn!("Circuit breaker opened due to {} consecutive failures", count);
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the current cache TTL
pub fn get_cache_ttl() -> u64 {
    CACHE_TTL_SECS.load(std::sync::atomic::Ordering::SeqCst)
}

/// Set the cache TTL (in seconds)
pub fn set_cache_ttl(seconds: u64) {
    CACHE_TTL_SECS.store(seconds, std::sync::atomic::Ordering::SeqCst);
}

/// Clear the search cache
pub fn clear_cache() {
    PACKAGE_CACHE.clear();
}

/// Get cache statistics
pub fn get_cache_stats() -> (usize, usize) {
    let total = PACKAGE_CACHE.len();
    let expired = PACKAGE_CACHE.iter().filter(|r| r.is_expired()).count();
    (total, expired)
}

/// Enforce cache size limit to prevent memory exhaustion
pub fn enforce_cache_limit() {
    use crate::constants::cache::{CLEANUP_BATCH_SIZE, MAX_CACHE_ENTRIES};

    if PACKAGE_CACHE.len() > MAX_CACHE_ENTRIES {
        tracing::warn!("Cache size {} exceeds limit {}, cleaning up", PACKAGE_CACHE.len(), MAX_CACHE_ENTRIES);

        // Remove expired entries first
        let keys_to_remove: Vec<String> = PACKAGE_CACHE
            .iter()
            .filter(|r| r.is_expired())
            .take(CLEANUP_BATCH_SIZE)
            .map(|r| r.key().to_string())
            .collect();

        for key in keys_to_remove {
            PACKAGE_CACHE.remove(&key);
        }

        // If still over limit, remove oldest entries
        if PACKAGE_CACHE.len() > MAX_CACHE_ENTRIES {
            // Note: DashMap doesn't maintain insertion order, so we just clear expired again
            let remaining_keys: Vec<String> = PACKAGE_CACHE
                .iter()
                .take(PACKAGE_CACHE.len() - MAX_CACHE_ENTRIES + CLEANUP_BATCH_SIZE)
                .map(|r| r.key().to_string())
                .collect();

            for key in remaining_keys {
                PACKAGE_CACHE.remove(&key);
            }
        }
    }
}

/// Cached search entry with timestamp
#[derive(Clone)]
struct CachedSearch {
    packages: Vec<Package>,
    cached_at: std::time::Instant,
}

impl CachedSearch {
    fn is_expired(&self) -> bool {
        let ttl = CACHE_TTL_SECS.load(std::sync::atomic::Ordering::SeqCst);
        self.cached_at.elapsed() > std::time::Duration::from_secs(ttl)
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
    #[must_use = "this async method should be .await'd"]
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

    #[must_use = "this async method should be .await'd"]
    async fn is_installed(&self, pkg_name: &str) -> bool {
        let pkg_name = pkg_name.to_string();
        match tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .arg("-Qi")
                .arg(&pkg_name)
                .output()
                .map(|o| o.status.success())
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                tracing::warn!("Failed to check if package is installed: {}", e);
                false
            }
            Err(e) => {
                tracing::warn!("Failed to join is_installed task: {}", e);
                false
            }
        }
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
        })
    }
}

/// AUR package provider implementation
pub struct AurProvider {
    client: reqwest::Client,
}

impl AurProvider {
    pub fn new() -> Self {
        use crate::constants::network::{AUR_CONNECT_TIMEOUT_SECS, AUR_REQUEST_TIMEOUT_SECS, HTTP_IDLE_TIMEOUT_SECS, HTTP_MAX_CONNECTIONS};

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(AUR_REQUEST_TIMEOUT_SECS))
            .connect_timeout(Duration::from_secs(AUR_CONNECT_TIMEOUT_SECS))
            .pool_max_idle_per_host(HTTP_MAX_CONNECTIONS as usize)
            .pool_idle_timeout(Duration::from_secs(HTTP_IDLE_TIMEOUT_SECS))
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to create optimized HTTP client: {}, using default", e);
                reqwest::Client::new()
            });
        Self { client }
    }
}

#[async_trait]
impl PackageProvider for AurProvider {
    #[must_use = "this async method should be .await'd"]
    async fn search(&self, query: &str) -> Result<Vec<Package>> {
        // Check circuit breaker first
        if !AUR_CIRCUIT_BREAKER.is_available() {
            tracing::warn!("AUR circuit breaker is open, skipping request");
            return Err(AppError::Aur("AUR service temporarily unavailable (circuit breaker open)".to_string()));
        }

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

        // Record result in circuit breaker
        if response.is_none() {
            AUR_CIRCUIT_BREAKER.record_failure();
        } else {
            AUR_CIRCUIT_BREAKER.record_success();
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

                let is_outdated = aur_pkg.out_of_date.is_some();
                let package_base_id = aur_pkg.package_base_id.map(|id| id.to_string());

                Package {
                    name: aur_pkg.name,
                    version: aur_pkg.version,
                    description: aur_pkg.description.unwrap_or_default(),
                    source: PackageSource::Aur,
                    is_installed: false,
                    is_outdated,
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
                    votes: aur_pkg.num_votes,
                    popularity: aur_pkg.popularity,
                    first_submitted: aur_pkg.first_submitted,
                    last_updated: aur_pkg.last_updated,
                    package_base_id,
                    num_votes: aur_pkg.num_votes,
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

    #[must_use = "this async method should be .await'd"]
    async fn is_installed(&self, pkg_name: &str) -> bool {
        let pkg_name = pkg_name.to_string();
        match tokio::task::spawn_blocking(move || {
            Command::new("pacman")
                .arg("-Qm")
                .arg(&pkg_name)
                .output()
                .map(|o| o.status.success())
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                tracing::warn!("Failed to check AUR package installation status: {}", e);
                false
            }
            Err(e) => {
                tracing::warn!("Failed to join AUR is_installed task: {}", e);
                false
            }
        }
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
    #[must_use = "this async method should be .await'd"]
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

    async fn get_outdated_packages(&self) -> Result<Vec<OutdatedPackage>> {
        tokio::task::spawn_blocking(move || {
            let mut outdated = Vec::new();

            // Try pacman -Qu first (installed packages with newer versions available)
            let output = Command::new("pacman")
                .arg("-Qu")
                .output()
                .map_err(|e| AppError::Pacman(format!("Failed to execute pacman -Qu: {}", e)))?;

            if output.status.success() {
                let stdout = String::from_utf8(output.stdout).map_err(|e| {
                    AppError::Pacman(format!("Invalid UTF-8 in pacman -Qu output: {}", e))
                })?;

                for line in stdout.lines().filter(|l| !l.is_empty()) {
                    let parts: Vec<&str> = line.splitn(3, ' ').collect();
                    if parts.len() >= 2 {
                        let name = parts[0].to_string();
                        let version = parts[1].to_string();

                        let mut pkg = OutdatedPackage::new(
                            name.clone(),
                            "?".to_string(),
                            version,
                            "unknown".to_string(),
                        );

                        // Get package info
                        if let Ok(info) = Command::new("pacman")
                            .arg("-Qi")
                            .arg(&name)
                            .output()
                        {
                            if info.status.success() {
                                let info_str = String::from_utf8_lossy(&info.stdout);
                                for info_line in info_str.lines() {
                                    if info_line.starts_with("Repository") {
                                        if let Some(repo) = info_line.split(':').nth(1) {
                                            pkg.repository = repo.trim().to_string();
                                            pkg.is_aur = pkg.repository.to_lowercase() == "aur";
                                        }
                                    } else if info_line.starts_with("Installed Size") {
                                        if let Some(size) = info_line.split(':').nth(1) {
                                            let size_str = size.trim();
                                            // Parse size like "150.00 MiB"
                                            let multiplier: u64 = if size_str.contains("GiB") {
                                                1024 * 1024
                                            } else if size_str.contains("MiB") {
                                                1024
                                            } else if size_str.contains("KiB") {
                                                1
                                            } else {
                                                1
                                            };
                                            let num: f64 = size_str
                                                .replace("GiB", "")
                                                .replace("MiB", "")
                                                .replace("KiB", "")
                                                .trim()
                                                .parse()
                                                .unwrap_or(0.0);
                                            pkg.download_size = (num * multiplier as f64) as u64;
                                        }
                                    } else if info_line.starts_with("Depends On") {
                                        let deps = info_line.split(':').nth(1).unwrap_or("");
                                        pkg.new_dependencies = deps
                                            .trim()
                                            .split_whitespace()
                                            .map(|s| s.to_string())
                                            .collect();
                                    }
                                }
                            }
                        }

                        outdated.push(pkg);
                    }
                }
            }

            Ok(outdated)
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
    #[must_use = "this async method should be .await'd"]
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
                PackageSource::Apt => "apt".to_string(),
                PackageSource::Dnf => "dnf".to_string(),
                PackageSource::Zypper => "zypper".to_string(),
                PackageSource::Brew => "brew".to_string(),
                PackageSource::Winget => "winget".to_string(),
                PackageSource::Chocolatey => "chocolatey".to_string(),
                PackageSource::Flatpak => "flatpak".to_string(),
                PackageSource::Snap => "snap".to_string(),
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

        let mut results: Vec<Package> = deduped.into_values().collect();
        
        // Use EnhancedSearch for filtering and ranking
        let search_engine = EnhancedSearch::new();
        let filtered = search_engine.filter_packages(&results, base_query);
        let mut results: Vec<Package> = filtered.into_iter().cloned().collect();
        
        results.sort_by(|a, b| {
            let a_name = a.name.to_lowercase();
            let b_name = b.name.to_lowercase();
            
            let a_score = search_engine.match_with_score(&a_name, base_query).map(|(s, _)| s).unwrap_or(0);
            let b_score = search_engine.match_with_score(&b_name, base_query).map(|(s, _)| s).unwrap_or(0);
            
            b_score.cmp(&a_score) // Higher score first
                .then_with(|| a_name.cmp(&b_name))
                .then_with(|| match (&a.source, &b.source) {
                    (PackageSource::Pacman, PackageSource::Aur) => std::cmp::Ordering::Less,
                    (PackageSource::Aur, PackageSource::Pacman) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                })
        });

        use crate::constants::search_limits::MAX_TOTAL_RESULTS;
        if results.len() > MAX_TOTAL_RESULTS {
            tracing::warn!(
                "Search results truncated from {} to {}",
                results.len(),
                MAX_TOTAL_RESULTS
            );
            results.truncate(MAX_TOTAL_RESULTS);
        }

        Ok(results)
    }

    /// Check for available updates
    pub async fn check_updates(&self) -> Result<usize> {
        self.update_provider.check_updates().await
    }

    /// Get detailed list of outdated packages
    pub async fn get_outdated_packages(&self) -> Result<Vec<OutdatedPackage>> {
        self.update_provider.get_outdated_packages().await
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
    #[serde(rename = "NumVotes")]
    num_votes: Option<i32>,
    #[serde(rename = "Popularity")]
    popularity: Option<f32>,
    #[serde(rename = "LastUpdated")]
    last_updated: Option<i64>,
    #[serde(rename = "FirstSubmitted")]
    first_submitted: Option<i64>,
    #[serde(rename = "OutOfDate")]
    out_of_date: Option<i64>,
    #[serde(rename = "PackageBaseID")]
    package_base_id: Option<i32>,
    #[serde(rename = "PackageBase")]
    package_base: Option<String>,
    #[serde(rename = "Download")]
    download: Option<String>,
    #[serde(rename = "FileSize")]
    file_size: Option<i64>,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct AurInfoResponse {
    #[serde(rename = "results")]
    results: Option<AurPackage>,
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

    fn sanitize_package_name(name: &str) -> String {
        name.chars().filter(|c| c.is_alphanumeric() || "@._+-".contains(*c)).collect()
    }

    fn is_valid_package_name(name: &str) -> bool {
        !name.is_empty() && regex::Regex::new(r"^[a-z0-9@._+-]+$").unwrap().is_match(name)
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
        args.extend(packages.iter().map(|p| Self::sanitize_package_name(p)));

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
        args.extend(packages.iter().map(|p| Self::sanitize_package_name(p)));
        CommandSpec::new("sudo", args)
    }

    /// Build install command with validation
    pub fn install_command_validated(&self, packages: &[&str]) -> std::result::Result<CommandSpec, String> {
        for pkg in packages {
            if !Self::is_valid_package_name(pkg) {
                return Err(format!("Invalid package name: {}", pkg));
            }
        }
        Ok(self.install_command(packages))
    }

    /// Build remove command with validation
    pub fn remove_command_validated(&self, packages: &[&str]) -> std::result::Result<CommandSpec, String> {
        for pkg in packages {
            if !Self::is_valid_package_name(pkg) {
                return Err(format!("Invalid package name: {}", pkg));
            }
        }
        Ok(self.remove_command(packages))
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
    fn test_package_name_sanitization() {
        assert_eq!(AurHelperCommand::sanitize_package_name("firefox"), "firefox");
        assert_eq!(AurHelperCommand::sanitize_package_name("linux-headers"), "linux-headers");
        assert_eq!(AurHelperCommand::sanitize_package_name("python-pytest"), "python-pytest");
        assert_eq!(AurHelperCommand::sanitize_package_name("pkg+name@test"), "pkg+name@test");
        assert_eq!(AurHelperCommand::sanitize_package_name("test; rm -rf /"), "testrm-rf");
    }

    #[test]
    fn test_package_name_validation() {
        assert!(AurHelperCommand::is_valid_package_name("firefox"));
        assert!(AurHelperCommand::is_valid_package_name("linux-headers"));
        assert!(AurHelperCommand::is_valid_package_name("python3"));
        assert!(AurHelperCommand::is_valid_package_name("nodejs-lts-hydrogen"));
        assert!(!AurHelperCommand::is_valid_package_name(""));
        assert!(!AurHelperCommand::is_valid_package_name("test; rm"));
        assert!(!AurHelperCommand::is_valid_package_name("test|grep"));
        assert!(!AurHelperCommand::is_valid_package_name("test\n"));
    }

    #[test]
    fn test_aur_helper_install_validated() {
        let config = AppConfig::default();
        let helper = AurHelperCommand::new(&config);

        let result = helper.install_command_validated(&["firefox", "vlc"]);
        assert!(result.is_ok());

        let result = helper.install_command_validated(&["test; rm -rf /"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid package name"));
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
