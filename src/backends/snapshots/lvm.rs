use crate::errors::{AppError, Result};
use crate::traits::{SnapshotInfo, SnapshotProvider};
use async_trait::async_trait;
use chrono::{DateTime, Local};
use std::path::PathBuf;
use tokio::process::Command;

pub struct LvmProvider {
    volume_group: String,
    logical_volume: String,
    snapshot_prefix: String,
}

impl LvmProvider {
    pub fn new(volume_group: &str, logical_volume: &str) -> Self {
        Self {
            volume_group: volume_group.to_string(),
            logical_volume: logical_volume.to_string(),
            snapshot_prefix: "metapak-snap".to_string(),
        }
    }

    async fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = Command::new("lvcreate").args(args).output().await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(AppError::Backend(format!("lvm: {}", stderr)))
        }
    }

    fn snapshot_name(&self, label: &str) -> String {
        format!(
            "{}-{}-{}",
            self.snapshot_prefix,
            label,
            Local::now().format("%Y%m%d-%H%M")
        )
    }
}

#[async_trait]
impl SnapshotProvider for LvmProvider {
    async fn create(&self, label: &str) -> Result<String> {
        let snap_name = self.snapshot_name(label);
        let full_lv = format!("{}/{}", self.volume_group, self.logical_volume);
        let _full_snap = format!("{}/{}", self.volume_group, snap_name);

        self.run_command(&["-s", &full_lv, "-n", &snap_name, "-L", "1G"])
            .await?;

        Ok(snap_name)
    }

    async fn rollback(&self, id: &str) -> Result<()> {
        let full_snap = format!("{}/{}", self.volume_group, id);
        let _full_lv = format!("{}/{}", self.volume_group, self.logical_volume);

        Command::new("lvconvert")
            .args(["--merge", &full_snap])
            .output()
            .await?;

        Ok(())
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>> {
        let output = Command::new("lvs")
            .args(["--noheading", "-o", "lv_name,lv_time", "--separator", ","])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut snapshots = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            let prefix = if line.contains("metapak-snap") {
                Some("metapak-snap")
            } else if line.contains("arch-tui-snap") {
                Some("arch-tui-snap")
            } else {
                None
            };

            if let Some(p) = prefix {
                let parts: Vec<&str> = line.split(',').collect();
                if !parts.is_empty() {
                    let id = parts[0].trim().to_string();
                    let id_parts: Vec<&str> = id.split('-').collect();
                    
                    // prefix is metapak-snap (2 parts) or arch-tui-snap (3 parts)
                    let label_start = if p == "metapak-snap" { 2 } else { 3 };
                    let min_parts = label_start + 2; // label + timestamp

                    let label = if id_parts.len() >= min_parts {
                        let date_idx = id_parts.len() - 2;
                        id_parts[label_start..date_idx].join("-")
                    } else {
                        id.strip_prefix(&format!("{}-", p)).unwrap_or(&id).to_string()
                    };

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
            let full_snap = format!("{}/{}", self.volume_group, &snapshot.id);
            let _ = Command::new("lvremove")
                .args(["-f", &full_snap])
                .output()
                .await;
        }

        Ok(())
    }
}
