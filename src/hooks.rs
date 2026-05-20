use crate::config::HooksConfig;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct HookConfig {
    pub pre_install: Vec<String>,
    pub post_install: Vec<String>,
    pub pre_remove: Vec<String>,
    pub post_remove: Vec<String>,
    pub pre_update: Vec<String>,
    pub post_update: Vec<String>,
}


impl From<HooksConfig> for HookConfig {
    fn from(cfg: HooksConfig) -> Self {
        Self {
            pre_install: cfg.pre_install,
            post_install: cfg.post_install,
            pre_remove: cfg.pre_remove,
            post_remove: cfg.post_remove,
            pre_update: cfg.pre_update,
            post_update: cfg.post_update,
        }
    }
}

pub struct HookRunner {
    config: HookConfig,
}

impl HookRunner {
    /// Create a HookRunner from the application's HooksConfig
    pub fn from_config(config: &HooksConfig) -> Self {
        Self {
            config: HookConfig::from(config.clone()),
        }
    }

    fn run_hook(&self, hook: &str) -> Result<String, String> {
        tracing::info!("Running hook: {}", hook);

        let shell = if cfg!(windows) { "cmd" } else { "sh" };
        let flag = if cfg!(windows) { "/C" } else { "-c" };

        let output = Command::new(shell)
            .arg(flag)
            .arg(hook)
            .output()
            .map_err(|e| format!("Failed to run hook '{}': {}", hook, e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            tracing::info!("Hook succeeded: {}", hook);
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            tracing::warn!("Hook failed: {} — {}", hook, stderr);
            Err(stderr)
        }
    }

    fn run_hooks(&self, hooks: &[String], phase: &str) -> Vec<Result<String, String>> {
        if hooks.is_empty() {
            return Vec::new();
        }
        tracing::info!("Running {} hooks ({} total)", phase, hooks.len());
        hooks.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_pre_install(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.pre_install, "pre-install")
    }

    pub fn run_post_install(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.post_install, "post-install")
    }

    pub fn run_pre_remove(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.pre_remove, "pre-remove")
    }

    pub fn run_post_remove(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.post_remove, "post-remove")
    }

    pub fn run_pre_update(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.pre_update, "pre-update")
    }

    pub fn run_post_update(&self) -> Vec<Result<String, String>> {
        self.run_hooks(&self.config.post_update, "post-update")
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_config_default() {
        let config = HookConfig::default();
        assert!(config.pre_install.is_empty());
        assert!(config.post_install.is_empty());
    }

    #[test]
    fn test_from_hooks_config() {
        let hooks_config = HooksConfig {
            pre_install: vec!["echo pre".to_string()],
            post_install: vec!["echo post".to_string()],
            pre_remove: vec![],
            post_remove: vec![],
            pre_update: vec![],
            post_update: vec![],
        };
        let hook_config = HookConfig::from(hooks_config);
        assert_eq!(hook_config.pre_install.len(), 1);
        assert_eq!(hook_config.post_install.len(), 1);
    }
}
