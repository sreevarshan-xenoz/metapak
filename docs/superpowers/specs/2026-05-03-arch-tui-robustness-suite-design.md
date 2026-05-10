# Design Doc: Arch-TUI Robustness & Safety Suite

**Date:** 2026-05-03
**Status:** Approved
**Topic:** Implementing a mission-critical safety layer for Arch-TUI including snapshots, simulations, and health monitoring.

## 1. Purpose
The goal of this suite is to transform Arch-TUI from a package management convenience tool into a robust system utility that ensures system stability. It provides a "Never-Break" guarantee by integrating filesystem snapshots, predictive dry-runs, and real-time health monitoring.

## 2. Architecture

### 2.1 Transaction Middleware
We will implement a centralized `TransactionManager` that wraps all system-modifying actions (Install, Remove, Update, Upgrade). This manager orchestrates the safety pipeline.

### 2.2 Snapshot Engine
A pluggable architecture using the `SnapshotProvider` trait:
```rust
pub trait SnapshotProvider {
    async fn create(&self, label: &str) -> Result<String>;
    async fn rollback(&self, id: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<Snapshot>>;
    async fn cleanup(&self, keep_count: usize) -> Result<()>;
}
```
**Supported Providers:**
- **BTRFS:** Native subvolume snapshots.
- **Timeshift:** Integration with the Timeshift CLI.
- **LVM:** Logical volume snapshots.
- **ZFS:** Dataset snapshots.

### 2.3 Simulation Layer
A `SimulationEngine` that runs package manager operations with dry-run flags (e.g., `pacman -Sp`, `apt install --dry-run`).
- **Disk Space Projection:** Calculates required vs. available space.
- **Conflict Detection:** Parses stderr for file conflicts or broken dependencies.
- **Configuration Analysis:** Identifies if system config files in `/etc` will be modified.

### 2.4 Health Watchdog
A background monitor that checks:
- **Mirror Latency:** Pings configured mirrors to ensure high-speed downloads.
- **GPG Integrity:** Verifies that package repository keys are valid and not expired.
- **DB Locks:** Ensures no other package manager is running (preventing `db.lck` errors).

## 3. Workflow (The Safe Pipeline)
1. **Initiate:** User selects packages and confirms.
2. **Health Check:** Watchdog verifies mirrors and GPG keys.
3. **Simulate:** Dry-run calculates impact. If critical conflicts are found, the operation is blocked with a detailed report.
4. **Snapshot:** If safety is enabled, a pre-transaction snapshot is created (e.g., `arch-tui-pre-20260503-1430`).
5. **Execute:** The backend command is executed with real-time logging.
6. **Handle:**
   - **Success:** The transaction is logged; the snapshot is marked for auto-cleanup (default 24h).
   - **Failure:** A "Rollback Dialog" is presented, offering a 1-click reversion to the pre-transaction state.

## 4. UI/UX Changes
- **Simulation Summary:** A new modal before execution showing net disk change and conflict warnings.
- **Safety Indicator:** A status bar icon showing mirror health and snapshot availability.
- **Rollback Screen:** A dedicated error handling screen with a prominent "Rollback" button.

## 5. Implementation Strategy
- Phase 1: Implement `SnapshotProvider` trait and BTRFS/Timeshift providers.
- Phase 2: Create `TransactionManager` and refactor existing actions to use the pipeline.
- Phase 3: Build the `SimulationEngine` and `HealthWatchdog`.
- Phase 4: Integrated UI/UX for rollback and simulation summaries.

## 6. Testing Strategy
- **Unit Tests:** Mock `SnapshotProvider` to verify `TransactionManager` logic.
- **Integration Tests:** Run dry-runs against a mock filesystem.
- **Robustness Tests:** Simulate failed installs and verify that `rollback()` is called with correct parameters.
