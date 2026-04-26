use std::process::Command;

pub struct DesktopNotifier;

impl DesktopNotifier {
    pub fn new() -> Self {
        Self
    }

    pub fn send(&self, title: &str, body: &str) -> Result<(), String> {
        // Try notify-send (libnotify)
        if Command::new("which").arg("notify-send").output().map(|o| o.status.success()).unwrap_or(false) {
            let output = Command::new("notify-send")
                .args(["-i", "system-software-install", title, body])
                .output();
            if output.map(|o| o.status.success()).unwrap_or(false) {
                return Ok(());
            }
        }

        // Try kdialog (KDE)
        if Command::new("which").arg("kdialog").output().map(|o| o.status.success()).unwrap_or(false) {
            let output = Command::new("kdialog")
                .args(["--msgbox", body, title])
                .output();
            if output.map(|o| o.status.success()).unwrap_or(false) {
                return Ok(());
            }
        }

        // Try zenlist (generic)
        if Command::new("which").arg("zenity").output().map(|o| o.status.success()).unwrap_or(false) {
            let output = Command::new("zenity")
                .args(["--info", &format!("--text={}", body), &format!("--title={}", title)])
                .output();
            if output.map(|o| o.status.success()).unwrap_or(false) {
                return Ok(());
            }
        }

        Err("No notification tool available".to_string())
    }

    pub fn notify_install(&self, package_name: &str) {
        let _ = self.send("Package Installed", &format!("{} has been installed successfully.", package_name));
    }

    pub fn notify_update(&self, count: usize) {
        let _ = self.send(
            "System Updated", 
            &format!("{} packages have been updated.", count)
        );
    }

    pub fn notify_error(&self, error: &str) {
        let _ = self.send("Error", error);
    }
}

impl Default for DesktopNotifier {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DesktopNotifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DesktopNotifier").finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifier_creation() {
        let notifier = DesktopNotifier::new();
        assert!(!format!("{:?}", notifier).is_empty());
    }
}