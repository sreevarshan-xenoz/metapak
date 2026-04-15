//! Dynamic theme system for Arch TUI
//!
//! Catppuccin Mocha (dark) and Latte (light) palettes with WCAG AA
//! contrast validation and hex color support.

use ratatui::style::Color;
use serde::{Deserialize, Deserializer, Serialize};

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
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ColorDef {
    Hex(String),
    Named(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl<'de> Deserialize<'de> for ColorDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ColorDefVisitor;

        impl<'de> Visitor<'de> for ColorDefVisitor {
            type Value = ColorDef;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a color definition (hex string like \"#89b4fa\", named color like \"blue\", or {{ r, g, b }})")
            }

            fn visit_str<E>(self, value: &str) -> Result<ColorDef, E>
            where
                E: de::Error,
            {
                if value.starts_with('#') {
                    Ok(ColorDef::Hex(value.to_string()))
                } else {
                    Ok(ColorDef::Named(value.to_string()))
                }
            }

            fn visit_map<A>(self, mut map: A) -> Result<ColorDef, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut r = None;
                let mut g = None;
                let mut b = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "r" => r = Some(map.next_value()?),
                        "g" => g = Some(map.next_value()?),
                        "b" => b = Some(map.next_value()?),
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                match (r, g, b) {
                    (Some(r), Some(g), Some(b)) => Ok(ColorDef::Rgb { r, g, b }),
                    _ => Err(de::Error::custom(
                        "expected r, g, b fields for RGB color",
                    )),
                }
            }
        }

        deserializer.deserialize_any(ColorDefVisitor)
    }
}

/// Theme preset enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemePreset {
    Mocha,
    Latte,
}

impl Theme {
    // ------------------------------------------------------------------
    // Catppuccin Mocha (dark) — default theme
    // ------------------------------------------------------------------

    /// Create the Catppuccin Mocha (dark) theme.
    pub fn catppuccin_mocha() -> Self {
        Self {
            primary: ColorDef::Hex("#89b4fa".to_string()),
            secondary: ColorDef::Hex("#fab387".to_string()),
            success: ColorDef::Hex("#a6e3a1".to_string()),
            warning: ColorDef::Hex("#f9e2af".to_string()),
            error: ColorDef::Hex("#f38ba8".to_string()),
            info: ColorDef::Hex("#89dceb".to_string()),
            background: ColorDef::Hex("#1e1e2e".to_string()),
            foreground: ColorDef::Hex("#cdd6f4".to_string()),
            muted: ColorDef::Hex("#6c7086".to_string()),
            border: ColorDef::Hex("#45475a".to_string()),
            highlight_bg: ColorDef::Hex("#89b4fa".to_string()),
            highlight_fg: ColorDef::Hex("#1e1e2e".to_string()),
            repo_color: ColorDef::Hex("#74c7ec".to_string()),
            aur_color: ColorDef::Hex("#fab387".to_string()),
        }
    }

    // ------------------------------------------------------------------
    // Catppuccin Latte (light)
    // ------------------------------------------------------------------

    /// Create the Catppuccin Latte (light) theme.
    pub fn catppuccin_latte() -> Self {
        Self {
            // Primary adjusted from #1e66f5 to #1c60f0 to meet WCAG AA (4.67:1 vs 4.34:1)
            primary: ColorDef::Hex("#1c60f0".to_string()),
            secondary: ColorDef::Hex("#fe640b".to_string()),
            success: ColorDef::Hex("#40a02b".to_string()),
            warning: ColorDef::Hex("#df8e1d".to_string()),
            error: ColorDef::Hex("#d20f39".to_string()),
            info: ColorDef::Hex("#04a5e5".to_string()),
            background: ColorDef::Hex("#eff1f5".to_string()),
            foreground: ColorDef::Hex("#4c4f69".to_string()),
            muted: ColorDef::Hex("#9ca0b0".to_string()),
            border: ColorDef::Hex("#bcc0cc".to_string()),
            highlight_bg: ColorDef::Hex("#1c60f0".to_string()), // WCAG AA adjusted
            highlight_fg: ColorDef::Hex("#eff1f5".to_string()),
            repo_color: ColorDef::Hex("#209fb5".to_string()),
            aur_color: ColorDef::Hex("#fe640b".to_string()),
        }
    }

