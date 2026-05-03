//! Customizable keyboard shortcuts system
//!
//! This module provides a flexible keybinding system that allows users
//! to customize keyboard shortcuts through configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crossterm::event::{KeyCode, KeyModifiers, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Vec<String>,
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
        Self {
            key: Self::keycode_to_string(key),
            modifiers: Self::modifiers_to_strings(modifiers),
            action,
        }
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        event.kind == KeyEventKind::Press
            && Self::keycode_to_string(event.code) == self.key
            && Self::modifiers_to_strings(event.modifiers) == self.modifiers
    }

    fn keycode_to_string(code: KeyCode) -> String {
        match code {
            KeyCode::Char(c) => c.to_string(),
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
            KeyCode::Char(' ') => "Space".to_string(),
            _ => format!("{:?}", code),
        }
    }

    fn modifiers_to_strings(modifiers: KeyModifiers) -> Vec<String> {
        let mut result = Vec::new();
        if modifiers.contains(KeyModifiers::CONTROL) {
            result.push("Ctrl".to_string());
        }
        if modifiers.contains(KeyModifiers::SHIFT) {
            result.push("Shift".to_string());
        }
        if modifiers.contains(KeyModifiers::ALT) {
            result.push("Alt".to_string());
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub bindings: Vec<KeyBinding>,
    #[serde(skip)]
    pub action_map: HashMap<Action, (KeyCode, KeyModifiers)>,
}

impl KeyBindings {
    pub fn default_bindings() -> Self {
        let bindings = vec![
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
            KeyBinding::new(KeyCode::Char('/'), KeyModifiers::NONE, Action::Search),
            KeyBinding::new(KeyCode::Char('n'), KeyModifiers::NONE, Action::ClearSearch),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::ClearSearch),
            KeyBinding::new(KeyCode::Char('i'), KeyModifiers::NONE, Action::Install),
            KeyBinding::new(KeyCode::Char('r'), KeyModifiers::NONE, Action::Remove),
            KeyBinding::new(KeyCode::Char('u'), KeyModifiers::NONE, Action::Update),
            KeyBinding::new(KeyCode::Char('U'), KeyModifiers::SHIFT, Action::UpdateAll),
            KeyBinding::new(KeyCode::Char('S'), KeyModifiers::SHIFT, Action::Upgrade),
            KeyBinding::new(KeyCode::Char('c'), KeyModifiers::NONE, Action::CleanCache),
            KeyBinding::new(KeyCode::Char('R'), KeyModifiers::NONE, Action::Refresh),
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
            KeyBinding::new(KeyCode::Char('B'), KeyModifiers::NONE, Action::Backup),
            KeyBinding::new(KeyCode::Char('b'), KeyModifiers::NONE, Action::Restore),
            KeyBinding::new(KeyCode::Char('X'), KeyModifiers::NONE, Action::SecurityAudit),
            KeyBinding::new(KeyCode::Char('T'), KeyModifiers::NONE, Action::ThemeNext),
            KeyBinding::new(KeyCode::Char('t'), KeyModifiers::SHIFT, Action::ThemePrevious),
            KeyBinding::new(KeyCode::Tab, KeyModifiers::NONE, Action::SwitchView),
            KeyBinding::new(KeyCode::Char('h'), KeyModifiers::NONE, Action::Help),
            KeyBinding::new(KeyCode::Char('?'), KeyModifiers::NONE, Action::Help),
            KeyBinding::new(KeyCode::Char('s'), KeyModifiers::NONE, Action::Settings),
            KeyBinding::new(KeyCode::Char('L'), KeyModifiers::NONE, Action::Logs),
            KeyBinding::new(KeyCode::Char(' '), KeyModifiers::NONE, Action::ToggleSelection),
            KeyBinding::new(KeyCode::Char('m'), KeyModifiers::NONE, Action::SelectMultiple),
            KeyBinding::new(KeyCode::Char('q'), KeyModifiers::NONE, Action::Quit),
            KeyBinding::new(KeyCode::Char('Q'), KeyModifiers::SHIFT, Action::Quit),
            KeyBinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::Cancel),
            KeyBinding::new(KeyCode::Enter, KeyModifiers::NONE, Action::Confirm),
        ];

        let mut action_map = HashMap::new();
        for binding in &bindings {
            action_map.insert(binding.action.clone(), (
                Self::parse_keycode(&binding.key),
                Self::parse_modifiers(&binding.modifiers),
            ));
        }

        Self { bindings, action_map }
    }

    fn parse_keycode(s: &str) -> KeyCode {
        if s.len() == 1 {
            return KeyCode::Char(s.chars().next().unwrap());
        }
        match s {
            "Up" => KeyCode::Up,
            "Down" => KeyCode::Down,
            "Left" => KeyCode::Left,
            "Right" => KeyCode::Right,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp,
            "PageDown" => KeyCode::PageDown,
            "Tab" => KeyCode::Tab,
            "Backspace" => KeyCode::Backspace,
            "Delete" => KeyCode::Delete,
            "Insert" => KeyCode::Insert,
            "Enter" => KeyCode::Enter,
            "Esc" => KeyCode::Esc,
            "Space" => KeyCode::Char(' '),
            s if s.starts_with('F') && s.len() <= 3 => {
                KeyCode::F(s[1..].parse().unwrap_or(1))
            }
            _ => KeyCode::Null,
        }
    }

    fn parse_modifiers(mods: &[String]) -> KeyModifiers {
        let mut result = KeyModifiers::NONE;
        for m in mods {
            match m.as_str() {
                "Ctrl" => result |= KeyModifiers::CONTROL,
                "Shift" => result |= KeyModifiers::SHIFT,
                "Alt" => result |= KeyModifiers::ALT,
                _ => {}
            }
        }
        result
    }

    pub fn find_action(&self, event: &KeyEvent) -> Option<Action> {
        for binding in &self.bindings {
            if binding.matches(event) {
                return Some(binding.action.clone());
            }
        }
        None
    }

    pub fn get_key_for_action(&self, action: &Action) -> Option<(KeyCode, KeyModifiers)> {
        self.action_map.get(action).copied()
    }

    pub fn reset_to_defaults(&mut self) {
        *self = Self::default_bindings();
    }

    pub fn get_bindings_for_action(&self, action: &Action) -> Vec<(String, Vec<String>)> {
        self.bindings
            .iter()
            .filter(|b| &b.action == action)
            .map(|b| (b.key.clone(), b.modifiers.clone()))
            .collect()
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
    fn test_keycode_parsing() {
        let key = KeyBindings::parse_keycode("F1");
        assert_eq!(key, KeyCode::F(1));

        let key = KeyBindings::parse_keycode("a");
        assert_eq!(key, KeyCode::Char('a'));
    }
}