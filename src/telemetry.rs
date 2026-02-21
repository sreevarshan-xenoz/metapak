use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn log_path() -> PathBuf {
    let base = dirs::state_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    base.join("arch-tui").join("operations.log")
}

pub fn append_log_line(line: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
        let _ = writeln!(file, "{}", line);
    }
}
