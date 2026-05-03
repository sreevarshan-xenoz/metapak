//! Customizable keyboard shortcuts system
//!
//! This module provides a flexible keybinding system that allows users
//! to customize keyboard shortcuts through configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Quit,
    Search,
    Install,
    Remove,
    Update,
    UpdateAll,
    Upgrade,
    ViewDetails,
    ViewDeps,
    ViewReverseDeps,
    CleanCache,
    SwitchView,
    NextItem,
    PreviousItem,
    NextPage,
    PreviousPage,
    GoTop,
    GoBottom,
    Refresh,
    ToggleSelection,
    SelectMultiple,
    ClearSearch,
    Help,
    Settings,
    SystemInfo,
    Orphans,
    PackageSizes,
    CacheInfo,
    ForeignPackages,
    PackageGroups,
    Backup,
    Restore,
    SecurityAudit,
    ThemeNext,
    ThemePrevious,
    Logs,
    Execute,
    Confirm,
    Cancel,
    TabNext,
    TabPrevious,
    Resize,
}

impl KeyBinding {
    pub fn new(key: KeyCode, modifiers: KeyModifiers, action: Action) -> Self {
        Self { key, modifiers, action }
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.kind == KeyEventKind::Press
            && self.key == event.code
            && self.modifiers == event.modifiers
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub bindings: Vec<KeyBinding>,
    pub custom: HashMap<String, Action>,
}

impl KeyBindings {
    pub fn default_bindings() -> Self {
        let bindings = vec![
            // Navigation
            KeyBinding::new(KeyCode::Char('j'), KeyModifiers::NONE, Action::NextItem),
            KeyBinding::new(KeyCode::Char('k'), KeyModifiers::NONE, Action::PreviousItem),
            KeyBinding::new(KeyCode::Down, KeyModifiers::NONE, Action::NextItem),
            KeyBinding::new(KeyCode::Up, KeyModifiers::NONE, Action::PreviousItem),
            KeyBinding::new(KeyCode::Char('d'), KeyModifiers::CONTROL, Action::NextPage),
            KeyBinding::new(KeyCode::Char('u'), KeyModifiers::CONTROL, Action::PreviousPage),
            KeyBinding::new(KeyCode::Char('f'), KeyModifiers::CONTROL, Action::NextPage),
            KeyBinding::new(KeyCode::Char('b'), KeyModifiers::CONTROL, Action::PreviousPage),
            KeyBinding::new(KeyCode::Char('g'), KeyModifiers::NONE, Action::GoTop),
            KeyBinding::new(KeyCode::Char('G'), KeyModifiers::SHIFT, Action::GoBottom),
            KeyBinding::new(KeyCode::Home, KeyModifiers::NONE, Action::GoTop),
            KeyBinding::new(KeyCode::End, KeyModifiers::NONE, Action::GoBottom),

            // Search and Filter
            KeyBinding::new(KeyCode::Char('/'), KeyModifiers::NONE, Action::Search),
            KeyBinding::new(KeyCode::Char('n'), KeyModifiers::NONE, Action::ClearSearch),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::ClearSearch),

            // Package Operations
            KeyBinding::new(KeyCode::Char('i'), KeyModifiers::NONE, Action::Install),
            KeyBinding::new(KeyCode::Char('r'), KeyModifiers::NONE, Action::Remove),
            KeyBinding::new(KeyCode::Char('u'), KeyModifiers::NONE, Action::Update),
            KeyBinding::new(KeyCode::Char('U'), KeyModifiers::SHIFT, Action::UpdateAll),

            // System Operations
            KeyBinding::new(KeyCode::Char('S'), KeyModifiers::SHIFT, Action::Upgrade),
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::NONE, Action::CleanCache),
            KeyBinding::new(KeyCode::Char('R'), KeyModifiers::NONE, Action::Refresh),

