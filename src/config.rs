use crate::theme::{ColorDef, Theme};
use config::{Config, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub aur_helper: String,
    pub theme: ThemeConfig,
    pub keyboard: KeyboardConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    pub name: String,
    pub primary_color: ColorDef,
    pub secondary_color: ColorDef,
    pub accent_color: ColorDef,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub quit: String,
    pub search: String,
    pub install: String,
    pub toggle_selection: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UiConfig {
    pub items_per_page: usize,
    pub search_debounce_ms: u64,
    pub max_search_history: usize,
    pub max_undo_history: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aur_helper: "auto".to_string(),
            theme: ThemeConfig {
                name: "dark".to_string(),
                primary_color: ColorDef::Rgb {
                    r: 100,
                    g: 150,
                    b: 255,
                },
                secondary_color: ColorDef::Rgb {
                    r: 255,
                    g: 165,
                    b: 0,
                },
                accent_color: ColorDef::Named("green".to_string()),
            },
            keyboard: KeyboardConfig {
                quit: "q".to_string(),
                search: "/".to_string(),
                install: "enter".to_string(),
                toggle_selection: "tab".to_string(),
            },
            ui: UiConfig {
                items_per_page: 20,
                search_debounce_ms: 300,
                max_search_history: 50,
                max_undo_history: 20,
            },
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        let mut cfg = Config::builder();

        // Add default configuration
        cfg = cfg.add_source(config::File::from_str(
            r#"
            aur_helper = "auto"
            
            [theme]
            name = "dark"
            primary_color = { r = 100, g = 150, b = 255 }
            secondary_color = { r = 255, g = 165, b = 0 }
            accent_color = "green"
            
            [keyboard]
            quit = "q"
            search = "/"
            install = "enter"
            toggle_selection = "tab"
            
            [ui]
            items_per_page = 20
            search_debounce_ms = 300
            max_search_history = 50
            max_undo_history = 20
        "#,
            config::FileFormat::Toml,
        ));

        // Add user configuration file if it exists
        let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

        let config_path = config_dir.join("arch-tui").join("config.toml");

        if Path::exists(&config_path) {
            cfg = cfg.add_source(File::with_name(config_path.to_str().unwrap()).required(false));
        }

        // Add environment variables as overrides
        cfg = cfg.add_source(config::Environment::with_prefix("ARCH_TUI"));

        let config: AppConfig = cfg.build()?.try_deserialize()?;

        // Validate configuration
        if let Err(e) = config.validate() {
            eprintln!("Configuration validation failed: {}", e);
            eprintln!("Using default configuration");
            return Ok(AppConfig::default());
        }

        Ok(config)
    }

    /// Validate the configuration values
    pub fn validate(&self) -> Result<(), ConfigValidationError> {
        // Validate AUR helper
        let valid_helpers = ["auto", "paru", "yay", "pacman"];
        if !valid_helpers.contains(&self.aur_helper.as_str()) {
            return Err(ConfigValidationError::InvalidAurHelper(
                self.aur_helper.clone(),
            ));
        }

        // Validate UI settings
        if self.ui.items_per_page == 0 {
            return Err(ConfigValidationError::InvalidValue(
                "items_per_page must be greater than 0".to_string(),
            ));
        }

        if self.ui.search_debounce_ms < 50 {
            return Err(ConfigValidationError::InvalidValue(
                "search_debounce_ms must be at least 50".to_string(),
            ));
        }

        if self.ui.max_search_history == 0 {
            return Err(ConfigValidationError::InvalidValue(
                "max_search_history must be greater than 0".to_string(),
            ));
        }

        if self.ui.max_undo_history == 0 {
            return Err(ConfigValidationError::InvalidValue(
                "max_undo_history must be greater than 0".to_string(),
            ));
        }

        // Validate keyboard shortcuts
        if self.keyboard.quit.is_empty() {
            return Err(ConfigValidationError::InvalidValue(
                "quit key cannot be empty".to_string(),
            ));
        }

        if self.keyboard.search.is_empty() {
            return Err(ConfigValidationError::InvalidValue(
                "search key cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the theme based on configuration
    pub fn get_theme(&self) -> Theme {
        match self.theme.name.as_str() {
            "light" => Theme::light(),
            _ => Theme::default_dark(),
        }
    }
}

/// Configuration validation errors
#[derive(Debug, Clone)]
pub enum ConfigValidationError {
    InvalidAurHelper(String),
    InvalidValue(String),
}

impl std::fmt::Display for ConfigValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValidationError::InvalidAurHelper(helper) => {
                write!(
                    f,
                    "Invalid AUR helper: '{}'. Valid options are: auto, paru, yay, pacman",
                    helper
                )
            }
            ConfigValidationError::InvalidValue(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.aur_helper, "auto");
        assert_eq!(config.ui.items_per_page, 20);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_aur_helper() {
        let config = AppConfig {
            aur_helper: "invalid".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_items_per_page() {
        let config = AppConfig {
            ui: UiConfig {
                items_per_page: 0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_config() {
        let config = AppConfig {
            aur_helper: "paru".to_string(),
            ui: UiConfig {
                items_per_page: 50,
                search_debounce_ms: 500,
                max_search_history: 100,
                max_undo_history: 50,
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
