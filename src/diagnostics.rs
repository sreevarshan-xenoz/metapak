use std::process::Command;

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub label: String,
    pub status: String,
}

pub fn run_diagnostics() -> Vec<DiagnosticItem> {
    let mut items = Vec::new();

    items.push(DiagnosticItem {
        label: "pacman binary".to_string(),
        status: if command_exists("pacman") {
            "OK".to_string()
        } else {
            "MISSING".to_string()
        },
    });

    let aur_helper = if command_exists("paru") {
        "paru"
    } else if command_exists("yay") {
        "yay"
    } else {
        "none"
    };
    items.push(DiagnosticItem {
        label: "AUR helper".to_string(),
        status: aur_helper.to_string(),
    });

    let lock_exists = std::path::Path::new("/var/lib/pacman/db.lck").exists();
    items.push(DiagnosticItem {
        label: "pacman db lock".to_string(),
        status: if lock_exists {
            "LOCKED".to_string()
        } else {
            "clear".to_string()
        },
    });

    items.push(DiagnosticItem {
        label: "disk space /".to_string(),
        status: disk_usage_root().unwrap_or_else(|| "unknown".to_string()),
    });

    items
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn disk_usage_root() -> Option<String> {
    let output = Command::new("df").arg("-h").arg("/").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 5 {
        return None;
    }
    Some(format!("{} used", cols[4]))
}
