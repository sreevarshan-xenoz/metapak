//! Async traits for package management operations
//!
//! This module defines the core interfaces for package providers,
//! allowing for pluggable implementations and easier testing.

use crate::errors::Result;
use crate::models::Package;
use async_trait::async_trait;

/// Trait for package search providers
#[async_trait]
pub trait PackageProvider: Send + Sync {
    /// Search for packages matching the query
    #[must_use = "this async method should be .await'd"]
    async fn search(&self, query: &str) -> Result<Vec<Package>>;

    /// Check if a specific package is installed
    #[must_use = "this async method should be .await'd"]
    async fn is_installed(&self, pkg_name: &str) -> bool;
}

/// Trait for system update operations
#[async_trait]
pub trait UpdateProvider: Send + Sync {
    /// Check for available updates
    #[must_use = "this async method should be .await'd"]
    async fn check_updates(&self) -> Result<usize>;

    /// Get a detailed list of outdated packages
    #[must_use = "this async method should be .await'd"]
    #[allow(dead_code)]
    async fn get_outdated_packages(&self) -> Result<Vec<crate::models::OutdatedPackage>>;
}

/// Trait for filesystem snapshots
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    /// Create a new snapshot with a label
    async fn create(&self, label: &str) -> Result<String>;

    /// Rollback to a specific snapshot ID
    async fn rollback(&self, id: &str) -> Result<()>;

    /// List available snapshots
    async fn list(&self) -> Result<Vec<SnapshotInfo>>;

    /// Cleanup old snapshots, keeping the specified number of most recent ones
    async fn cleanup(&self, keep_count: usize) -> Result<()>;
}

/// Information about a filesystem snapshot
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    /// Unique identifier for the snapshot
    pub id: String,
    /// Human-readable label
    pub label: String,
    /// When the snapshot was created
    pub created_at: chrono::DateTime<chrono::Local>,
}

/// Interface for package operation simulation (dry-run)
#[async_trait]
pub trait PackageSimulator: Send + Sync {
    /// Simulate installing a set of packages
    async fn simulate_install(&self, packages: &[&str]) -> Result<SimulationResult>;
}

/// Result of a package operation simulation
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Total download size in bytes
    pub total_download_bytes: u64,
    /// Projected change in disk usage (positive for growth, negative for shrinkage)
    pub disk_change_bytes: i64,
    /// List of identified package conflicts
    pub conflicts: Vec<String>,
    /// List of potential configuration file changes
    pub config_changes: Vec<String>,
}
