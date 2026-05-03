//! Export and import functionality for package lists.
//!
//! This module provides functions to export installed packages to a file
//! and import package lists for batch operations.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn export_installed(packages: &[crate::models::Package], path: &Path) -> std::io::Result<()> {
    // Validate path for security
    if !crate::utils::validate_path(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path: potential path traversal detected"
        ));
    }

    let mut file = File::create(path)?;

    writeln!(file, "# Arch TUI - Exported Package List")?;
    writeln!(file, "# Generated on: {}", chrono_lite())?;
    writeln!(file, "#")?;

    for pkg in packages.iter().filter(|p| p.is_installed) {
        writeln!(file, "{}", pkg.name)?;
    }

    Ok(())
}

pub fn export_all(packages: &[crate::models::Package], path: &Path) -> std::io::Result<()> {
    // Validate path for security
    if !crate::utils::validate_path(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path: potential path traversal detected"
        ));
    }

    let mut file = File::create(path)?;

    writeln!(file, "# Arch TUI - Exported Package List")?;
    writeln!(file, "# Generated on: {}", chrono_lite())?;
    writeln!(file, "#")?;
    writeln!(file, "# Format: package_name [installed]")?;
    writeln!(file, "#")?;

    for pkg in packages {
        let status = if pkg.is_installed { "installed" } else { "" };
        if status.is_empty() {
            writeln!(file, "{}", pkg.name)?;
        } else {
            writeln!(file, "{} [{}]", pkg.name, status)?;
        }
    }

    Ok(())
}

pub fn import_list(path: &Path) -> std::io::Result<Vec<String>> {
    // Validate path for security
    if !crate::utils::validate_path(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path: potential path traversal detected"
        ));
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut packages = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Remove [installed] suffix if present
        let name = trimmed.split_whitespace().next().unwrap_or(trimmed);
        packages.push(name.to_string());
    }

    Ok(packages)
}

pub fn chrono_lite() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple timestamp without full chrono dependency
    let days = secs / 86400;
    let years = days / 365 + 1970;
    let remaining_days = days % 365;
    let months = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;

    format!(
        "{}-{:02}-{:02} {:02}:{:02}",
        years, months, day, hours, minutes
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_export_installed() {
        let mut pkg = crate::models::Package::new("test-pkg", "1.0");
        pkg.source = crate::models::PackageSource::Pacman;
        pkg.is_installed = true;

        let path = "/tmp/arch_tui_test_export.txt";
        export_installed(&[pkg], std::path::Path::new(path)).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("test-pkg"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_import_list() {
        let path = "/tmp/arch_tui_test_import.txt";
        let mut file = fs::File::create(path).unwrap();
        use std::io::Write;
        writeln!(file, "# Comment").unwrap();
        writeln!(file, "pkg1").unwrap();
        writeln!(file, "pkg2 [installed]").unwrap();
        writeln!(file, "").unwrap();
        drop(file);

        let packages = import_list(std::path::Path::new(path)).unwrap();
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0], "pkg1");
        assert_eq!(packages[1], "pkg2");
        let _ = fs::remove_file(path);
    }
}

use std::process::Command;

/// Export all explicitly installed packages for system backup
pub fn export_system_backup(path: &Path) -> std::io::Result<()> {
    if !crate::utils::validate_path(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid path: potential path traversal detected"
        ));
    }

    let output = Command::new("pacman")
        .args(["-Qet", "--color", "never"])
        .output()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;

    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get package list"
        ));
    }

    let packages = String::from_utf8_lossy(&output.stdout);

    let mut file = File::create(path)?;

    writeln!(file, "# Arch Linux System Backup")?;
    writeln!(file, "# Generated on: {}", chrono_lite())?;
    writeln!(file, "# This file contains all explicitly installed packages")?;
    writeln!(file, "# To restore: pacman -S --needed < package_list.txt")?;
    writeln!(file, "#")?;
    writeln!(file)?;

    for line in packages.lines() {
        let name = line.split_whitespace().next().unwrap_or("");
        if !name.is_empty() {
            writeln!(file, "{}", name)?;
        }
    }

    Ok(())
}

/// Get backup file info
pub fn get_backup_info(path: &Path) -> std::io::Result<(String, usize)> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    let mut date = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("# Generated on:") {
            date = line.replace("# Generated on:", "").trim().to_string();
        } else if !line.starts_with('#') && !line.trim().is_empty() {
            count += 1;
        }
    }

    Ok((date, count))
}
