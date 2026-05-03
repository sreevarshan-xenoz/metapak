use crate::errors::{AppError, Result};
use crate::traits::{PackageSimulator, SimulationResult};
use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use tokio::process::Command;

static DOWNLOAD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Total Download Size:\s+([\d.]+)\s+(\w+)").expect("Invalid download regex")
});
static INSTALL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Total Installed Size:\s+([\d.]+)\s+(\w+)").expect("Invalid install regex")
});
static NET_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Net Upgrade Size:\s+([\d.-]+)\s+(\w+)").expect("Invalid net regex")
});
static CONFLICT_PKG_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"::\s+(.+)\s+and\s+(.+)\s+are\s+in\s+conflict").expect("Invalid conflict pkg regex")
});
static UNRESOLVABLE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"error:\s+unresolvable\s+package\s+conflicts").expect("Invalid unresolvable regex")
});
static TRANSACTION_FAILED_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"error: failed to (?:commit|prepare) transaction \((.+)\)")
        .expect("Invalid transaction failed regex")
});

/// Simulation engine for package operations
pub struct SimulationEngine {
    backend: String,
}

impl SimulationEngine {
    /// Create a new simulation engine for the specified backend
    pub fn new(backend: String) -> Self {
        Self { backend }
    }

    /// Parse pacman dry-run output to extract simulation metrics
    pub fn parse_pacman_output(output: &str) -> SimulationResult {
        let mut total_download_bytes = 0;
        let mut disk_change_bytes = 0;
        let mut conflicts = Vec::new();
        let config_changes = Vec::new();

        for line in output.lines() {
            // Check for download size
            if let Some(caps) = DOWNLOAD_RE.captures(line) {
                if let Ok(val) = caps[1].parse::<f64>() {
                    total_download_bytes = convert_to_bytes(val, &caps[2]).max(0) as u64;
                }
            }
            // Check for installed size (fallback for disk change if net_re not found)
            else if let Some(caps) = INSTALL_RE.captures(line) {
                if let Ok(val) = caps[1].parse::<f64>() {
                    let bytes = convert_to_bytes(val, &caps[2]);
                    if disk_change_bytes == 0 {
                        disk_change_bytes = bytes;
                    }
                }
            }
            // Net upgrade size is often more accurate for "change"
            else if let Some(caps) = NET_RE.captures(line) {
                if let Ok(val) = caps[1].parse::<f64>() {
                    disk_change_bytes = convert_to_bytes(val, &caps[2]);
                }
            }

            // Detect conflicts
            if let Some(caps) = TRANSACTION_FAILED_RE.captures(line) {
                conflicts.push(caps[1].trim().to_string());
            } else if line.contains("conflicting files") || line.contains("exists in filesystem") {
                conflicts.push(line.trim().to_string());
            } else if CONFLICT_PKG_RE.is_match(line) || UNRESOLVABLE_RE.is_match(line) {
                conflicts.push(line.trim().to_string());
            }
        }

        SimulationResult {
            total_download_bytes,
            disk_change_bytes,
            conflicts,
            config_changes,
        }
    }
}

/// Helper to convert size with unit to bytes (handles negative values)
fn convert_to_bytes(val: f64, unit: &str) -> i64 {
    let multiplier = match unit.to_uppercase().as_str() {
        "KIB" | "KB" => 1024.0,
        "MIB" | "MB" => 1024.0 * 1024.0,
        "GIB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "TIB" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (val * multiplier) as i64
}

#[async_trait]
impl PackageSimulator for SimulationEngine {
    async fn simulate_install(&self, packages: &[&str]) -> Result<SimulationResult> {
        match self.backend.as_str() {
            "pacman" => {
                // -Sw: download only, shows summary block without installing
                // Using LC_ALL=C to ensure predictable output for parsing
                let output = Command::new("pacman")
                    .env("LC_ALL", "C")
                    .args(["-Sw", "--noconfirm"])
                    .args(packages)
                    .output()
                    .await?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                let result = Self::parse_pacman_output(&combined);

                if !output.status.success() && result.conflicts.is_empty() {
                    return Err(AppError::Pacman(stderr.trim().to_string()));
                }

                Ok(result)
            }
            "apt" => {
                // Stub for apt simulation
                let output = Command::new("apt-get")
                    .env("LC_ALL", "C")
                    .args(["install", "-s"])
                    .args(packages)
                    .output()
                    .await?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(AppError::Command(format!("apt-get failed: {}", stderr.trim())));
                }

                // For now return empty result for apt
                Ok(SimulationResult {
                    total_download_bytes: 0,
                    disk_change_bytes: 0,
                    conflicts: Vec::new(),
                    config_changes: Vec::new(),
                })
            }
            _ => {
                // Default fallback for unimplemented backends
                Ok(SimulationResult {
                    total_download_bytes: 0,
                    disk_change_bytes: 0,
                    conflicts: Vec::new(),
                    config_changes: Vec::new(),
                })
            }
        }
    }

    async fn simulate_upgrade(&self) -> Result<SimulationResult> {
        match self.backend.as_str() {
            "pacman" => {
                // -Syuw: simulate full system upgrade (download only)
                let output = Command::new("pacman")
                    .env("LC_ALL", "C")
                    .args(["-Syuw", "--noconfirm"])
                    .output()
                    .await?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                let result = Self::parse_pacman_output(&combined);

                if !output.status.success() && result.conflicts.is_empty() {
                    return Err(AppError::Pacman(stderr.trim().to_string()));
                }

                Ok(result)
            }
            _ => {
                Ok(SimulationResult {
                    total_download_bytes: 0,
                    disk_change_bytes: 0,
                    conflicts: Vec::new(),
                    config_changes: Vec::new(),
                })
            }
        }
    }
}
