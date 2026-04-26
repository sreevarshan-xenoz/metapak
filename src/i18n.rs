//! Internationalization (i18n) support for Arch TUI
//!
//! This module provides localization capabilities for the application,
//! allowing it to display text in different languages based on user preference.

use std::collections::HashMap;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Chinese,
    Japanese,
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
            Language::Japanese => "ja",
        }
    }

    /// Detect language from environment or system locale
    pub fn detect() -> Self {
        // Check LANG environment variable
        if let Ok(lang) = std::env::var("LANG") {
            let lang_lower = lang.to_lowercase();
            if lang_lower.starts_with("es") {
                return Language::Spanish;
            }
            if lang_lower.starts_with("fr") {
                return Language::French;
            }
            if lang_lower.starts_with("de") {
                return Language::German;
            }
            if lang_lower.starts_with("zh") {
                return Language::Chinese;
            }
            if lang_lower.starts_with("ja") {
                return Language::Japanese;
            }
        }
        Language::English
    }
}

/// Localization manager
pub struct Localizer {
    current_language: Language,
    translations: HashMap<String, HashMap<String, String>>,
}

impl Localizer {
    /// Create a new localizer with auto-detected language
    pub fn new() -> Self {
        let language = Language::detect();
        let mut localizer = Self {
            current_language: language,
            translations: HashMap::new(),
        };

        localizer.load_all_translations();
        localizer
    }

    /// Load all translations
    fn load_all_translations(&mut self) {
        self.load_english();
        self.load_spanish();
        self.load_french();
        self.load_german();
        self.load_chinese();
        self.load_japanese();
    }

    fn load_english(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - Package Manager".into());
        t.insert("search_placeholder".into(), "Search packages...".into());
        t.insert("packages_label".into(), "Packages".into());
        t.insert("loading_label".into(), "Loading...".into());
        t.insert("installed_label".into(), "[INSTALLED]".into());
        t.insert("repo_label".into(), "Repo".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "Install".into());
        t.insert("remove_button".into(), "Remove".into());
        t.insert("update_system_button".into(), "Update System".into());
        t.insert("confirm_button".into(), "Confirm".into());
        t.insert("cancel_button".into(), "Cancel".into());
        t.insert("confirm_install".into(), "Confirm Installation".into());
        t.insert("confirm_remove".into(), "Confirm Removal".into());
        t.insert("confirm_multiple".into(), "Confirm Multiple Actions".into());
        t.insert("up_to_date".into(), "Up to Date".into());
        t.insert("updates_available".into(), "Updates Available".into());
        t.insert("error_occurred".into(), "An error occurred".into());
        t.insert("status".into(), "Status".into());
        t.insert("help_quit".into(), "Quit".into());
        t.insert("help_search".into(), "Search".into());
        t.insert("help_select".into(), "Select/Deselect".into());
        t.insert("help_install".into(), "Install/Remove".into());
        t.insert("help_update".into(), "Update System".into());
        t.insert("help_details".into(), "Show Details".into());
        t.insert("console_output_title".into(), "Command Output (Type 'y'/'n' to interact, 'Esc' to close)".into());
        t.insert("sudo_password_required".into(), "Sudo Password Required".into());
        t.insert("enter_sudo_password".into(), "Please enter your sudo password:".into());
        t.insert("password_label".into(), "Password".into());
        t.insert("confirm_single".into(), "Are you sure you want to".into());
        t.insert("confirmation_instructions".into(), "Press 'y' to proceed, 'n' to cancel".into());
        t.insert("filter_all".into(), "All".into());
        t.insert("filter_installed".into(), "Installed".into());
        t.insert("filter_not_installed".into(), "Not Installed".into());
        t.insert("sort_name".into(), "Name".into());
        t.insert("sort_size".into(), "Size".into());
        t.insert("sort_source".into(), "Source".into());
        t.insert("help_title".into(), "Help".into());
        t.insert("help_navigation".into(), "Navigation".into());
        t.insert("help_actions".into(), "Actions".into());
        t.insert("history_title".into(), "Transaction History".into());
        t.insert("rollback_prompt".into(), "Rollback this transaction?".into());
        t.insert("no_results_found".into(), "No packages found".into());
        t.insert("size_label".into(), "Size".into());
        t.insert("total_size".into(), "Total".into());
        self.translations.insert("en".into(), t);
    }