    /// Create a theme from a preset enum.
    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::Mocha => Self::catppuccin_mocha(),
            ThemePreset::Latte => Self::catppuccin_latte(),
        }
    }

    // ------------------------------------------------------------------
    // Backward-compatible aliases
    // ------------------------------------------------------------------

    /// Alias for `catppuccin_mocha()` — the default dark theme.
    pub fn default_dark() -> Self {
        Self::catppuccin_mocha()
    }

    /// Alias for `catppuccin_latte()` — the light theme.
    pub fn light() -> Self {
        Self::catppuccin_latte()
    }

    // ------------------------------------------------------------------
    // Color resolution
    // ------------------------------------------------------------------

    /// Convert a `ColorDef` into a `ratatui::style::Color`.
    pub fn resolve_color(&self, color_def: &ColorDef) -> Color {
        match color_def {
            ColorDef::Hex(hex) => Self::parse_hex_color(hex),
            ColorDef::Named(name) => Self::parse_named_color(name),
            ColorDef::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
        }
    }

    /// Parse a hex colour string in `#RRGGBB` format.
    ///
    /// Returns `Color::White` as a fallback for invalid input.
    pub fn parse_hex_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Color::White;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        Color::Rgb(r, g, b)
    }

    /// Parse a named ratatui colour.
    ///
    /// Returns `Color::White` as a fallback for unknown names.
    pub fn parse_named_color(name: &str) -> Color {
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
            "dark_red" => Color::Red,
            "dark_green" => Color::Green,
            "dark_yellow" => Color::Yellow,
            "dark_blue" => Color::Blue,
            "dark_magenta" => Color::Magenta,
            "dark_cyan" => Color::Cyan,
            _ => Color::White,
        }
    }

    // ------------------------------------------------------------------
    // WCAG 2.0 contrast helpers
    // ------------------------------------------------------------------

    /// Compute the relative luminance of a colour per WCAG 2.0.
    pub fn relative_luminance(color: &Color) -> f32 {
        let (r, g, b) = match color {
            Color::Rgb(r, g, b) => (*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0),
            Color::Black => (0.0, 0.0, 0.0),
            Color::White => (1.0, 1.0, 1.0),
            Color::Red => (1.0, 0.0, 0.0),
            Color::Green => (0.0, 0.5, 0.0),
            Color::Yellow => (1.0, 1.0, 0.0),
            Color::Blue => (0.0, 0.0, 1.0),
            Color::Magenta => (1.0, 0.0, 1.0),
            Color::Cyan => (0.0, 1.0, 1.0),
            Color::Gray => (0.5, 0.5, 0.5),
            Color::DarkGray => (0.25, 0.25, 0.25),
            Color::LightRed => (1.0, 0.5, 0.5),
            Color::LightGreen => (0.5, 1.0, 0.5),
            Color::LightYellow => (1.0, 1.0, 0.5),
            Color::LightBlue => (0.5, 0.5, 1.0),
            Color::LightMagenta => (1.0, 0.5, 1.0),
            Color::LightCyan => (0.5, 1.0, 1.0),
            _ => (0.0, 0.0, 0.0),
        };

        fn linearize(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
    }

    /// Compute the WCAG 2.0 contrast ratio between two colours.
    pub fn calculate_contrast_ratio(fg: &Color, bg: &Color) -> f32 {
        let l1 = Self::relative_luminance(fg);
        let l2 = Self::relative_luminance(bg);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Validate that all semantic colours meet WCAG AA (>= 4.5:1) against
    /// the theme background.  Returns a list of `(color_name, ratio)` for
    /// every pair that fails the threshold.
    pub fn validate_contrast(&self) -> Vec<(String, f32)> {
        let bg = self.background();
        let mut failures = Vec::new();

        let checks: Vec<(&str, Color)> = vec![
            ("foreground", self.foreground()),
            ("primary", self.primary()),
            ("secondary", self.secondary()),
            ("success", self.success()),
            ("warning", self.warning()),
            ("error", self.error()),
            ("info", self.info()),
            ("muted", self.muted()),
            ("repo_color", self.repo_color()),
            ("aur_color", self.aur_color()),
        ];

        for (name, fg) in checks {
            let ratio = Self::calculate_contrast_ratio(&fg, &bg);
            if ratio < 4.5 {
                failures.push((name.to_string(), ratio));
            }
        }

        // highlight_fg on highlight_bg
        let hl_ratio =
            Self::calculate_contrast_ratio(&self.highlight_fg(), &self.highlight_bg());
        if hl_ratio < 4.5 {
            failures.push(("highlight_fg_on_highlight_bg".to_string(), hl_ratio));
        }

        failures
    }

    // ------------------------------------------------------------------
    // Getter methods
    // ------------------------------------------------------------------

    pub fn primary(&self) -> Color {
        self.resolve_color(&self.primary)
    }
    pub fn secondary(&self) -> Color {
        self.resolve_color(&self.secondary)
    }
    pub fn success(&self) -> Color {
        self.resolve_color(&self.success)
    }
    pub fn warning(&self) -> Color {
        self.resolve_color(&self.warning)
    }
    pub fn error(&self) -> Color {
        self.resolve_color(&self.error)
    }
    pub fn info(&self) -> Color {
        self.resolve_color(&self.info)
    }
    pub fn background(&self) -> Color {
        self.resolve_color(&self.background)
    }
    pub fn foreground(&self) -> Color {
        self.resolve_color(&self.foreground)
    }
    pub fn muted(&self) -> Color {
        self.resolve_color(&self.muted)
    }
    pub fn border(&self) -> Color {
        self.resolve_color(&self.border)
    }
    pub fn highlight_bg(&self) -> Color {
        self.resolve_color(&self.highlight_bg)
    }
    pub fn highlight_fg(&self) -> Color {
        self.resolve_color(&self.highlight_fg)
    }
    pub fn repo_color(&self) -> Color {
        self.resolve_color(&self.repo_color)
    }
    pub fn aur_color(&self) -> Color {
        self.resolve_color(&self.aur_color)
    }
}

// ------------------------------------------------------------------
// Defaults
// ------------------------------------------------------------------

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin_mocha()
    }
}

