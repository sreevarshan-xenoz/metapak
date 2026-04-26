use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

pub struct HookRunner {
    config: HookConfig,
}

impl HookRunner {
    pub fn new(config: HookConfig) -> Self {
        Self { config }
    }

    fn run_hook(&self, hook: &str) -> Result<String, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(hook)
            .output()
            .map_err(|e| format!("Failed to run hook: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    pub fn run_pre_install(&self) -> Vec<Result<String, String>> {
        self.config.pre_install.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_post_install(&self) -> Vec<Result<String, String>> {
        self.config.post_install.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_pre_remove(&self) -> Vec<Result<String, String>> {
        self.config.pre_remove.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_post_remove(&self) -> Vec<Result<String, String>> {
        self.config.post_remove.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_pre_update(&self) -> Vec<Result<String, String>> {
        self.config.pre_update.iter().map(|h| self.run_hook(h)).collect()
    }

    pub fn run_post_update(&self) -> Vec<Result<String, String>> {
        self.config.post_update.iter().map(|h| self.run_hook(h)).collect()
    }
}

pub fn load_hooks_from_config() -> HookConfig {
    HookConfig::default()
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
    }
}