use crate::errors::{AppError, Result};
use crate::services::CommandSpec;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub id: String,
    pub created_at: String,
    pub installed_packages: Vec<String>,
    pub removed_packages: Vec<String>,
    pub commands: Vec<String>,
    pub status: TransactionStatus,
    pub error: Option<String>,
}

fn history_path() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("arch-tui").join("transactions.json")
}

pub fn load_history() -> Result<Vec<TransactionRecord>> {
    let path = history_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| {
        AppError::Other(format!(
            "Failed to read history '{}': {}",
            path.display(),
            e
        ))
    })?;
    let parsed = serde_json::from_str::<Vec<TransactionRecord>>(&data).map_err(|e| {
        AppError::Other(format!(
            "Failed to parse transaction history '{}': {}",
            path.display(),
            e
        ))
    })?;
    Ok(parsed)
}

pub fn save_history(history: &[TransactionRecord]) -> Result<()> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Other(format!(
                "Failed to create history directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    let payload = serde_json::to_string_pretty(history)
        .map_err(|e| AppError::Other(format!("Failed to serialize history: {}", e)))?;
    let mut file = File::create(&path).map_err(|e| {
        AppError::Other(format!(
            "Failed to create history '{}': {}",
            path.display(),
            e
        ))
    })?;
    file.write_all(payload.as_bytes()).map_err(|e| {
        AppError::Other(format!(
            "Failed to write history '{}': {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}

pub fn new_record(
    installed_packages: Vec<String>,
    removed_packages: Vec<String>,
    commands: &[CommandSpec],
) -> TransactionRecord {
    let id = format!(
        "tx-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let created_at = chrono_like_now();
    let commands = commands
        .iter()
        .map(|c| format!("{} {}", c.prog, c.args.join(" ")))
        .collect();

    TransactionRecord {
        id,
        created_at,
        installed_packages,
        removed_packages,
        commands,
        status: TransactionStatus::Pending,
        error: None,
    }
}

fn chrono_like_now() -> String {
    // Keep dependencies minimal; UNIX timestamp with seconds precision is enough.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}", d.as_secs()))
        .unwrap_or_else(|_| "0".to_string())
}
