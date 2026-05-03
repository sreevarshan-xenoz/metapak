# Arch-TUI Robustness & Safety Suite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a comprehensive safety layer for Arch-TUI including filesystem snapshots, predictive dry-runs, and real-time health monitoring to ensure "never-break" system updates.

**Architecture:** A centralized `TransactionManager` middleware orchestrates a 5-step pipeline: Health Check -> Simulation -> Atomic Snapshot -> Execution -> Failure/Success Handling. It uses pluggable `SnapshotProvider` traits.

**Tech Stack:** Rust, Ratatui, BTRFS/ZFS/LVM CLI tools, reqwest (for health pings).

---

## File Structure

- `src/traits.rs`: Add `SnapshotProvider` and `PackageSimulator` traits.
- `src/backends/snapshots/mod.rs`: Module declaration for snapshot providers.
- `src/backends/snapshots/btrfs.rs`: BTRFS native implementation.
- `src/backends/snapshots/timeshift.rs`: Timeshift CLI wrapper.
- `src/simulation.rs`: `SimulationEngine` for dry-run parsing and disk projection.
- `src/watchdog.rs`: `HealthWatchdog` for mirror latency and GPG checks.
- `src/transaction_manager.rs`: The `TransactionManager` orchestrator.
- `src/app.rs`: State management for safety flags and transaction status.
- `src/ui.rs`: Rendering for Simulation Modal and Rollback Dialog.

---

### Task 1: Define Core Traits

**Files:**
- Modify: `src/traits.rs`

- [ ] **Step 1: Add SnapshotProvider trait**

```rust
#[async_trait]
pub trait SnapshotProvider: Send + Sync {
    async fn create(&self, label: &str) -> crate::errors::Result<String>;
    async fn rollback(&self, id: &str) -> crate::errors::Result<()>;
    async fn list(&self) -> crate::errors::Result<Vec<SnapshotInfo>>;
    async fn cleanup(&self, keep_count: usize) -> crate::errors::Result<()>;
}

pub struct SnapshotInfo {
    pub id: String,
    pub label: String,
    pub created_at: chrono::DateTime<chrono::Local>,
}
```

- [ ] **Step 2: Add PackageSimulator trait**

```rust
#[async_trait]
pub trait PackageSimulator: Send + Sync {
    async fn simulate_install(&self, packages: &[&str]) -> crate::errors::Result<SimulationResult>;
    async fn simulate_upgrade(&self) -> crate::errors::Result<SimulationResult>;
}

pub struct SimulationResult {
    pub disk_change_bytes: i64,
    pub conflicts: Vec<String>,
    pub config_changes: Vec<String>,
}
```

- [ ] **Step 3: Commit**

```bash
git add src/traits.rs
git commit -m "feat: define SnapshotProvider and PackageSimulator traits"
```

---

### Task 2: Implement BTRFS Snapshot Provider

**Files:**
- Create: `src/backends/snapshots/mod.rs`
- Create: `src/backends/snapshots/btrfs.rs`

- [ ] **Step 1: Implement BtrfsProvider**

```rust
pub struct BtrfsProvider {
    root_path: String,
    snapshots_dir: String,
}

#[async_trait]
impl SnapshotProvider for BtrfsProvider {
    async fn create(&self, label: &str) -> Result<String> {
        let id = format!("arch-tui-{}-{}", label, Local::now().format("%Y%m%d-%H%M"));
        let dest = format!("{}/{}", self.snapshots_dir, id);
        Command::new("btrfs")
            .args(["subvolume", "snapshot", "-r", &self.root_path, &dest])
            .status()
            .await?;
        Ok(id)
    }
    // ... implement rollback and list using btrfs subvolume set-default
}
```

- [ ] **Step 2: Create unit test for BtrfsProvider (Mocked CLI)**

- [ ] **Step 3: Commit**

```bash
git add src/backends/snapshots/
git commit -m "feat: implement BTRFS snapshot provider"
```

---

### Task 3: Implement Simulation Engine

**Files:**
- Create: `src/simulation.rs`

- [ ] **Step 1: Implement SimulationEngine**
- [ ] **Step 2: Add parser for Pacman dry-run output (-Sp)**

```rust
pub fn parse_pacman_simulation(output: &str) -> SimulationResult {
    // Regex for: "Total Download Size:   123.45 MiB"
    // Regex for: "Total Installed Size:  567.89 MiB"
    // Regex for: "error: failed to commit transaction (conflicting files)"
}
```

- [ ] **Step 3: Add parser for Apt dry-run output**

- [ ] **Step 4: Commit**

```bash
git add src/simulation.rs
git commit -m "feat: implement SimulationEngine for dry-run parsing"
```

---

### Task 4: Implement Health Watchdog

**Files:**
- Create: `src/watchdog.rs`

- [ ] **Step 1: Implement HealthWatchdog**

```rust
pub struct HealthWatchdog {
    timeout: Duration,
}

impl HealthWatchdog {
    pub async fn check_mirrors(&self, mirrors: &[String]) -> Result<Vec<MirrorHealth>> {
        // Parallel pings using reqwest
    }
    pub async fn check_gpg_keys(&self) -> Result<bool> {
        // Run "pacman-key --list-keys" or equivalent and check for [expired]
    }
}
```

- [ ] **Step 2: Commit**

```bash
git add src/watchdog.rs
git commit -m "feat: implement HealthWatchdog for system pre-checks"
```

---

### Task 5: Implement Transaction Manager (The Orchestrator)

**Files:**
- Create: `src/transaction_manager.rs`

- [ ] **Step 1: Define TransactionPipeline**

```rust
pub struct TransactionManager {
    snapshotter: Option<Box<dyn SnapshotProvider>>,
    simulator: SimulationEngine,
    watchdog: HealthWatchdog,
}

impl TransactionManager {
    pub async fn execute_safe<F, Fut>(&self, action_name: &str, action: F) -> Result<()>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<()>> 
    {
        // 1. Watchdog check
        // 2. Simulation
        // 3. Snapshot
        // 4. Run action
        // 5. If error -> Rollback?
    }
}
```

- [ ] **Step 2: Integrate into operation_queue.rs**

- [ ] **Step 3: Commit**

```bash
git add src/transaction_manager.rs src/operation_queue.rs
git commit -m "feat: implement TransactionManager safety pipeline"
```

---

### Task 6: UI - Simulation Modal & Rollback Dialog

**Files:**
- Modify: `src/ui.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Add UI state to App**

```rust
pub struct App {
    pub show_simulation: bool,
    pub simulation_result: Option<SimulationResult>,
    pub show_rollback_confirm: bool,
    pub pending_rollback_id: Option<String>,
}
```

- [ ] **Step 2: Implement render_simulation_modal in ui.rs**

- [ ] **Step 3: Implement render_rollback_dialog in ui.rs**

- [ ] **Step 4: Commit**

```bash
git add src/ui.rs src/app.rs
git commit -m "feat: add Simulation and Rollback UI screens"
```

---

### Task 7: Final Integration & Robustness Testing

- [ ] **Step 1: End-to-end test with a mock backend**
- [ ] **Step 2: Verify snapshot cleanup logic**
- [ ] **Step 3: Final commit**