    fn load_spanish(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - Gestor de Paquetes".into());
        t.insert("search_placeholder".into(), "Buscar paquetes...".into());
        t.insert("packages_label".into(), "Paquetes".into());
        t.insert("loading_label".into(), "Cargando...".into());
        t.insert("installed_label".into(), "[INSTALADO]".into());
        t.insert("repo_label".into(), "Repositorio".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "Instalar".into());
        t.insert("remove_button".into(), "Eliminar".into());
        t.insert("update_system_button".into(), "Actualizar Sistema".into());
        t.insert("confirm_button".into(), "Confirmar".into());
        t.insert("cancel_button".into(), "Cancelar".into());
        t.insert("confirm_install".into(), "Confirmar Instalacion".into());
        t.insert("confirm_remove".into(), "Confirmar Eliminacion".into());
        t.insert("confirm_multiple".into(), "Confirmar Acciones Multiples".into());
        t.insert("up_to_date".into(), "Actualizado".into());
        t.insert("updates_available".into(), "Actualizaciones Disponibles".into());
        t.insert("error_occurred".into(), "Ocurrio un error".into());
        t.insert("no_results_found".into(), "No se encontraron paquetes".into());
        t.insert("status".into(), "Estado".into());
        t.insert("help_quit".into(), "Salir".into());
        t.insert("help_search".into(), "Buscar".into());
        t.insert("help_select".into(), "Seleccionar".into());
        t.insert("help_install".into(), "Instalar/Eliminar".into());
        t.insert("help_update".into(), "Actualizar Sistema".into());
        t.insert("help_details".into(), "Mostrar Detalles".into());
        t.insert("console_output_title".into(), "Salida (Escribe 'y'/'n' para interactuar, 'Esc' para cerrar)".into());
        t.insert("sudo_password_required".into(), "Contrasena Sudo Requerida".into());
        t.insert("enter_sudo_password".into(), "Ingrese su contrasena sudo:".into());
        t.insert("password_label".into(), "Contrasena".into());
        t.insert("confirm_single".into(), "Esta seguro de que desea".into());
        t.insert("confirmation_instructions".into(), "Presione 'y' para continuar, 'n' para cancelar".into());
        t.insert("filter_all".into(), "Todos".into());
        t.insert("filter_installed".into(), "Instalados".into());
        t.insert("filter_not_installed".into(), "No Instalados".into());
        t.insert("sort_name".into(), "Nombre".into());
        t.insert("sort_size".into(), "Tamano".into());
        t.insert("sort_source".into(), "Origen".into());
        t.insert("help_title".into(), "Ayuda".into());
        t.insert("help_navigation".into(), "Navegacion".into());
        t.insert("help_actions".into(), "Acciones".into());
        t.insert("history_title".into(), "Historial de Transacciones".into());
        t.insert("rollback_prompt".into(), "Revertir esta transaccion?".into());
        self.translations.insert("es".into(), t);
    }

