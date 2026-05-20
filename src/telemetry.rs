use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn log_dir() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("metapak")
}

fn log_path() -> PathBuf {
    log_dir().join("operations.log")
}

/// Rotate logs if the current log exceeds the max size.
/// Keeps up to `max_files` rotated logs (operations.log.1, .2, etc.)
pub fn rotate_logs(max_size_mb: u64, max_files: usize) {
    let path = log_path();
    if !path.exists() {
        return;
    }

    let max_bytes = max_size_mb * 1024 * 1024;

    if let Ok(metadata) = fs::metadata(&path) {
        if metadata.len() < max_bytes {
            return;
        }
    }

    tracing::info!("Rotating telemetry log (exceeded {}MB)", max_size_mb);

    // Shift existing rotated files: .4 -> delete, .3 -> .4, .2 -> .3, etc.
    let dir = log_dir();
    for i in (1..max_files).rev() {
        let from = dir.join(format!("operations.log.{}", i));
        let to = dir.join(format!("operations.log.{}", i + 1));
        if from.exists() {
            if i + 1 >= max_files {
                let _ = fs::remove_file(&from);
            } else {
                let _ = fs::rename(&from, &to);
            }
        }
    }

    // Move current log to .1
    let rotated = dir.join("operations.log.1");
    let _ = fs::rename(&path, &rotated);
}

/// Append a structured log line with timestamp
pub fn append_log_line(line: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(&path) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(file, "[{}] {}", timestamp, line);
        let _ = file.flush();
    }
}

pub fn flush() {
    // Force sync to ensure all pending writes are flushed
    let path = log_path();
    if let Ok(file) = OpenOptions::new().append(true).open(&path) {
        let _ = file.sync_all();
    }
    tracing::debug!("Telemetry flushed");
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_log_path_is_valid() {
        let path = log_path();
        assert!(path.to_str().unwrap().contains("metapak"));
    }
}
