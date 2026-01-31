use std::process::{Command, Stdio};
use std::io::Write;
use crate::errors::AppError;
use crate::config::AppConfig;

pub fn get_aur_helper(configured_helper: Option<&str>) -> &'static str {
    match configured_helper {
        Some("paru") => {
            if check_command("paru") {
                "paru"
            } else {
                "pacman"
            }
        },
        Some("yay") => {
            if check_command("yay") {
                "yay"
            } else {
                "pacman"
            }
        },
        Some("pacman") => "pacman",
        Some("auto") | None => {
            if check_command("paru") {
                "paru"
            } else if check_command("yay") {
                "yay"
            } else {
                "pacman"
            }
        },
        _ => "pacman", // Default fallback
    }
}

fn check_command(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn check_sudo_password(password: &str) -> bool {
    let child = Command::new("sudo")
        .arg("-S")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match child {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = writeln!(stdin, "{}", password);
            }
            match child.wait() {
                Ok(status) => status.success(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
