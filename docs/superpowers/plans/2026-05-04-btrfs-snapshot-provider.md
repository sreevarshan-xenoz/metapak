# BTRFS Snapshot Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a BTRFS-based snapshot provider for Arch-TUI to enable system-level snapshots before package operations.

**Architecture:** Implement the `SnapshotProvider` trait defined in `src/traits.rs` using the `btrfs` CLI tool. Snapshots will be created as read-only subvolumes in a dedicated directory.

**Tech Stack:** Rust, `tokio` (for async process execution and FS operations), `chrono` (for timestamps), `async-trait`.

---

### Task 1: Add Backend Error to AppError

**Files:**
- Modify: `src/errors.rs`

- [ ] **Step 1: Add Backend variant to AppError enum**

```rust
// In src/errors.rs, add to AppError enum:
    #[error("Backend error: {0}")]
    Backend(String),
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add src/errors.rs
git commit -m "feat: add Backend error variant to AppError"
```

### Task 2: Create Snapshots Module

**Files:**
- Create: `src/backends/snapshots/mod.rs`
- Modify: `src/backends/mod.rs`

- [ ] **Step 1: Create mod.rs in backends/snapshots**

```rust
pub mod btrfs;
```

- [ ] **Step 2: Export snapshots module in backends/mod.rs**

```rust
// In src/backends/mod.rs
pub mod snapshots;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`

- [ ] **Step 4: Commit**

```bash
git add src/backends/snapshots/mod.rs src/backends/mod.rs
git commit -m "feat: create snapshots module structure"
```

### Task 3: Implement BtrfsProvider

**Files:**
- Create: `src/backends/snapshots/btrfs.rs`

- [ ] **Step 1: Implement BtrfsProvider with create and rollback**

```rust
use crate::errors::{Result, AppError};
use crate::traits::{SnapshotProvider, SnapshotInfo};
use async_trait::async_trait;
use chrono::{Local, DateTime};
use std::path::Path;
use tokio::process::Command;
use tokio::fs;

pub struct BtrfsProvider {
    root_path: String,
    snapshots_dir: String,
}

impl BtrfsProvider {
    pub fn new(root_path: String, snapshots_dir: String) -> Self {
        Self { root_path, snapshots_dir }
    }
}

#[async_trait]
impl SnapshotProvider for BtrfsProvider {
    async fn create(&self, label: &str) -> Result<String> {
        let id = format!("arch-tui-{}-{}", label, Local::now().format("%Y%m%d-%H%M"));
        let dest = format!("{}/{}", self.snapshots_dir, id);
        
        let status = Command::new("btrfs")
            .args(["subvolume", "snapshot", "-r", &self.root_path, &dest])
            .status()
            .await?;
        
        if status.success() {
            Ok(id)
        } else {
            Err(AppError::Backend(format!("Failed to create BTRFS snapshot: {}", status)))
        }
    }

    async fn rollback(&self, id: &str) -> Result<()> {
        let source = format!("{}/{}", self.snapshots_dir, id);
        let status = Command::new("btrfs")
            .args(["subvolume", "set-default", &source, "/"])
            .status()
            .await?;
        
        if status.success() {
            Ok(())
        } else {
            Err(AppError::Backend(format!("Failed to rollback BTRFS snapshot: {}", status)))
        }
    }

    async fn list(&self) -> Result<Vec<SnapshotInfo>> {
        let mut snapshots = Vec::new();
        let mut entries = fs::read_dir(&self.snapshots_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name().into_string().map_err(|_| {
                AppError::Backend("Invalid filename in snapshots directory".to_string())
            })?;

            if file_name.starts_with("arch-tui-") {
                // Parse label and timestamp: arch-tui-LABEL-YYYYMMDD-HHMM
                let parts: Vec<&str> = file_name.split('-').collect();
                if parts.len() >= 4 {
                    let label = parts[2].to_string();
                    let date_str = parts[3];
                    let time_str = parts[4];
                    let datetime_str = format!("{}-{}", date_str, time_str);
                    
                    if let Ok(created_at) = DateTime::parse_from_str(&format!("{}+0000", datetime_str), "%Y%m%d-%H%M%z") {
                        snapshots.push(SnapshotInfo {
                            id: file_name,
                            label,
                            created_at: created_at.with_timezone(&Local),
                        });
                    }
                }
            }
        }
        
        // Sort by creation time descending
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    async fn cleanup(&self, keep_count: usize) -> Result<()> {
        let snapshots = self.list().await?;
        if snapshots.len() <= keep_count {
            return Ok(());
        }

        for snapshot in snapshots.iter().skip(keep_count) {
            let path = format!("{}/{}", self.snapshots_dir, snapshot.id);
            let status = Command::new("btrfs")
                .args(["subvolume", "delete", &path])
                .status()
                .await?;
            
            if !status.success() {
                return Err(AppError::Backend(format!("Failed to delete BTRFS snapshot: {}", status)));
            }
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`

- [ ] **Step 3: Commit**

```bash
git add src/backends/snapshots/btrfs.rs
git commit -m "feat: implement BtrfsProvider"
```

### Task 4: Final Verification

**Files:**
- N/A

- [ ] **Step 1: Run full check**

Run: `cargo check && cargo test --lib`

- [ ] **Step 2: Commit any fixes if needed**
