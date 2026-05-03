use crate::errors::{AppError, Result};
use crate::traits::SnapshotProvider;
use crate::simulation::SimulationEngine;
use crate::watchdog::HealthWatchdog;
use std::sync::Arc;
use tracing;

/// Orchestrates safe package transactions with health checks, simulations, and snapshots.
pub struct TransactionManager {
    snapshot_provider: Option<Arc<dyn SnapshotProvider>>,
    _simulator: Arc<SimulationEngine>,
    watchdog: Arc<HealthWatchdog>,
}

impl TransactionManager {
    /// Create a new TransactionManager
    pub fn new(
        snapshot_provider: Option<Arc<dyn SnapshotProvider>>,
        simulator: Arc<SimulationEngine>,
        watchdog: Arc<HealthWatchdog>,
    ) -> Self {
        Self {
            snapshot_provider,
            _simulator: simulator,
            watchdog,
        }
    }

    /// Execute an action within a safe transaction pipeline
    pub async fn run_safe_transaction<F, Fut, T>(&self, action_name: &str, action: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        tracing::info!(action = %action_name, "Starting safe transaction");

        // 1. Health Check (Watchdog)
        if self.watchdog.check_db_lock().await? {
            tracing::error!("Package database is locked");
            return Err(AppError::Backend("Package database is locked by another process".to_string()));
        }

        // 2. Simulation (Optional for now)
        tracing::debug!("Transaction simulation phase");

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
                Ok(val)
            }
            Err(e) => {
                tracing::error!(
                    action = %action_name,
                    error = %e,
                    "CRITICAL: Transaction failed"
                );
                
                if let Some(id) = snapshot_id {
                    tracing::warn!(
                        snapshot_id = %id,
                        "System may be in an inconsistent state. A snapshot is available for rollback."
                    );
                }
                
                Err(e)
            }
        }
    }
}
