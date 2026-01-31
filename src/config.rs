use config::{Config, File};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub aur_helper: String,
    pub theme: ThemeConfig,
    pub keyboard: KeyboardConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ThemeConfig {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeyboardConfig {
    pub quit: String,
    pub search: String,
    pub install: String,
    pub toggle_selection: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aur_helper: "auto".to_string(), // auto-detect paru/yay/pacman
            theme: ThemeConfig {
                primary_color: "blue".to_string(),
                secondary_color: "yellow".to_string(),
                accent_color: "green".to_string(),
            },
            keyboard: KeyboardConfig {
                quit: "q".to_string(),
                search: "/".to_string(),
                install: "enter".to_string(),
                toggle_selection: "tab".to_string(),
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
            primary_color = "blue"
            secondary_color = "yellow"
            accent_color = "green"
            
            [keyboard]
            quit = "q"
            search = "/"
            install = "enter"
            toggle_selection = "tab"
        "#,
            config::FileFormat::Toml,
        ));

        // Add user configuration file if it exists
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        
        let config_path = config_dir.join("arch-tui").join("config.toml");
        
        if Path::exists(&config_path) {
            cfg = cfg.add_source(File::with_name(config_path.to_str().unwrap()).required(false));
        }

        // Add environment variables as overrides
        cfg = cfg.add_source(config::Environment::with_prefix("ARCH_TUI"));

        let config: AppConfig = cfg.build()?.try_deserialize()?;
        Ok(config)
    }
}