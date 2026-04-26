use crate::errors::AppError;

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub package: String,
    pub status: InstallStatus,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    Pending,
    Running,
    Success,
    Failed,
}

pub struct ParallelInstaller {
    max_concurrent: usize,
}

impl ParallelInstaller {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1).min(10),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1).min(10);
    }

    pub fn install_packages(
        &self,
        packages: &[String],
    ) -> Vec<InstallProgress> {
        packages
            .iter()
            .map(|pkg| {
                let status = if Self::install_sync(pkg).is_ok() {
                    InstallStatus::Success
                } else {
                    InstallStatus::Failed
                };

                InstallProgress {
                    package: pkg.clone(),
                    status,
                    output: String::new(),
                }
            })
            .collect()
    }

    fn install_sync(package: &str) -> Result<(), AppError> {
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("echo Installing {}...", package))
            .output()
            .map_err(|e| AppError::Other(e.to_string()))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(AppError::Other("Install failed".to_string()))
        }
    }
}

impl Default for ParallelInstaller {
    fn default() -> Self {
        Self::new(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_installer_default() {
        let installer = ParallelInstaller::default();
        assert!(installer.max_concurrent() > 0);
    }

    #[test]
    fn test_parallel_installer_max() {
        let mut installer = ParallelInstaller::new(1);
        installer.set_max_concurrent(10);
        assert_eq!(installer.max_concurrent(), 10);
    }

    #[test]
    fn test_parallel_installer_clamped() {
        let installer = ParallelInstaller::new(100);
        assert!(installer.max_concurrent() <= 10);
    }
}