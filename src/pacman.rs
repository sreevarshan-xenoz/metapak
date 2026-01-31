//! Pacman package manager integration
//!
//! This module provides functions for interacting with the pacman package manager,
//! including searching for packages, checking if packages are installed,
//! and checking for available updates.

use std::process::Command;
use crate::models::{Package, PackageSource};
use crate::errors::{Result, AppError};

/// Searches for packages in the pacman repositories
///
/// # Arguments
/// * `query` - The search term to look for in package names and descriptions
///
/// # Returns
/// A vector of packages matching the search query
///
/// # Errors
/// Returns an error if the pacman command fails or produces invalid output
pub fn search(query: &str) -> Result<Vec<Package>> {
    let output = Command::new("pacman")
        .arg("-Ss")
        .arg(query)
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
}

/// Parses a pacman package entry from command output
///
/// # Arguments
/// * `header` - The header line containing package info (e.g., "core/linux 6.6.1-arch1 [installed]")
/// * `desc` - The description line
///
/// # Returns
/// A Package struct if parsing is successful, None otherwise
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

/// Checks if a package is installed on the system
///
/// # Arguments
/// * `pkg_name` - The name of the package to check
///
/// # Returns
/// True if the package is installed, false otherwise
pub fn is_installed(pkg_name: &str) -> bool {
    Command::new("pacman")
        .arg("-Qi")
        .arg(pkg_name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Checks for available package updates
///
/// This function attempts to use `checkupdates` from pacman-contrib first,
/// and falls back to `pacman -Qu` if that's not available.
///
/// # Returns
/// The number of available updates
///
/// # Errors
/// Returns an error if both checkupdate methods fail
pub fn check_updates() -> Result<usize> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pacman_entry_installed() {
        let header = "core/linux 6.6.1-arch1 (base) [installed]";
        let desc = "The Linux kernel and modules";

        let pkg = parse_pacman_entry(header, desc);
        assert!(pkg.is_some());

        let pkg = pkg.unwrap();
        assert_eq!(pkg.name, "linux");
        assert_eq!(pkg.version, "6.6.1-arch1");
        assert_eq!(pkg.description, "The Linux kernel and modules");
        assert_eq!(pkg.source, PackageSource::Pacman);
        assert!(pkg.is_installed);
    }

    #[test]
    fn test_parse_pacman_entry_not_installed() {
        let header = "community/firefox 120.0-1";
        let desc = "Standalone web browser from mozilla.org";

        let pkg = parse_pacman_entry(header, desc);
        assert!(pkg.is_some());

        let pkg = pkg.unwrap();
        assert_eq!(pkg.name, "firefox");
        assert_eq!(pkg.version, "120.0-1");
        assert_eq!(pkg.description, "Standalone web browser from mozilla.org");
        assert_eq!(pkg.source, PackageSource::Pacman);
        assert!(!pkg.is_installed);
    }

    #[test]
    fn test_parse_pacman_entry_invalid_format() {
        let header = "just_one_part";  // Only one part, less than required 2
        let desc = "Some description";

        let pkg = parse_pacman_entry(header, desc);
        assert!(pkg.is_none());
    }
}