    fn load_french(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - Gestionnaire de Paquets".into());
        t.insert("search_placeholder".into(), "Rechercher des paquets...".into());
        t.insert("packages_label".into(), "Paquets".into());
        t.insert("loading_label".into(), "Chargement...".into());
        t.insert("installed_label".into(), "[INSTALLE]".into());
        t.insert("repo_label".into(), "Depot".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "Installer".into());
        t.insert("remove_button".into(), "Supprimer".into());
        t.insert("update_system_button".into(), "Mettre a jour".into());
        t.insert("confirm_button".into(), "Confirmer".into());
        t.insert("cancel_button".into(), "Annuler".into());
        t.insert("confirm_install".into(), "Confirmer Installation".into());
        t.insert("confirm_remove".into(), "Confirmer Suppression".into());
        t.insert("confirm_multiple".into(), "Confirmer Actions Multiples".into());
        t.insert("up_to_date".into(), "A jour".into());
        t.insert("updates_available".into(), "Mises a jour disponibles".into());
        t.insert("error_occurred".into(), "Une erreur est survenue".into());
        t.insert("no_results_found".into(), "Aucun paquet trouve".into());
        t.insert("status".into(), "Statut".into());
        t.insert("help_quit".into(), "Quitter".into());
        t.insert("help_search".into(), "Rechercher".into());
        t.insert("help_select".into(), "Selectionner".into());
        t.insert("help_install".into(), "Installer/Supprimer".into());
        t.insert("help_update".into(), "Mettre a jour".into());
        t.insert("help_details".into(), "Afficher Details".into());
        t.insert("console_output_title".into(), "Sortie (Tapez 'y'/'n', 'Esc' pour fermer)".into());
        t.insert("sudo_password_required".into(), "Mot de passe Sudo Requis".into());
        t.insert("enter_sudo_password".into(), "Entrez votre mot de passe sudo:".into());
        t.insert("password_label".into(), "Mot de passe".into());
        t.insert("confirm_single".into(), "Etes-vous sur de vouloir".into());
        t.insert("confirmation_instructions".into(), "Appuyez 'y' pour continuer, 'n' pour annuler".into());
        t.insert("filter_all".into(), "Tous".into());
        t.insert("filter_installed".into(), "Installes".into());
        t.insert("filter_not_installed".into(), "Non Installes".into());
        t.insert("sort_name".into(), "Nom".into());
        t.insert("sort_size".into(), "Taille".into());
        t.insert("sort_source".into(), "Origine".into());
        t.insert("help_title".into(), "Aide".into());
        t.insert("help_navigation".into(), "Navigation".into());
        t.insert("help_actions".into(), "Actions".into());
        t.insert("history_title".into(), "Historique des Transactions".into());
        t.insert("rollback_prompt".into(), "Annuler cette transaction?".into());
        self.translations.insert("fr".into(), t);
    }

    fn load_german(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - Paketverwaltung".into());
        t.insert("search_placeholder".into(), "Pakete suchen...".into());
        t.insert("packages_label".into(), "Pakete".into());
        t.insert("loading_label".into(), "Laden...".into());
        t.insert("installed_label".into(), "[INSTALLIERT]".into());
        t.insert("repo_label".into(), "Depot".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "Installieren".into());
        t.insert("remove_button".into(), "Entfernen".into());
        t.insert("update_system_button".into(), "System aktualisieren".into());
        t.insert("confirm_button".into(), "Bestaetigen".into());
        t.insert("cancel_button".into(), "Abbrechen".into());
        t.insert("confirm_install".into(), "Installation bestaetigen".into());
        t.insert("confirm_remove".into(), "Entfernen bestaetigen".into());
        t.insert("confirm_multiple".into(), "Mehrfache Aktionen bestaetigen".into());
        t.insert("up_to_date".into(), "Aktuell".into());
        t.insert("updates_available".into(), "Updates verfuegbar".into());
        t.insert("error_occurred".into(), "Ein Fehler ist aufgetreten".into());
        t.insert("status".into(), "Status".into());
        t.insert("help_quit".into(), "Beenden".into());
        t.insert("help_search".into(), "Suchen".into());
        t.insert("help_select".into(), "Auswaehlen".into());
        t.insert("help_install".into(), "Installieren/Entfernen".into());
        t.insert("help_update".into(), "System aktualisieren".into());
        t.insert("help_details".into(), "Details anzeigen".into());
        t.insert("console_output_title".into(), "Ausgabe ('y'/'n', 'Esc' zum Schliessen)".into());
        t.insert("sudo_password_required".into(), "Sudo-Passwort erforderlich".into());
        t.insert("enter_sudo_password".into(), "Bitte Passwort eingeben:".into());
        t.insert("password_label".into(), "Passwort".into());
        t.insert("confirm_single".into(), "Sind Sie sicher, dass Sie".into());
        t.insert("confirmation_instructions".into(), "'y' zum Fortfahren, 'n' zum Abbrechen".into());
        t.insert("filter_all".into(), "Alle".into());
        t.insert("filter_installed".into(), "Installiert".into());
        t.insert("filter_not_installed".into(), "Nicht Installiert".into());
        t.insert("sort_name".into(), "Name".into());
        t.insert("sort_size".into(), "Groesse".into());
        t.insert("sort_source".into(), "Quelle".into());
        t.insert("help_title".into(), "Hilfe".into());
        t.insert("help_navigation".into(), "Navigation".into());
        t.insert("help_actions".into(), "Aktionen".into());
        t.insert("history_title".into(), "Transaktionsverlauf".into());
        t.insert("rollback_prompt".into(), "Diese Transaktion rueckgaengig machen?".into());
        self.translations.insert("de".into(), t);
    }

