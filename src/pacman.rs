use std::process::Command;
use crate::models::{Package, PackageSource};
use anyhow::Result;

pub fn search(query: &str) -> Result<Vec<Package>> {
    let output = Command::new("pacman")
        .arg("-Ss")
        .arg(query)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
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
    })
}

pub fn is_installed(pkg_name: &str) -> bool {
    Command::new("pacman")
        .arg("-Qi")
        .arg(pkg_name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_updates() -> Result<usize> {
    // Try checkupdates first (from pacman-contrib)
    if let Ok(output) = Command::new("checkupdates").output() {
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout)?;
            return Ok(stdout.lines().count());
        }
    }
    
    // Fallback to pacman -Qu (checks against local DB, which might be stale but better than nothing if checkupdates missing)
    let output = Command::new("pacman")
        .arg("-Qu")
        .output()?;
        
    if output.status.success() {
         let stdout = String::from_utf8(output.stdout)?;
         return Ok(stdout.lines().count());
    }
    
    // If it fails (e.g. no updates or error), return 0
    Ok(0)
}
