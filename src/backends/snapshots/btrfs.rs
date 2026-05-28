//! btrfs snapshot provider implementation.
//!
//! Creates and manages btrfs filesystem snapshots before package
//! operations, with automatic cleanup of old snapshots.

use crate::errors::{AppError, Result};
use crate::traits::{SnapshotInfo, SnapshotProvider};
use crate::utils::validate_path;
use async_trait::async_trait;
use chrono::{DateTime, Local};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

/// BTRFS-based snapshot provider.
///
/// Uses the `btrfs` command-line utility to create and manage subvolume snapshots.
pub struct BtrfsProvider {
    root_path: PathBuf,
    snapshots_dir: PathBuf,
}

impl BtrfsProvider {
    /// Creates a new BtrfsProvider.
    ///
    /// # Arguments
    /// * `root_path` - The path to the root subvolume to snapshot.
    /// * `snapshots_dir` - The directory where snapshots will be stored.
    pub fn new<P: AsRef<Path>>(root_path: P, snapshots_dir: P) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
            snapshots_dir: snapshots_dir.as_ref().to_path_buf(),
        }
    }

    /// Helper to run a command and capture its output for better error reporting.
    async fn run_command(&self, prog: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(prog).args(args).output().await?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let err_msg = if stderr.is_empty() {
                format!("{} exited with status: {}", prog, output.status)
            } else {
                format!("{}: {}", prog, stderr)
            };
            Err(AppError::Backend(err_msg))
        }
    }
}

#[async_trait]
impl SnapshotProvider for BtrfsProvider {
    async fn create(&self, label: &str) -> Result<String> {
        if !validate_path(&self.snapshots_dir) {
            return Err(AppError::Validation(
                "Invalid snapshots directory path".to_string(),
            ));
        }

        let id = format!("metapak-{}-{}", label, Local::now().format("%Y%m%d-%H%M"));
        let dest = self.snapshots_dir.join(&id);

        self.run_command(
            "btrfs",
            &[
                "subvolume",
                "snapshot",
                "-r",
                &self.root_path.to_string_lossy(),
                &dest.to_string_lossy(),
            ],
        )
        .await?;

        Ok(id)
    }

    async fn rollback(&self, id: &str) -> Result<()> {
        let source = self.snapshots_dir.join(id);
        if !validate_path(&source) {
            return Err(AppError::Validation(
                "Invalid snapshot path for rollback".to_string(),
            ));
        }

        self.run_command(
            "btrfs",
            &["subvolume", "set-default", &source.to_string_lossy(), "/"],
        )
        .await
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>> {
        let mut snapshots = Vec::new();

        // Ensure snapshots_dir exists
        if !fs::try_exists(&self.snapshots_dir).await.unwrap_or(false) {
            return Ok(snapshots);
        }

        let mut entries = fs::read_dir(&self.snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name().into_string().map_err(|_| {
                AppError::Backend("Invalid filename in snapshots directory".to_string())
            })?;

            let (is_metapak, is_arch_tui) = (
                file_name.starts_with("metapak-"),
                file_name.starts_with("arch-tui-"),
            );

            if is_metapak || is_arch_tui {
                // Expected Format: metapak-LABEL-YYYYMMDD-HHMM or arch-tui-LABEL-YYYYMMDD-HHMM
                // LABEL can contain dashes, so we split and take the last two parts as timestamp.
                let parts: Vec<&str> = file_name.split('-').collect();
                let label_start = if is_metapak { 1 } else { 2 };
                let min_parts = label_start + 3; // label + date + time

                if parts.len() >= min_parts {
                    let date_idx = parts.len() - 2;
                    let time_idx = parts.len() - 1;

                    let label = parts[label_start..date_idx].join("-");
                    let date_str = parts[date_idx];
                    let time_str = parts[time_idx];
                    let datetime_str = format!("{}-{}", date_str, time_str);

                    // Try to parse the datetime (YYYYMMDD-HHMM)
                    if let Ok(created_at) =
                        DateTime::parse_from_str(&format!("{}+0000", datetime_str), "%Y%m%d-%H%M%z")
                    {
                        snapshots.push(SnapshotInfo {
                            id: file_name,
                            label,
                            created_at: created_at.with_timezone(&Local),
                        });
                    }
                }
            }
        }

        // Sort by creation time descending (newest first)
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    async fn cleanup(&self, keep_count: usize) -> Result<()> {
        let snapshots = self.list().await?;
        if snapshots.len() <= keep_count {
            return Ok(());
        }

        for snapshot in snapshots.iter().skip(keep_count) {
            let path = self.snapshots_dir.join(&snapshot.id);
            self.run_command("btrfs", &["subvolume", "delete", &path.to_string_lossy()])
                .await?;
        }
        Ok(())
    }
}