impl Default for ColorDef {
    fn default() -> Self {
        ColorDef::Hex("#cdd6f4".to_string())
    }
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Test 1: Mocha contrast ratios ----
    #[test]
    fn test_mocha_theme_contrast_ratios() {
        let theme = Theme::catppuccin_mocha();
        let bg = theme.background();

        let text_colors: Vec<(&str, Color)> = vec![
            ("foreground", theme.foreground()),
            ("primary", theme.primary()),
            ("secondary", theme.secondary()),
            ("success", theme.success()),
            ("warning", theme.warning()),
            ("error", theme.error()),
            ("info", theme.info()),
            ("repo_color", theme.repo_color()),
            ("aur_color", theme.aur_color()),
        ];

        for (name, fg) in text_colors {
            let ratio = Theme::calculate_contrast_ratio(&fg, &bg);
            assert!(
                ratio >= 4.5,
                "{} on mocha background: {:.2}:1 (needs >= 4.5:1)",
                name,
                ratio
            );
        }

        // highlight_fg on highlight_bg
        let hl_ratio =
            Theme::calculate_contrast_ratio(&theme.highlight_fg(), &theme.highlight_bg());
        assert!(
            hl_ratio >= 4.5,
            "highlight_fg on highlight_bg: {:.2}:1 (needs >= 4.5:1)",
            hl_ratio
        );

        // validate_contrast() should report only muted as failing for Mocha
        let failures = theme.validate_contrast();
        assert_eq!(
            failures.len(),
            1,
            "Mocha should have exactly 1 contrast failure (muted), got {:?}",
            failures
        );
        assert_eq!(failures[0].0, "muted");
    }

