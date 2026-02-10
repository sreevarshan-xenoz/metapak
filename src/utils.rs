//! Utility functions for secure operations
//!
//! This module provides secure password handling and other security-related utilities.

use crate::config::AppConfig;
use secrecy::{ExposeSecret, SecretString};
use std::io::Write;
use std::process::{Command, Stdio};

/// Securely verify sudo password
pub fn check_sudo_password(password: &SecretString) -> bool {
    let mut child = match Command::new("sudo")
        .arg("-S")
        .arg("-v")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            tracing::error!("Failed to spawn sudo process: {}", e);
            return false;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        if let Err(e) = writeln!(stdin, "{}", password.expose_secret()) {
            tracing::error!("Failed to write password to stdin: {}", e);
            return false;
        }
    }

    match child.wait() {
        Ok(status) => {
            let success = status.success();
            if !success {
                tracing::warn!("Sudo authentication failed");
            }
            success
        }
        Err(e) => {
            tracing::error!("Failed to wait for sudo process: {}", e);
            false
        }
    }
}

/// Secure password input with masking
pub struct PasswordInput {
    value: SecretString,
}

impl PasswordInput {
    pub fn new() -> Self {
        Self {
            value: SecretString::new(String::new()),
        }
    }

    pub fn from_string(s: String) -> Self {
        Self {
            value: SecretString::new(s),
        }
    }

    pub fn push(&mut self, c: char) {
        let mut current = self.value.expose_secret().to_string();
        current.push(c);
        self.value = SecretString::new(current);
    }

    pub fn pop(&mut self) {
        let mut current = self.value.expose_secret().to_string();
        current.pop();
        self.value = SecretString::new(current);
    }

    pub fn len(&self) -> usize {
        self.value.expose_secret().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.value = SecretString::new(String::new());
    }

    pub fn get_secret(&self) -> &SecretString {
        &self.value
    }

    pub fn expose_secret(&self) -> &str {
        self.value.expose_secret()
    }

    /// Get masked representation for display
    pub fn masked(&self) -> String {
        format!("{}█", "*".repeat(self.len()))
    }
}

impl Default for PasswordInput {
    fn default() -> Self {
        Self::new()
    }
}

/// AUR helper detection and command building
pub struct AurHelper {
    helper: String,
}

impl AurHelper {
    pub fn detect(config: &AppConfig) -> Self {
        let helper = Self::resolve_helper(&config.aur_helper);
        Self { helper }
    }

    fn resolve_helper(configured: &str) -> String {
        match configured {
            "paru" => {
                if Self::command_exists("paru") {
                    "paru".to_string()
                } else {
                    tracing::warn!("paru not found, falling back to pacman");
                    "sudo pacman".to_string()
                }
            }
            "yay" => {
                if Self::command_exists("yay") {
                    "yay".to_string()
                } else {
                    tracing::warn!("yay not found, falling back to pacman");
                    "sudo pacman".to_string()
                }
            }
            "pacman" => "pacman".to_string(),
            "auto" | _ => {
                if Self::command_exists("paru") {
                    tracing::debug!("Using paru as AUR helper");
                    "paru".to_string()
                } else if Self::command_exists("yay") {
                    tracing::debug!("Using yay as AUR helper");
                    "yay".to_string()
                } else {
                    tracing::warn!("No AUR helper found, using pacman only");
                    "sudo pacman".to_string()
                }
            }
        }
    }

    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn helper(&self) -> &str {
        &self.helper
    }

    pub fn build_install_command(&self, packages: &[String]) -> String {
        if self.helper == "sudo pacman" {
            format!("sudo pacman -S --noconfirm {}", packages.join(" "))
        } else {
            format!("{} -S --noconfirm {}", self.helper, packages.join(" "))
        }
    }

    pub fn build_remove_command(&self, packages: &[String]) -> String {
        format!("sudo pacman -Rns --noconfirm {}", packages.join(" "))
    }

    pub fn build_update_command(&self) -> String {
        if self.helper == "sudo pacman" {
            "sudo pacman -Syu --noconfirm".to_string()
        } else {
            format!("{} -Syu --noconfirm", self.helper)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_input() {
        let mut pwd = PasswordInput::new();
        assert!(pwd.is_empty());

        pwd.push('a');
        pwd.push('b');
        pwd.push('c');

        assert_eq!(pwd.len(), 3);
        assert_eq!(pwd.expose_secret(), "abc");
        assert_eq!(pwd.masked(), "***█");

        pwd.pop();
        assert_eq!(pwd.len(), 2);
        assert_eq!(pwd.expose_secret(), "ab");

        pwd.clear();
        assert!(pwd.is_empty());
    }

    #[test]
    fn test_password_input_from_string() {
        let pwd = PasswordInput::from_string("test123".to_string());
        assert_eq!(pwd.len(), 7);
        assert_eq!(pwd.masked(), "*******█");
    }
}
