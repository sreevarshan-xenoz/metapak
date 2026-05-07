use crate::errors::Result;
use futures::StreamExt;
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tracing;

#[derive(Debug, Clone)]
pub struct MirrorHealth {
    pub url: String,
    pub latency: Duration,
    pub reachable: bool,
}

#[derive(Debug, Clone)]
pub struct DiskHealth {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f64,
}

pub struct HealthWatchdog {
    ping_timeout: Duration,
}

impl HealthWatchdog {
    pub fn new(ping_timeout: Duration) -> Self {
        Self { ping_timeout }
    }

    pub async fn check_mirrors(&self, mirrors: &[String]) -> Result<Vec<MirrorHealth>> {
        let client = reqwest::Client::builder()
            .timeout(self.ping_timeout)
            .build()?;

        let stream = futures::stream::iter(mirrors.iter().cloned())
            .map(|url| {
                let client = client.clone();
                async move {
                    let start = Instant::now();
                    let result = client.head(&url).send().await;
                    let latency = start.elapsed();

                    match result {
                        Ok(resp) => {
                            let is_success = resp.status().is_success();
                            if !is_success {
                                tracing::warn!(url = %url, status = %resp.status(), "Mirror returned non-success status");
                            }
                            MirrorHealth {
                                url,
                                latency,
                                reachable: is_success,
                            }
                        }
                        Err(e) => {
                            tracing::error!(url = %url, error = %e, "Mirror check failed");
                            MirrorHealth {
                                url,
                                latency,
                                reachable: false,
                            }
                        }
                    }
                }
            })
            .buffer_unordered(5);

        let results = stream.collect::<Vec<_>>().await;
        Ok(results)
    }

    pub async fn check_gpg_keys(&self) -> Result<bool> {
        let output = Command::new("pacman-key")
            .arg("--list-keys")
            .output()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to execute pacman-key");
                crate::errors::AppError::Command(format!("Failed to run pacman-key: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::error!(status = ?output.status, stderr = %stderr, "pacman-key command failed");
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(!stdout.contains("[expired]"))
    }

    pub async fn check_db_lock(&self) -> Result<bool> {
        Ok(Path::new("/var/lib/pacman/db.lck").exists())
    }

    pub async fn check_disk_space(&self) -> Result<Vec<DiskHealth>> {
        let output = Command::new("df")
            .args(["-B1", "-T", "/", "/boot", "/home"])
            .output()
            .await
            .map_err(|e| crate::errors::AppError::Command(format!("Failed to run df: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut disks = Vec::new();

        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                let fs_type = parts[1];
                if fs_type == "tmpfs" || fs_type == "devtmpfs" || fs_type == "squashfs" {
                    continue;
                }

                let mount = parts[6].to_string();
                if let (Ok(total), Ok(used), Ok(available)) = (
                    parts[2].parse::<u64>(),
                    parts[3].parse::<u64>(),
                    parts[4].parse::<u64>(),
                ) {
                    let usage_percent = if total > 0 {
                        (used as f64 / total as f64) * 100.0
                    } else {
                        0.0
                    };

                    disks.push(DiskHealth {
                        mount_point: mount,
                        total_bytes: total,
                        used_bytes: used,
                        available_bytes: available,
                        usage_percent,
                    });
                }
            }
        }

        Ok(disks)
    }

    pub async fn check_pacman_status(&self) -> Result<PacmanStatus> {
        let db_lock = self.check_db_lock().await?;
        let gpg_ok = self.check_gpg_keys().await?;

        Ok(PacmanStatus {
            db_locked: db_lock,
            gpg_valid: gpg_ok,
            last_update: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PacmanStatus {
    pub db_locked: bool,
    pub gpg_valid: bool,
    pub last_update: Option<chrono::DateTime<chrono::Local>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watchdog_new() {
        let watchdog = HealthWatchdog::new(Duration::from_secs(5));
        assert_eq!(watchdog.ping_timeout, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_check_db_lock() {
        let watchdog = HealthWatchdog::new(Duration::from_secs(1));
        // We can't easily test existence of /var/lib/pacman/db.lck in generic environment,
        // but we can verify the method returns a bool.
        let _ = watchdog.check_db_lock().await;
    }
}
