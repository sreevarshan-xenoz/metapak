//! Health monitoring and circuit breaker.
//!
//! Monitors application health, tracks AUR API failures, and provides
//! circuit breaker protection against cascading service failures.

use crate::errors::Result;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DiskHealth {
    pub mount_point: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f64,
}

#[allow(dead_code)]
pub struct HealthWatchdog {
    _ping_timeout: Duration,
}

impl HealthWatchdog {
    #[allow(dead_code)]
    pub fn new(ping_timeout: Duration) -> Self {
        Self {
            _ping_timeout: ping_timeout,
        }
    }

    pub async fn check_db_lock(&self) -> Result<bool> {
        Ok(Path::new("/var/lib/pacman/db.lck").exists())
    }
}

#[derive(Debug, Clone)]
pub struct PacmanStatus {
    pub db_locked: bool,
    pub gpg_valid: bool,
}