    fn load_chinese(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - 软件包管理器".into());
        t.insert("search_placeholder".into(), "搜索软件包...".into());
        t.insert("packages_label".into(), "软件包".into());
        t.insert("loading_label".into(), "加载中...".into());
        t.insert("installed_label".into(), "[已安装]".into());
        t.insert("repo_label".into(), "仓库".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "安装".into());
        t.insert("remove_button".into(), "卸载".into());
        t.insert("update_system_button".into(), "更新系统".into());
        t.insert("confirm_button".into(), "确认".into());
        t.insert("cancel_button".into(), "取消".into());
        t.insert("confirm_install".into(), "确认安装".into());
        t.insert("confirm_remove".into(), "确认卸载".into());
        t.insert("confirm_multiple".into(), "确认多操作".into());
        t.insert("up_to_date".into(), "已是最新".into());
        t.insert("updates_available".into(), "有可用更新".into());
        t.insert("error_occurred".into(), "发生错误".into());
        t.insert("status".into(), "状态".into());
        t.insert("help_quit".into(), "退出".into());
        t.insert("help_search".into(), "搜索".into());
        t.insert("help_select".into(), "选择".into());
        t.insert("help_install".into(), "安装/卸载".into());
        t.insert("help_update".into(), "更新系统".into());
        t.insert("help_details".into(), "显示详情".into());
        t.insert("console_output_title".into(), "命令输出 (输入'y'/'n', 'Esc'关闭)".into());
        t.insert("sudo_password_required".into(), "需要sudo密码".into());
        t.insert("enter_sudo_password".into(), "请输入sudo密码:".into());
        t.insert("password_label".into(), "密码".into());
        t.insert("confirm_single".into(), "确定要".into());
        t.insert("confirmation_instructions".into(), "按'y'继续, 'n'取消".into());
        t.insert("filter_all".into(), "全部".into());
        t.insert("filter_installed".into(), "已安装".into());
        t.insert("filter_not_installed".into(), "未安装".into());
        t.insert("sort_name".into(), "名称".into());
        t.insert("sort_size".into(), "大小".into());
        t.insert("sort_source".into(), "来源".into());
        t.insert("help_title".into(), "帮助".into());
        t.insert("help_navigation".into(), "导航".into());
        t.insert("help_actions".into(), "操作".into());
        t.insert("history_title".into(), "事务历史".into());
        t.insert("rollback_prompt".into(), "回滚此事务?".into());
        self.translations.insert("zh".into(), t);
    }

