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
    pub preset: String,
    pub primary_color: Option<ColorDef>,
    pub secondary_color: Option<ColorDef>,
    pub accent_color: Option<ColorDef>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub quit: String,
    pub search: String,
    pub install: String,
    pub toggle_selection: String,
    pub next_page: String,
    pub prev_page: String,
    pub next: String,
    pub prev: String,
    pub help: String,
    pub history: String,
    pub diagnostics: String,
    pub filter: String,
    pub sort: String,
    pub undo: String,
    pub details: String,
    pub dependencies: String,
    pub sidebar: String,
    pub refresh: String,
    pub update: String,
    pub rollback: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct UiConfig {
    pub items_per_page: usize,
    pub search_debounce_ms: u64,
    pub max_search_history: usize,
    pub max_undo_history: usize,
    pub auto_check_updates: bool,
    pub update_check_interval_minutes: u64,
    pub auto_update_on_startup: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aur_helper: "auto".to_string(),
            theme: ThemeConfig {
                preset: "mocha".to_string(),
                primary_color: None,
                secondary_color: None,
                accent_color: None,
            },
            keyboard: KeyboardConfig {
                quit: "q".to_string(),
                search: "/".to_string(),
                install: "enter".to_string(),
                toggle_selection: "tab".to_string(),
                next_page: "n".to_string(),
                prev_page: "p".to_string(),
                next: "j".to_string(),
                prev: "k".to_string(),
                help: "?".to_string(),
                history: "t".to_string(),
                diagnostics: "h".to_string(),
                filter: "f".to_string(),
                sort: "s".to_string(),
                undo: "u".to_string(),
                details: "d".to_string(),
                dependencies: "v".to_string(),
                sidebar: "\\".to_string(),
                refresh: "r".to_string(),
                update: "U".to_string(),
                rollback: "R".to_string(),
            },
            ui: UiConfig {
                items_per_page: 20,
                search_debounce_ms: 50,
                max_search_history: 50,
                max_undo_history: 20,
                auto_check_updates: false,
                update_check_interval_minutes: 60,
                auto_update_on_startup: false,
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
            preset = "mocha"
            primary_color = "blue"
            secondary_color = "yellow"
            accent_color = "green"
            
            [keyboard]
            quit = "q"
            search = "/"
            install = "enter"
            toggle_selection = "tab"
            next_page = "n"
            prev_page = "p"
            next = "j"
            prev = "k"
            help = "?"
            history = "t"
            diagnostics = "h"
            filter = "f"
            sort = "s"
            undo = "u"
            details = "d"
            dependencies = "v"
            sidebar = "\\"
            refresh = "r"
            update = "U"
            rollback = "R"
            
            [ui]
            items_per_page = 20
            search_debounce_ms = 300
            max_search_history = 50
            max_undo_history = 20
            auto_check_updates = false
            update_check_interval_minutes = 60
            auto_update_on_startup = false
        "#,
            config::FileFormat::Toml,
        ));

        // Add user configuration file if it exists
        let config_dir = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));

        let config_path = config_dir.join("arch-tui").join("config.toml");

        if Path::exists(&config_path) {
            if let Some(path) = config_path.to_str() {
                cfg = cfg.add_source(File::with_name(path).required(false));
            }
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
        let mut theme = match self.theme.preset.as_str() {
            "latte" | "light" => Theme::catppuccin_latte(),
            "dark" | "mocha" => Theme::catppuccin_mocha(),
            _ => Theme::default(),
        };

        if let Some(ref color) = self.theme.primary_color {
            theme.primary = color.clone();
            theme.highlight_bg = color.clone();
        }
        if let Some(ref color) = self.theme.secondary_color {
            theme.secondary = color.clone();
            theme.aur_color = color.clone();
        }
        if let Some(ref color) = self.theme.accent_color {
            theme.success = color.clone();
        }
        theme
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
                auto_check_updates: false,
                update_check_interval_minutes: 60,
                auto_update_on_startup: false,
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
