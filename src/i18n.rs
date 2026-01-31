//! Internationalization (i18n) support for Arch TUI
//! 
//! This module provides localization capabilities for the application,
//! allowing it to display text in different languages based on user preference.

use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, PartialEq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
}

impl Language {
    /// Get the language code (e.g., "en", "es", "fr")
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Chinese => "zh",
        }
    }
}

/// Localization manager
pub struct Localizer {
    current_language: Language,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Localizer {
    /// Create a new localizer with the default language (English)
    pub fn new() -> Self {
        let mut localizer = Self {
            current_language: Language::English,
            translations: HashMap::new(),
        };
        
        // Initialize with default English translations
        localizer.load_default_translations();
        localizer
    }

    /// Load default English translations
    fn load_default_translations(&mut self) {
        let mut en_translations = HashMap::new();
        
        // UI labels
        en_translations.insert("app_title".to_string(), "Arch TUI - Package Manager".to_string());
        en_translations.insert("search_placeholder".to_string(), "Search packages...".to_string());
        en_translations.insert("packages_label".to_string(), "Packages".to_string());
        en_translations.insert("loading_label".to_string(), "Loading...".to_string());
        en_translations.insert("installed_label".to_string(), "[INSTALLED]".to_string());
        en_translations.insert("repo_label".to_string(), "Repo".to_string());
        en_translations.insert("aur_label".to_string(), "AUR".to_string());
        
        // Button labels
        en_translations.insert("install_button".to_string(), "Install".to_string());
        en_translations.insert("remove_button".to_string(), "Remove".to_string());
        en_translations.insert("update_system_button".to_string(), "Update System".to_string());
        en_translations.insert("confirm_button".to_string(), "Confirm".to_string());
        en_translations.insert("cancel_button".to_string(), "Cancel".to_string());
        
        // Messages
        en_translations.insert("confirm_install".to_string(), "Confirm Installation".to_string());
        en_translations.insert("confirm_remove".to_string(), "Confirm Removal".to_string());
        en_translations.insert("confirm_multiple".to_string(), "Confirm Multiple Actions".to_string());
        en_translations.insert("up_to_date".to_string(), "Up to Date".to_string());
        en_translations.insert("updates_available".to_string(), "Updates Available".to_string());
        en_translations.insert("error_occurred".to_string(), "An error occurred".to_string());
        en_translations.insert("status".to_string(), "Status".to_string());
        
        // Help text
        en_translations.insert("help_quit".to_string(), "Quit".to_string());
        en_translations.insert("help_search".to_string(), "Search".to_string());
        en_translations.insert("help_select".to_string(), "Select/Deselect".to_string());
        en_translations.insert("help_install".to_string(), "Install/Remove".to_string());
        en_translations.insert("help_update".to_string(), "Update System".to_string());
        en_translations.insert("help_details".to_string(), "Show Details".to_string());
        en_translations.insert("console_output_title".to_string(), "Command Output (Type 'y'/'n' to interact, 'Esc' to close)".to_string());
        en_translations.insert("sudo_password_required".to_string(), "Sudo Password Required".to_string());
        en_translations.insert("enter_sudo_password".to_string(), "Please enter your sudo password:".to_string());
        en_translations.insert("password_label".to_string(), "Password".to_string());
        en_translations.insert("confirm_single".to_string(), "Are you sure you want to".to_string());
        en_translations.insert("confirmation_instructions".to_string(), "Press 'y' to proceed, 'n' to cancel".to_string());

        self.translations.insert(Language::English.code().to_string(), en_translations);
    }

    /// Set the current language
    pub fn set_language(&mut self, language: Language) {
        self.current_language = language;
    }

    /// Get the current language
    pub fn current_language(&self) -> &Language {
        &self.current_language
    }

    /// Get a localized string
    pub fn t(&self, key: &str) -> String {
        let lang_code = self.current_language.code();
        
        if let Some(lang_map) = self.translations.get(lang_code) {
            if let Some(value) = lang_map.get(key) {
                return value.clone();
            }
        }
        
        // Fallback to English
        if lang_code != Language::English.code() {
            if let Some(en_map) = self.translations.get(Language::English.code()) {
                if let Some(value) = en_map.get(key) {
                    return value.clone();
                }
            }
        }
        
        // Return the key if no translation is found
        key.to_string()
    }

    /// Add translations for a language
    pub fn add_translations(&mut self, language: Language, translations: HashMap<String, String>) {
        self.translations.insert(language.code().to_string(), translations);
    }
}

impl Default for Localizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_language_is_english() {
        let localizer = Localizer::new();
        assert_eq!(localizer.current_language(), &Language::English);
    }

    #[test]
    fn test_get_translation() {
        let localizer = Localizer::new();
        assert_eq!(localizer.t("app_title"), "Arch TUI - Package Manager");
    }

    #[test]
    fn test_fallback_to_english() {
        let mut localizer = Localizer::new();
        localizer.set_language(Language::Spanish);
        // Spanish translation doesn't exist, should fall back to English
        assert_eq!(localizer.t("app_title"), "Arch TUI - Package Manager");
    }

    #[test]
    fn test_unknown_key_returns_key() {
        let localizer = Localizer::new();
        assert_eq!(localizer.t("nonexistent_key"), "nonexistent_key");
    }

    #[test]
    fn test_custom_translation() {
        let mut localizer = Localizer::new();
        let mut es_translations = HashMap::new();
        es_translations.insert("app_title".to_string(), "Gestor de Paquetes Arch TUI".to_string());
        localizer.add_translations(Language::Spanish, es_translations);
        localizer.set_language(Language::Spanish);
        assert_eq!(localizer.t("app_title"), "Gestor de Paquetes Arch TUI");
    }
}