    fn load_japanese(&mut self) {
        let mut t = HashMap::new();
        t.insert("app_title".into(), "Arch TUI - パッケージ管理".into());
        t.insert("search_placeholder".into(), "パッケージを検索...".into());
        t.insert("packages_label".into(), "パッケージ".into());
        t.insert("loading_label".into(), "読み込み中...".into());
        t.insert("installed_label".into(), "[インストール済み]".into());
        t.insert("repo_label".into(), "リポジトリ".into());
        t.insert("aur_label".into(), "AUR".into());
        t.insert("install_button".into(), "インストール".into());
        t.insert("remove_button".into(), "削除".into());
        t.insert("update_system_button".into(), "システム更新".into());
        t.insert("confirm_button".into(), "確認".into());
        t.insert("cancel_button".into(), "キャンセル".into());
        t.insert("confirm_install".into(), "インストール確認".into());
        t.insert("confirm_remove".into(), "削除確認".into());
        t.insert("confirm_multiple".into(), "複数操作の確認".into());
        t.insert("up_to_date".into(), "最新です".into());
        t.insert("updates_available".into(), "アップデートあり".into());
        t.insert("error_occurred".into(), "エラーが発生しました".into());
        t.insert("status".into(), "ステータス".into());
        t.insert("help_quit".into(), "終了".into());
        t.insert("help_search".into(), "検索".into());
        t.insert("help_select".into(), "選択".into());
        t.insert("help_install".into(), "インストール/削除".into());
        t.insert("help_update".into(), "システム更新".into());
        t.insert("help_details".into(), "詳細表示".into());
        t.insert("console_output_title".into(), "出力 ('y'/'n', 'Esc'で閉じる)".into());
        t.insert("sudo_password_required".into(), "sudoパスワードが必要".into());
        t.insert("enter_sudo_password".into(), "sudoパスワードを入力:".into());
        t.insert("password_label".into(), "パスワード".into());
        t.insert("confirm_single".into(), "本当に".into());
        t.insert("confirmation_instructions".into(), "'y'で続行, 'n'でキャンセル".into());
        t.insert("filter_all".into(), "すべて".into());
        t.insert("filter_installed".into(), "インストール済み".into());
        t.insert("filter_not_installed".into(), "未インストール".into());
        t.insert("sort_name".into(), "名前".into());
        t.insert("sort_size".into(), "サイズ".into());
        t.insert("sort_source".into(), "ソース".into());
        t.insert("help_title".into(), "ヘルプ".into());
        t.insert("help_navigation".into(), "ナビゲーション".into());
        t.insert("help_actions".into(), "アクション".into());
        t.insert("history_title".into(), "トランザクション履歴".into());
        t.insert("rollback_prompt".into(), "このトランザクションをロールバック?".into());
        self.translations.insert("ja".into(), t);
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
        self.translations
            .insert(language.code().to_string(), translations);
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
        assert_eq!(*localizer.current_language(), Language::English);
    }

    #[test]
    fn test_get_translation() {
        let localizer = Localizer::new();
        assert_eq!(localizer.t("app_title"), "Arch TUI - Package Manager");
    }

    #[test]
    fn test_fallback_to_english() {
        let mut localizer = Localizer::new();
        localizer.set_language(Language::French);
        // French will work now with full translations
        assert_eq!(localizer.t("app_title"), "Arch TUI - Gestionnaire de Paquets");
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
        es_translations.insert(
            "app_title".to_string(),
            "Gestor de Paquetes Arch TUI".to_string(),
        );
        localizer.add_translations(Language::Spanish, es_translations);
        localizer.set_language(Language::Spanish);
        assert_eq!(localizer.t("app_title"), "Gestor de Paquetes Arch TUI");
    }

    #[test]
    fn test_spanish_translations_loaded() {
        let mut localizer = Localizer::new();
        localizer.set_language(Language::Spanish);
        assert_eq!(localizer.t("search_placeholder"), "Buscar paquetes...");
        assert_eq!(localizer.t("installed_label"), "[INSTALADO]");
    }

    #[test]
    fn test_chinese_translations_loaded() {
        let mut localizer = Localizer::new();
        localizer.set_language(Language::Chinese);
        assert_eq!(localizer.t("search_placeholder"), "搜索软件包...");
        assert_eq!(localizer.t("installed_label"), "[已安装]");
    }

    #[test]
    fn test_japanese_translations_loaded() {
        let mut localizer = Localizer::new();
        localizer.set_language(Language::Japanese);
        assert_eq!(localizer.t("search_placeholder"), "パッケージを検索...");
        assert_eq!(localizer.t("installed_label"), "[インストール済み]");
    }
}
