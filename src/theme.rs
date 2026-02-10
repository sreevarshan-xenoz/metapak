//! Dynamic theme system for Arch TUI
//!
//! This module provides comprehensive theming support with customizable colors
//! for all UI elements.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Complete theme configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Theme {
    /// Primary accent color (used for highlights, selected items)
    pub primary: ColorDef,

    /// Secondary color (used for secondary highlights)
    pub secondary: ColorDef,

    /// Success color (installed packages, success messages)
    pub success: ColorDef,

    /// Warning color (updates available, warnings)
    pub warning: ColorDef,

    /// Error color (errors, failures)
    pub error: ColorDef,

    /// Info color (informational elements)
    pub info: ColorDef,

    /// Background color
    pub background: ColorDef,

    /// Foreground/text color
    pub foreground: ColorDef,

    /// Muted/secondary text color
    pub muted: ColorDef,

    /// Border color
    pub border: ColorDef,

    /// Highlight background color
    pub highlight_bg: ColorDef,

    /// Highlight foreground color
    pub highlight_fg: ColorDef,

    /// Source-specific colors
    pub repo_color: ColorDef,
    pub aur_color: ColorDef,
}

/// Color definition that can be parsed from string or RGB values
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorDef {
    Named(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl Theme {
    /// Create default dark theme
    pub fn default_dark() -> Self {
        Self {
            primary: ColorDef::Rgb {
                r: 100,
                g: 150,
                b: 255,
            },
            secondary: ColorDef::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
            success: ColorDef::Named("green".to_string()),
            warning: ColorDef::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
            error: ColorDef::Named("red".to_string()),
            info: ColorDef::Named("cyan".to_string()),
            background: ColorDef::Rgb {
                r: 25,
                g: 25,
                b: 35,
            },
            foreground: ColorDef::Rgb {
                r: 220,
                g: 220,
                b: 220,
            },
            muted: ColorDef::Rgb {
                r: 128,
                g: 128,
                b: 128,
            },
            border: ColorDef::Rgb {
                r: 80,
                g: 80,
                b: 80,
            },
            highlight_bg: ColorDef::Rgb {
                r: 100,
                g: 150,
                b: 255,
            },
            highlight_fg: ColorDef::Named("black".to_string()),
            repo_color: ColorDef::Named("blue".to_string()),
            aur_color: ColorDef::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
        }
    }

    /// Create light theme
    pub fn light() -> Self {
        Self {
            primary: ColorDef::Rgb {
                r: 0,
                g: 100,
                b: 200,
            },
            secondary: ColorDef::Rgb {
                r: 200,
                g: 100,
                b: 0,
            },
            success: ColorDef::Named("dark_green".to_string()),
            warning: ColorDef::Rgb {
                r: 200,
                g: 150,
                b: 0,
            },
            error: ColorDef::Named("dark_red".to_string()),
            info: ColorDef::Named("dark_cyan".to_string()),
            background: ColorDef::Named("white".to_string()),
            foreground: ColorDef::Named("black".to_string()),
            muted: ColorDef::Rgb {
                r: 100,
                g: 100,
                b: 100,
            },
            border: ColorDef::Rgb {
                r: 150,
                g: 150,
                b: 150,
            },
            highlight_bg: ColorDef::Rgb {
                r: 0,
                g: 100,
                b: 200,
            },
            highlight_fg: ColorDef::Named("white".to_string()),
            repo_color: ColorDef::Named("blue".to_string()),
            aur_color: ColorDef::Rgb {
                r: 200,
                g: 100,
                b: 0,
            },
        }
    }

    /// Convert ColorDef to ratatui Color
    pub fn resolve_color(&self, color_def: &ColorDef) -> Color {
        match color_def {
            ColorDef::Named(name) => Self::parse_named_color(name),
            ColorDef::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }

    /// Get primary color
    pub fn primary(&self) -> Color {
        self.resolve_color(&self.primary)
    }

    /// Get secondary color
    pub fn secondary(&self) -> Color {
        self.resolve_color(&self.secondary)
    }

    /// Get success color
    pub fn success(&self) -> Color {
        self.resolve_color(&self.success)
    }

    /// Get warning color
    pub fn warning(&self) -> Color {
        self.resolve_color(&self.warning)
    }

    /// Get error color
    pub fn error(&self) -> Color {
        self.resolve_color(&self.error)
    }

    /// Get info color
    pub fn info(&self) -> Color {
        self.resolve_color(&self.info)
    }

    /// Get background color
    pub fn background(&self) -> Color {
        self.resolve_color(&self.background)
    }

    /// Get foreground color
    pub fn foreground(&self) -> Color {
        self.resolve_color(&self.foreground)
    }

    /// Get muted color
    pub fn muted(&self) -> Color {
        self.resolve_color(&self.muted)
    }

    /// Get border color
    pub fn border(&self) -> Color {
        self.resolve_color(&self.border)
    }

    /// Get highlight background color
    pub fn highlight_bg(&self) -> Color {
        self.resolve_color(&self.highlight_bg)
    }

    /// Get highlight foreground color
    pub fn highlight_fg(&self) -> Color {
        self.resolve_color(&self.highlight_fg)
    }

    /// Get repo package color
    pub fn repo_color(&self) -> Color {
        self.resolve_color(&self.repo_color)
    }

    /// Get AUR package color
    pub fn aur_color(&self) -> Color {
        self.resolve_color(&self.aur_color)
    }

    /// Parse named color
    fn parse_named_color(name: &str) -> Color {
        match name.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "dark_gray" | "dark_grey" => Color::DarkGray,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_yellow" => Color::LightYellow,
            "light_blue" => Color::LightBlue,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            "white" => Color::White,
            _ => Color::White,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::default_dark()
    }
}

impl Default for ColorDef {
    fn default() -> Self {
        ColorDef::Named("white".to_string())
    }
}

/// Theme manager for runtime theme switching
pub struct ThemeManager {
    current: Theme,
    available: Vec<(String, Theme)>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut available = Vec::new();
        available.push(("dark".to_string(), Theme::default_dark()));
        available.push(("light".to_string(), Theme::light()));

        Self {
            current: Theme::default_dark(),
            available,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.current
    }

    pub fn set_theme(&mut self, name: &str) -> bool {
        if let Some((_, theme)) = self.available.iter().find(|(n, _)| n == name) {
            self.current = theme.clone();
            true
        } else {
            false
        }
    }

    pub fn available_themes(&self) -> Vec<&str> {
        self.available.iter().map(|(n, _)| n.as_str()).collect()
    }

    pub fn add_theme(&mut self, name: String, theme: Theme) {
        // Remove if exists
        self.available.retain(|(n, _)| n != &name);
        self.available.push((name, theme));
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.primary(), Color::Rgb(100, 150, 255));
    }

    #[test]
    fn test_parse_named_colors() {
        let theme = Theme::default();
        assert_eq!(
            theme.resolve_color(&ColorDef::Named("red".to_string())),
            Color::Red
        );
        assert_eq!(
            theme.resolve_color(&ColorDef::Named("BLUE".to_string())),
            Color::Blue
        );
        assert_eq!(
            theme.resolve_color(&ColorDef::Named("Green".to_string())),
            Color::Green
        );
    }

    #[test]
    fn test_rgb_color() {
        let theme = Theme::default();
        let rgb = ColorDef::Rgb {
            r: 255,
            g: 128,
            b: 64,
        };
        assert_eq!(theme.resolve_color(&rgb), Color::Rgb(255, 128, 64));
    }
}