            // Information Views
            KeyBinding::new(KeyCode::Char('D'), KeyModifiers::NONE, Action::ViewDeps),
            KeyBinding::new(KeyCode::Char('d'), KeyModifiers::SHIFT, Action::ViewReverseDeps),
            KeyBinding::new(KeyCode::Char('V'), KeyModifiers::NONE, Action::ViewDetails),
            KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE, Action::ViewDetails),
            KeyBinding::new(KeyCode::Char('I'), KeyModifiers::NONE, Action::SystemInfo),
            KeyBinding::new(KeyCode::Char('O'), KeyModifiers::NONE, Action::Orphans),
            KeyBinding::new(KeyCode::Char('P'), KeyModifiers::NONE, Action::PackageSizes),
            KeyBinding::new(KeyCode::Char('C'), KeyModifiers::SHIFT, Action::CacheInfo),
            KeyBinding::new(KeyCode::Char('F'), KeyModifiers::NONE, Action::ForeignPackages),
            KeyBinding::new(KeyCode::Char('G'), KeyModifiers::NONE, Action::PackageGroups),

            // Backup/Restore/Security
            KeyBinding::new(KeyCode::Char('B'), KeyModifiers::NONE, Action::Backup),
            KeyBinding::new(KeyCode::Char('b'), KeyModifiers::NONE, Action::Restore),
            KeyBinding::new(KeyCode::Char('X'), KeyModifiers::NONE, Action::SecurityAudit),

            // Theme
            KeyBinding::new(KeyCode::Char('T'), KeyModifiers::NONE, Action::ThemeNext),
            KeyBinding::new(KeyCode::Char('t'), KeyModifiers::SHIFT, Action::ThemePrevious),

