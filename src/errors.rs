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

    #[error("Sudo authentication failed")]
    SudoAuthFailed,

    #[error("Command execution failed: {0}")]
    Command(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Other error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_conversions() {
        // Test IO error conversion
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        match app_err {
            AppError::Io(_) => {}, // Expected
            _ => panic!("Expected Io error"),
        }

        // Test JSON error conversion
        let invalid_json = "invalid json {";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let app_err: AppError = json_err.into();
        match app_err {
            AppError::Json(_) => {}, // Expected
            _ => panic!("Expected Json error"),
        }

        // Test config error conversion
        let config_err = config::ConfigError::NotFound("key".to_string());
        let app_err: AppError = config_err.into();
        match app_err {
            AppError::Config(_) => {}, // Expected
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
    }
}