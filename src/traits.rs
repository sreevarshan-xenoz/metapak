//! Async traits for package management operations
//!
//! This module defines the core interfaces for package providers,
//! allowing for pluggable implementations and easier testing.

use crate::errors::Result;
use crate::models::{Package, OutdatedPackage};
use async_trait::async_trait;

/// Trait for package search providers
#[async_trait]
pub trait PackageProvider: Send + Sync {
    /// Search for packages matching the query
    async fn search(&self, query: &str) -> Result<Vec<Package>>;

    /// Check if a specific package is installed
    async fn is_installed(&self, pkg_name: &str) -> bool;
}

/// Trait for system update operations
#[async_trait]
pub trait UpdateProvider: Send + Sync {
    /// Check for available updates
    async fn check_updates(&self) -> Result<usize>;

    /// Get detailed list of outdated packages
    async fn get_outdated_packages(&self) -> Result<Vec<OutdatedPackage>>;
}