            // General
            KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE, Action::SwitchView),
            KeyBinding::new(KeyCode::Char('h'), KeyModifiers::NONE, Action::Help),
            KeyBinding::new(KeyCode::Char('?'), KeyModifiers::NONE, Action::Help),
            KeyBinding::new(KeyCode::Char('s'), KeyModifiers::NONE, Action::Settings),
            KeyBinding::new(KeyCode::Char('L'), KeyModifiers::NONE, Action::Logs),
            KeyBinding::new(KeyCode::Char('e'), KeyModifiers::NONE, Action::Execute),
            KeyBinding::new(KeyCode::Space, KeyModifiers::NONE, Action::ToggleSelection),
            KeyBinding::new(KeyCode::Char('m'), KeyModifiers::NONE, Action::SelectMultiple),

            // Quit and navigation
            KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE, Action::Quit),
            KeyBinding::new(KeyCode::Char('Q'), KeyModifiers::SHIFT, Action::Quit),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::Cancel),

            // Confirm/Cancel for operations
            KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE, Action::Confirm),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::Cancel),
        ];

        let custom = HashMap::new();

        Self { bindings, custom }
    }

    pub fn find_action(&self, event: &KeyEvent) -> Option<Action> {
        for binding in &self.bindings {
            if binding.matches(event) {
                return Some(binding.action.clone());
            }
        }
        None
    }

    pub fn set_custom_keybinding(&mut self, name: &str, key: KeyCode, modifiers: KeyModifiers) {
        if let Some(action) = self.custom.get(name) {
            self.bindings.retain(|b| b.action != *action);
            self.bindings.push(KeyBinding::new(key, modifiers, action.clone()));
        }
    }

    pub fn add_custom_binding(&mut self, name: String, key: KeyCode, modifiers: KeyModifiers, action: Action) {
        self.custom.insert(name.clone(), action.clone());
        self.bindings.push(KeyBinding::new(key, modifiers, action));
    }

    pub fn remove_custom_binding(&mut self, name: &str) {
        if let Some(action) = self.custom.remove(name) {
            self.bindings.retain(|b| b.action != action);
        }
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default_bindings();
    }

    pub fn get_bindings_for_action(&self, action: &Action) -> Vec<(KeyCode, KeyModifiers)> {
        self.bindings
            .iter()
            .filter(|b| &b.action == action)
            .map(|b| (b.key, b.modifiers))
            .collect()
    }

    pub fn format_binding(key: KeyCode, modifiers: KeyModifiers) -> String {
        let mut parts = Vec::new();

        if modifiers.contains(KeyModifiers::CONTROL) {
            parts.push("Ctrl".to_string());
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            parts.push("Shift".to_string());
        }
        if modifiers.contains(KeyModifiers::ALT) {
            parts.push("Alt".to_string());
        }

        let key_str = match key {
            KeyCode::Char(c) => c.to_uppercase().to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Space => "Space".to_string(),
            _ => format!("{:?}", key),
        };

        if parts.is_empty() {
            key_str
        } else {
            parts.push(key_str);
            parts.join("+")
        }
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::default_bindings()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindingConfig {
    pub enabled: bool,
    pub custom_keybindings: HashMap<String, KeyBindingConfigEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindingConfigEntry {
    pub key: String,
    pub modifiers: Vec<String>,
}

impl KeybindingConfig {
    pub fn default_config() -> Self {
        Self {
            enabled: true,
            custom_keybindings: HashMap::new(),
        }
    }
}

impl Default for KeybindingConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

pub struct HelpDisplay;

impl HelpDisplay {
    pub fn get_help_text() -> Vec<(&'static str, &'static str)> {
        vec![
            ("Navigation", ""),
            ("j/k or ↑/↓", "Move up/down"),
            ("g / G", "Go to top/bottom"),
            ("Ctrl+d / Ctrl+u", "Page down/up"),
            ("", ""),
            ("Search", ""),
            ("/", "Start search"),
            ("n", "Clear search"),
            ("Esc", "Clear search"),
            ("", ""),
            ("Package Operations", ""),
            ("i", "Install package"),
            ("r", "Remove package"),
            ("u", "Update package"),
            ("U", "Update all packages"),
            ("Enter", "View package details"),
            ("Space", "Toggle selection"),
            ("", ""),
            ("System", ""),
            ("S", "System upgrade"),
            ("c", "Clean cache"),
            ("R", "Refresh package databases"),
            ("", ""),
            ("Views", ""),
            ("Tab", "Switch view"),
            ("I", "System info"),
            ("O", "Orphan packages"),
            ("P", "Package sizes"),
            ("C", "Cache info"),
            ("F", "Foreign packages"),
            ("G", "Package groups"),
            ("D", "View dependencies"),
            ("d", "View reverse dependencies"),
            ("", ""),
            ("Tools", ""),
            ("B", "Backup packages"),
            ("b", "Restore packages"),
            ("X", "Security audit"),
            ("T", "Next theme"),
            ("t", "Previous theme"),
            ("", ""),
            ("General", ""),
            ("h / ?", "Show help"),
            ("s", "Settings"),
            ("L", "View logs"),
            ("q / Q", "Quit"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_bindings() {
        let bindings = KeyBindings::default_bindings();
        assert!(!bindings.bindings.is_empty());
    }

    #[test]
    fn test_find_action() {
        let bindings = KeyBindings::default_bindings();
        let event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE, KeyEventKind::Press);
        let action = bindings.find_action(&event);
        assert_eq!(action, Some(Action::Quit));
    }

    #[test]
    fn test_custom_bindings() {
        let mut bindings = KeyBindings::default_bindings();
        bindings.add_custom_binding(
            "my_action".to_string(),
            KeyCode::Char('x'),
            KeyModifiers::CONTROL,
            Action::Quit,
        );
        assert!(bindings.custom.contains_key("my_action"));
    }

    #[test]
    fn test_format_binding() {
        assert_eq!(
            KeyBindings::format_binding(KeyCode::Char('q'), KeyModifiers::CONTROL),
            "Ctrl+Q"
        );
        assert_eq!(
            KeyBindings::format_binding(KeyCode::Char('j'), KeyModifiers::NONE),
            "J"
        );
    }
}