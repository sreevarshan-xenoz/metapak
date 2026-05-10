use crate::errors::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub search_history: Vec<String>,
    pub recent_searches: Vec<String>,
    pub selected_packages: Vec<String>,
    pub favorite_packages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationState {
    pub session: SessionState,
    pub last_search: Option<String>,
    pub last_filter: Option<String>,
    pub last_sort: Option<String>,
}

impl Default for ApplicationState {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationState {
    pub fn new() -> Self {
        Self {
            session: SessionState {
                search_history: Vec::new(),
                recent_searches: Vec::new(),
                selected_packages: Vec::new(),
                favorite_packages: Vec::new(),
            },
            last_search: None,
            last_filter: None,
            last_sort: None,
        }
    }

    pub fn from_search_history(history: &VecDeque<String>) -> Self {
        let recent: Vec<String> = history.iter().take(50).cloned().collect();
        Self {
            session: SessionState {
                search_history: recent.clone(),
                recent_searches: recent,
                selected_packages: Vec::new(),
                favorite_packages: Vec::new(),
            },
            last_search: None,
            last_filter: None,
            last_sort: None,
        }
    }

    pub fn to_search_history(&self) -> VecDeque<String> {
        self.session.recent_searches.iter().cloned().collect()
    }
}

fn state_path() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("metapak").join("session.json")
}

pub fn load_state() -> Result<ApplicationState> {
    let path = state_path();
    if !path.exists() {
        return Ok(ApplicationState::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| {
        AppError::Other(format!("Failed to read state '{}': {}", path.display(), e))
    })?;
    let parsed = serde_json::from_str::<ApplicationState>(&data).map_err(|e| {
        AppError::Other(format!("Failed to parse state '{}': {}", path.display(), e))
    })?;
    Ok(parsed)
}

pub fn save_state(state: &ApplicationState) -> Result<()> {
    let path = state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Other(format!(
                "Failed to create state directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    let payload = serde_json::to_string_pretty(state)
        .map_err(|e| AppError::Other(format!("Failed to serialize state: {}", e)))?;
    let mut file = File::create(&path).map_err(|e| {
        AppError::Other(format!(
            "Failed to create state '{}': {}",
            path.display(),
            e
        ))
    })?;
    file.write_all(payload.as_bytes()).map_err(|e| {
        AppError::Other(format!("Failed to write state '{}': {}", path.display(), e))
    })?;
    Ok(())
}

pub fn clear_state() -> Result<()> {
    let path = state_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| {
            AppError::Other(format!(
                "Failed to remove state '{}': {}",
                path.display(),
                e
            ))
        })?;
    }
    Ok(())
}
