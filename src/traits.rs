//! Async traits for package management operations
//!
//! This module defines the core interfaces for package providers,
//! allowing for pluggable implementations and easier testing.

use async_trait::async_trait;
use crate::models::{Package, PackageSource};
use crate::errors::Result;

/// Trait for package search providers
#[async_trait]
pub trait PackageProvider: Send + Sync {
    /// Search for packages matching the query
    async fn search(&self, query: &str) -> Result<Vec<Package>>;
    
    /// Check if a specific package is installed
    async fn is_installed(&self, pkg_name: &str) -> bool;
    
    /// Get the package source type
    fn source(&self) -> PackageSource;
    
    /// Get provider name
    fn name(&self) -> &'static str;
}

/// Trait for system update operations
#[async_trait]
pub trait UpdateProvider: Send + Sync {
    /// Check for available updates
    async fn check_updates(&self) -> Result<usize>;
    
    /// Perform system update
    async fn update_system(&self) -> Result<()>;
}

/// Trait for package installation/removal
#[async_trait]
pub trait PackageInstaller: Send + Sync {
    /// Install packages
    async fn install(&self, packages: &[&str]) -> Result<()>;
    
    /// Remove packages
    async fn remove(&self, packages: &[&str]) -> Result<()>;
    
    /// Get the command that would be executed (for display/logging)
    fn get_install_command(&self, packages: &[&str]) -> String;
    fn get_remove_command(&self, packages: &[&str]) -> String;
}
