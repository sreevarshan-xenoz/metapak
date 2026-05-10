use crate::errors::{AppError, Result};
use crate::traits::{SnapshotInfo, SnapshotProvider};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use tokio::process::Command;

/// Timeshift-based snapshot provider.
///
/// Wraps the `timeshift` command-line utility.
pub struct TimeshiftProvider;

impl TimeshiftProvider {
    pub fn new() -> Self {
        Self
    }

    async fn run_timeshift(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("timeshift").args(args).output().await?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        if output.status.success() {
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(AppError::Backend(format!("Timeshift error: {}", stderr)))
        }
    }
}

#[async_trait]
impl SnapshotProvider for TimeshiftProvider {
    async fn create(&self, label: &str) -> Result<String> {
        let comment = format!("metapak-{}", label);
        let output = self
            .run_timeshift(&["--create", "--comments", &comment, "--tags", "O"])
            .await?;

        // Timeshift output usually contains the created snapshot name/timestamp
        // We'll try to extract it or just return the comment as ID if we can't be sure
        // For Timeshift, IDs are often timestamps like "2026-05-07_12-00-00"
        if let Some(line) = output.lines().find(|l| l.contains("Created snapshot:")) {
            let parts: Vec<&str> = line.split(':').collect();
            if let Some(id) = parts.get(1) {
                return Ok(id.trim().to_string());
            }
        }

        Ok(comment)
    }

    async fn rollback(&self, id: &str) -> Result<()> {
        // Warning: timeshift --restore is usually interactive.
        // We use --script to avoid prompts if supported, or assume --restore with snapshot id works.
        self.run_timeshift(&["--restore", "--snapshot", id, "--script"])
            .await?;
        Ok(())
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>> {
        let output = self.run_timeshift(&["--list"]).await?;
        let mut snapshots = Vec::new();

        // Timeshift --list output parsing:
        // Index  Name                 Tags  Description
        // 0      2026-05-07_12-00-00  O     metapak-operation

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[1].contains('_') {
                let id = parts[1].to_string();
                let label = parts[3..].join(" ");

                // Parse "2026-05-07_12-00-00"
                if let Ok(created_at) =
                    DateTime::parse_from_str(&format!("{}+0000", id), "%Y-%m-%d_%H-%M-%S%z")
                {
                    snapshots.push(SnapshotInfo {
                        id,
                        label,
                        created_at: created_at.with_timezone(&Local),
                    });
                }
            }
        }

        Ok(snapshots)
    }

    async fn cleanup(&self, _keep_count: usize) -> Result<()> {
        // Implementation for cleanup if needed, timeshift has its own retention policy
        // but we could implement manual deletion of oldest metapak/arch-tui snapshots.
        let snapshots = self.list().await?;
        let managed_snapshots: Vec<_> = snapshots
            .iter()
            .filter(|s| s.label.contains("metapak") || s.label.contains("arch-tui"))
            .collect();

        if managed_snapshots.len() > _keep_count {
            for s in managed_snapshots.iter().skip(_keep_count) {
                let _ = self.run_timeshift(&["--delete", "--snapshot", &s.id]).await;
            }
        }

        Ok(())
    }
}
