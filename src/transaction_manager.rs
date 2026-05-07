use crate::errors::{AppError, Result};
use crate::services::CommandSpec;
use crate::simulation::SimulationEngine;
use crate::traits::{PackageSimulator, SnapshotProvider};
use crate::watchdog::HealthWatchdog;
use std::sync::Arc;
use tracing;

/// Orchestrates safe package transactions with health checks, simulations, and snapshots.
pub struct TransactionManager {
    snapshot_provider: Option<Arc<dyn SnapshotProvider>>,
    simulator: Arc<SimulationEngine>,
    watchdog: Arc<HealthWatchdog>,
    keep_count: usize,
}

impl TransactionManager {
    /// Create a new TransactionManager
    pub fn new(
        snapshot_provider: Option<Arc<dyn SnapshotProvider>>,
        simulator: Arc<SimulationEngine>,
        watchdog: Arc<HealthWatchdog>,
        keep_count: usize,
    ) -> Self {
        Self {
            snapshot_provider,
            simulator,
            watchdog,
            keep_count,
        }
    }

    /// Get the snapshot provider if one is available
    pub fn get_snapshot_provider(&self) -> Option<Arc<dyn SnapshotProvider>> {
        self.snapshot_provider.clone()
    }

    /// Execute an action within a safe transaction pipeline
    pub async fn run_safe_transaction<F, Fut, T>(
        &self,
        action_name: &str,
        commands: Option<&[CommandSpec]>,
        action: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        tracing::info!(action = %action_name, "Starting safe transaction");

        // 1. Health Check (Watchdog)
        if self.watchdog.check_db_lock().await? {
            tracing::error!("Package database is locked");
            return Err(AppError::Backend(
                "Package database is locked by another process".to_string(),
            ));
        }

        if !self.watchdog.check_gpg_keys().await? {
            tracing::error!("GPG keys are expired or invalid");
            return Err(AppError::Backend(
                "Package GPG keys are invalid. Try: sudo pacman-key --refresh-keys".to_string(),
            ));
        }

        // Check mirrors (Warning only, don't fail unless all are down?)
        // For now, just log the health status.
        if let Ok(mirrors) = self
            .watchdog
            .check_mirrors(&["https://archlinux.org/mirrors/status/json/".to_string()])
            .await
        {
            let reachable_count = mirrors.iter().filter(|m| m.reachable).count();
            if reachable_count == 0 {
                tracing::warn!("No reachable mirrors detected. Operation might fail.");
            } else {
                tracing::info!(reachable = reachable_count, "Mirror health check passed");
            }
        }

        // 2. Simulation
        if let Some(cmds) = commands {
            tracing::info!("Running pre-transaction simulation...");
            // Extract package names from commands like "sudo pacman -S pkg1 pkg2"
            let mut packages = Vec::new();
            for cmd in cmds {
                if cmd.prog.contains("pacman") || cmd.prog == "sudo" {
                    // Very simple heuristic: anything that doesn't start with - is a package
                    for arg in &cmd.args {
                        if !arg.starts_with('-')
                            && arg != "pacman"
                            && arg != "sudo"
                            && arg != "-S"
                            && arg != "-R"
                            && arg != "-U"
                        {
                            packages.push(arg.as_str());
                        }
                    }
                }
            }

            if !packages.is_empty() {
                let sim_res = self.simulator.simulate_install(&packages).await?;
                if !sim_res.conflicts.is_empty() {
                    let conflicts = sim_res.conflicts.join(", ");
                    tracing::error!(conflicts = %conflicts, "Simulation detected conflicts, aborting");
                    return Err(AppError::Backend(format!(
                        "Transaction blocked by conflicts: {}",
                        conflicts
                    )));
                }
                tracing::info!("Simulation successful: no conflicts detected");
            }
        }

        // 3. Snapshot (if provider exists)
        let snapshot_id = if let Some(p) = &self.snapshot_provider {
            tracing::info!("Creating pre-transaction snapshot...");
            match p.create(action_name).await {
                Ok(id) => {
                    tracing::info!(snapshot_id = %id, "Snapshot created successfully");
                    Some(id)
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to create snapshot, aborting transaction");
                    return Err(e);
                }
            }
        } else {
            None
        };

        // 4. Run Action
        let result = action().await;

        // 5. Handle success/failure
        match result {
            Ok(val) => {
                tracing::info!(action = %action_name, "Transaction completed successfully");

                // Cleanup old snapshots
                if let Some(p) = &self.snapshot_provider {
                    if let Err(e) = p.cleanup(self.keep_count).await {
                        tracing::warn!(error = %e, "Failed to cleanup old snapshots");
                    }
                }

                Ok(val)
            }
            Err(e) => {
                tracing::error!(
                    action = %action_name,
                    error = %e,
                    "CRITICAL: Transaction failed"
                );

                if let Some(id) = &snapshot_id {
                    tracing::warn!(
                        snapshot_id = %id,
                        "System may be in an inconsistent state. A snapshot is available for rollback."
                    );
                }

                Err(AppError::TransactionFailed(e.to_string(), snapshot_id))
            }
        }
    }
}
