use crate::errors::Result;
use crate::traits::{PackageSimulator, SimulationResult};
use async_trait::async_trait;
use regex::Regex;
use tokio::process::Command;

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
        let mut disk_change_bytes = 0;
        let mut conflicts = Vec::new();
        let mut config_changes = Vec::new();

        // Regex for sizes
        let install_re = Regex::new(r"Total Installed Size:\s+([\d.]+)\s+(\w+)").unwrap();
        let net_re = Regex::new(r"Net Upgrade Size:\s+([\d.-]+)\s+(\w+)").unwrap();

        for line in output.lines() {
            // Check for installed size
            if let Some(caps) = install_re.captures(line) {
                if let Ok(val) = caps[1].parse::<f64>() {
                    disk_change_bytes = convert_to_bytes(val, &caps[2]) as i64;
                }
            }
            // Net upgrade size is often more accurate for "change"
            else if let Some(caps) = net_re.captures(line) {
                if let Ok(val) = caps[1].parse::<f64>() {
                    disk_change_bytes = convert_to_bytes(val, &caps[2]) as i64;
                }
            }

            // Detect conflicts
            if line.contains("error: failed to commit transaction") {
                if let Some(reason) = line.split('(').nth(1) {
                    conflicts.push(reason.trim_matches(')').to_string());
                }
            } else if line.contains("conflicting files") || line.contains("exists in filesystem") {
                conflicts.push(line.trim().to_string());
            }

            // Detect potential config changes (pacnew)
            if line.contains(".pacnew") {
                config_changes.push(line.trim().to_string());
            }
        }

        SimulationResult {
            disk_change_bytes,
            conflicts,
            config_changes,
        }
    }
}

/// Helper to convert size with unit to bytes
fn convert_to_bytes(val: f64, unit: &str) -> u64 {
    let multiplier = match unit.to_uppercase().as_str() {
        "KIB" | "KB" => 1024.0,
        "MIB" | "MB" => 1024.0 * 1024.0,
        "GIB" | "GB" => 1024.0 * 1024.0 * 1024.0,
        "TIB" | "TB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (val * multiplier) as u64
}

#[async_trait]
impl PackageSimulator for SimulationEngine {
    async fn simulate_install(&self, packages: &[&str]) -> Result<SimulationResult> {
        match self.backend.as_str() {
            "pacman" => {
                // -Sp: print targets and sizes without installing
                let output = Command::new("pacman")
                    .args(["-Sp", "--noconfirm"])
                    .args(packages)
                    .output()
                    .await?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                Ok(Self::parse_pacman_output(&combined))
            }
            "apt" => {
                // Stub for apt simulation
                let _output = Command::new("apt-get")
                    .args(["install", "-s"])
                    .args(packages)
                    .output()
                    .await?;
                
                // For now return empty result for apt
                Ok(SimulationResult {
                    disk_change_bytes: 0,
                    conflicts: Vec::new(),
                    config_changes: Vec::new(),
                })
            }
            _ => {
                // Default fallback for unimplemented backends
                Ok(SimulationResult {
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
                // -Syu -p: simulate full system upgrade
                let output = Command::new("pacman")
                    .args(["-Syu", "-p", "--noconfirm"])
                    .output()
                    .await?;

                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                Ok(Self::parse_pacman_output(&combined))
            }
            _ => {
                Ok(SimulationResult {
                    disk_change_bytes: 0,
                    conflicts: Vec::new(),
                    config_changes: Vec::new(),
                })
            }
        }
    }
}
