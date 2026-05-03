use crate::errors::Result;
use std::time::{Duration, Instant};
use futures::future::join_all;
use std::path::Path;
use tokio::process::Command;

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

        let mut tasks = Vec::new();
        for url in mirrors {
            let client = client.clone();
            let url = url.clone();
            tasks.push(tokio::spawn(async move {
                let start = Instant::now();
                let result = client.head(&url).send().await;
                let latency = start.elapsed();
                
                MirrorHealth {
                    url,
                    latency,
                    reachable: result.is_ok(),
                }
            }));
        }

        let results = join_all(tasks).await;
        let mut mirror_healths = Vec::new();
        for res in results {
            if let Ok(health) = res {
                mirror_healths.push(health);
            }
        }

        Ok(mirror_healths)
    }

    pub async fn check_gpg_keys(&self) -> Result<bool> {
        let output = Command::new("pacman-key")
            .arg("--list-keys")
            .output()
            .await
            .map_err(|e| crate::errors::AppError::Command(format!("Failed to run pacman-key: {}", e)))?;

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
