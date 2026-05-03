use crate::errors::Result;
use std::time::{Duration, Instant};
use futures::StreamExt;
use std::path::Path;
use tokio::process::Command;
use tracing;

#[derive(Debug, Clone)]
pub struct MirrorHealth {
    pub url: String,
    pub latency: Duration,
    pub reachable: bool,
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
