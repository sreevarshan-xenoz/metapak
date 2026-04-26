//! Export and import functionality for package lists.
//!
//! This module provides functions to export installed packages to a file
//! and import package lists for batch operations.

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub fn export_installed(packages: &[crate::models::Package], path: &Path) -> std::io::Result<()> {
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

fn chrono_lite() -> String {
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
