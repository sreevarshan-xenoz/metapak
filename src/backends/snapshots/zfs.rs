use crate::errors::{AppError, Result};
use crate::traits::{SnapshotInfo, SnapshotProvider};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use std::path::PathBuf;
use tokio::process::Command;

pub struct ZfsProvider {
    pool: String,
    dataset: String,
}

impl ZfsProvider {
    pub fn new(pool: &str, dataset: &str) -> Self {
        Self {
            pool: pool.to_string(),
            dataset: dataset.to_string(),
        }
    }

    async fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("zfs").args(args).output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(AppError::Backend(format!("zfs: {}", stderr)))
        }
    }

    fn full_dataset(&self) -> String {
        format!("{}/{}", self.pool, self.dataset)
    }
}

#[async_trait]
impl SnapshotProvider for ZfsProvider {
    async fn create(&self, label: &str) -> Result<String> {
        let snapshot_name = format!("{}/{}@arch-tui-{}", self.pool, self.dataset, label);
        let id = format!("arch-tui-{}-{}", label, Local::now().format("%Y%m%d-%H%M"));

        self.run_command(&["snapshot", &snapshot_name]).await?;

        Ok(id)
    }

    async fn rollback(&self, id: &str) -> Result<()> {
        let snapshot = format!("{}/{}@{}", self.pool, self.dataset, id);
        self.run_command(&["rollback", "-r", &snapshot]).await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>> {
        let dataset = self.full_dataset();
        let output = self
            .run_command(&["list", "-t", "snapshot", "-r", "-o", "name", "-H", &dataset])
            .await?;

        let mut snapshots = Vec::new();

        for line in output.lines() {
            let name = line.trim();
            if name.contains("@arch-tui-") {
                if let Some(snapshot_part) = name.rsplit('@').next() {
                    let id = snapshot_part.replace(&format!("{}/", self.dataset), "");
                    let label = id
                        .strip_prefix("arch-tui-")
                        .unwrap_or(&id)
                        .split('-')
                        .next()
                        .unwrap_or("unknown")
                        .to_string();

                    snapshots.push(SnapshotInfo {
                        id,
                        label,
                        created_at: Local::now(),
                    });
                }
            }
        }

        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    async fn cleanup(&self, keep_count: usize) -> Result<()> {
        let snapshots = self.list().await?;
        if snapshots.len() <= keep_count {
            return Ok(());
        }

        for snapshot in snapshots.iter().skip(keep_count) {
            let snapshot_name = format!("{}/{}@{}", self.pool, self.dataset, &snapshot.id);
            let _ = self.run_command(&["destroy", &snapshot_name]).await;
        }

        Ok(())
    }
}
