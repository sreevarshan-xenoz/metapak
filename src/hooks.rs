use crate::config::HooksConfig;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub pre_install: Vec<String>,
    pub post_install: Vec<String>,
    pub pre_remove: Vec<String>,
    pub post_remove: Vec<String>,
    pub pre_update: Vec<String>,
    pub post_update: Vec<String>,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            pre_install: vec![],
            post_install: vec![],
            pre_remove: vec![],
            post_remove: vec![],
            pre_update: vec![],
            post_update: vec![],
        }
    }
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
    pub fn new(config: HookConfig) -> Self {
        Self { config }
    }

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

    /// Check if any hooks are configured
    pub fn has_hooks(&self) -> bool {
        !self.config.pre_install.is_empty()
            || !self.config.post_install.is_empty()
            || !self.config.pre_remove.is_empty()
            || !self.config.post_remove.is_empty()
            || !self.config.pre_update.is_empty()
            || !self.config.post_update.is_empty()
    }
}

pub fn load_hooks_from_config() -> HookConfig {
    // Load from AppConfig if available, otherwise return defaults
    match crate::config::AppConfig::load() {
        Ok(config) => HookConfig::from(config.hooks),
        Err(_) => HookConfig::default(),
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
    fn test_hook_runner_empty() {
        let runner = HookRunner::new(HookConfig::default());
        assert!(runner.run_pre_install().is_empty());
        assert!(!runner.has_hooks());
    }

    #[test]
    fn test_hook_runner_has_hooks() {
        let config = HookConfig {
            pre_install: vec!["echo hello".to_string()],
            ..Default::default()
        };
        let runner = HookRunner::new(config);
        assert!(runner.has_hooks());
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