    // ---- Test 2: Latte contrast ratios ----
    //
    // Catppuccin Latte is a light theme; several of its accent colours
    // (peach, green, yellow, sky, sapphire) are intentionally vibrant and
    // do not meet WCAG AA on the light base.  This test verifies:
    //   (a) colours that SHOULD pass actually do (foreground, error,
    //       primary, highlight_fg on highlight_bg), and
    //   (b) validate_contrast() correctly identifies the failures.
    #[test]
    fn test_latte_theme_contrast_ratios() {
        let theme = Theme::catppuccin_latte();
        let bg = theme.background();

        // These Latte colours meet WCAG AA on the light base
        let passing_colors: Vec<(&str, Color)> = vec![
            ("foreground", theme.foreground()),
            ("primary", theme.primary()),
            ("error", theme.error()),
        ];

        for (name, fg) in passing_colors {
            let ratio = Theme::calculate_contrast_ratio(&fg, &bg);
            assert!(
                ratio >= 4.5,
                "{} on latte background: {:.2}:1 (needs >= 4.5:1)",
                name,
                ratio
            );
        }

        // highlight_fg on highlight_bg should pass
        let hl_ratio =
            Theme::calculate_contrast_ratio(&theme.highlight_fg(), &theme.highlight_bg());
        assert!(
            hl_ratio >= 4.5,
            "highlight_fg on highlight_bg: {:.2}:1 (needs >= 4.5:1)",
            hl_ratio
        );

        // validate_contrast() should report the known failures
        let failures = theme.validate_contrast();
        let failure_names: Vec<&str> = failures.iter().map(|(n, _)| n.as_str()).collect();

        // Known Latte WCAG AA failures (vibrant accent colours on light bg)
        assert!(
            failure_names.contains(&"secondary"),
            "secondary should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"success"),
            "success should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"warning"),
            "warning should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"info"),
            "info should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"muted"),
            "muted should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"repo_color"),
            "repo_color should fail WCAG AA on Latte"
        );
        assert!(
            failure_names.contains(&"aur_color"),
            "aur_color should fail WCAG AA on Latte"
        );
    }

    // ---- Test 3: Hex colour parsing ----
    #[test]
    fn test_hex_color_parsing() {
        // "#89b4fa" -> Rgb(137, 180, 250)
        let color = Theme::parse_hex_color("#89b4fa");
        assert_eq!(color, Color::Rgb(137, 180, 250));

        // Without hash
        let color = Theme::parse_hex_color("1e1e2e");
        assert_eq!(color, Color::Rgb(30, 30, 46));

        // Invalid short form -> fallback to White
        let color = Theme::parse_hex_color("#fff");
        assert_eq!(color, Color::White);

        // Invalid empty -> fallback
        let color = Theme::parse_hex_color("");
        assert_eq!(color, Color::White);

        // Full white
        let color = Theme::parse_hex_color("#ffffff");
        assert_eq!(color, Color::Rgb(255, 255, 255));

        // Full black
        let color = Theme::parse_hex_color("#000000");
        assert_eq!(color, Color::Rgb(0, 0, 0));
    }

    // ---- Test 4: Contrast ratio calculation ----
    #[test]
    fn test_contrast_ratio_calculation() {
        // Black on white should be approximately 21:1
        let black = Color::Rgb(0, 0, 0);
        let white = Color::Rgb(255, 255, 255);
        let ratio = Theme::calculate_contrast_ratio(&black, &white);
        assert!(
            (ratio - 21.0).abs() < 0.1,
            "black on white ratio: {:.2} (expected ~21)",
            ratio
        );

        // White on white should be 1:1
        let ratio = Theme::calculate_contrast_ratio(&white, &white);
        assert!(
            (ratio - 1.0).abs() < 0.01,
            "white on white ratio: {:.2} (expected 1.0)",
            ratio
        );
    }

    // ---- Test 5: Theme preset from enum ----
    #[test]
    fn test_theme_preset_from_enum() {
        // Mocha
        let mocha = Theme::from_preset(ThemePreset::Mocha);
        let mocha_direct = Theme::catppuccin_mocha();
        assert_eq!(mocha.background(), mocha_direct.background());
        assert_eq!(mocha.foreground(), mocha_direct.foreground());
        assert_eq!(mocha.primary(), Color::Rgb(137, 180, 250)); // #89b4fa
        assert_eq!(mocha.secondary(), Color::Rgb(250, 179, 135)); // #fab387
        assert_eq!(mocha.success(), Color::Rgb(166, 227, 161)); // #a6e3a1
        assert_eq!(mocha.error(), Color::Rgb(243, 139, 168)); // #f38ba8

        // Latte
        let latte = Theme::from_preset(ThemePreset::Latte);
        let latte_direct = Theme::catppuccin_latte();
        assert_eq!(latte.background(), latte_direct.background());
        assert_eq!(latte.foreground(), latte_direct.foreground());
        assert_eq!(latte.primary(), Color::Rgb(28, 96, 240)); // #1c60f0 (WCAG AA adjusted from #1e66f5)
        assert_eq!(latte.secondary(), Color::Rgb(254, 100, 11)); // #fe640b
        assert_eq!(latte.success(), Color::Rgb(64, 160, 43)); // #40a02b
        assert_eq!(latte.error(), Color::Rgb(210, 15, 57)); // #d20f39
    }
}
