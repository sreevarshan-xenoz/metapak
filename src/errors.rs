use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Pacman command failed: {0}")]
    Pacman(String),

    #[error("AUR command failed: {0}")]
    Aur(String),

    #[error("NPM command failed: {0}")]
    Npm(String),

    #[error("Sudo authentication failed")]
    SudoAuthFailed,

    #[error("Command execution failed: {0}")]
    Command(String),

    #[error("Dependency resolution failed: {0}")]
    Dependency(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Configuration validation error: {0}")]
    ConfigValidation(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Input validation failed: {0}")]
    Validation(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String, Option<String>),

    #[error("Other error: {0}")]
    Other(String),

    #[error("Cancelled by user")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<crate::config::ConfigValidationError> for AppError {
    fn from(err: crate::config::ConfigValidationError) -> Self {
        AppError::ConfigValidation(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        // Test IO error conversion
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        match app_err {
            AppError::Io(_) => {}
            _ => panic!("Expected Io error"),
        }

        // Test JSON error conversion
        let invalid_json = "invalid json {";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let app_err: AppError = json_err.into();
        match app_err {
            AppError::Json(_) => {}
            _ => panic!("Expected Json error"),
        }

        // Test config error conversion
        let config_err = config::ConfigError::NotFound("key".to_string());
        let app_err: AppError = config_err.into();
        match app_err {
            AppError::Config(_) => {}
            _ => panic!("Expected Config error"),
        }
    }

    #[test]
    fn test_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        let error_msg = format!("{}", app_err);
        assert!(error_msg.contains("IO error"));
        assert!(error_msg.contains("file not found"));

        let pacman_err = AppError::Pacman("command failed".to_string());
        let error_msg = format!("{}", pacman_err);
        assert!(error_msg.contains("Pacman command failed"));
        assert!(error_msg.contains("command failed"));

        let cancelled = AppError::Cancelled;
        assert_eq!(format!("{}", cancelled), "Cancelled by user");
    }

    #[test]
    fn test_custom_errors() {
        let aur_err = AppError::Aur("network timeout".to_string());
        assert!(format!("{}", aur_err).contains("AUR command failed"));

        let sudo_err = AppError::SudoAuthFailed;
        assert_eq!(format!("{}", sudo_err), "Sudo authentication failed");

        let cmd_err = AppError::Command("not found".to_string());
        assert!(format!("{}", cmd_err).contains("Command execution failed"));
    }
}
