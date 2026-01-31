//! Application state management for Arch TUI
//!
//! This module contains the main application state and logic for managing
//! the TUI application's state, including search results, selections,
//! and UI modes.

use crate::models::Package;
use crate::action::Action;
use tokio::sync::mpsc::UnboundedSender;
use std::collections::HashMap;
use crate::errors::Result;
use crate::config::AppConfig;

pub enum InputMode {
    Normal,
    Editing,
}

/// Main application state
///
/// This struct holds the entire state of the TUI application, including
/// search inputs, results, selections, UI modes, and configuration.
pub struct App {
    /// Current search input text
    pub search_input: String,
    /// Current input mode (normal or editing)
    pub input_mode: InputMode,
    /// Search results from pacman and AUR
    pub results: Vec<Package>,
    /// Flag indicating if the application should quit
    pub should_quit: bool,
    /// Loading state indicator
    pub is_loading: bool,
    /// Sender for sending actions to the background task
    pub action_tx: Option<UnboundedSender<Action>>,
    /// Index of the currently selected package
    pub selected_index: Option<usize>,
    /// Pending command to execute in foreground
    pub pending_command: Option<(String, Vec<String>)>,
    /// Error message to display
    pub error_message: Option<String>,
    /// Number of available updates
    pub available_updates: Option<usize>,
    /// Flag indicating if password prompt should be shown
    pub show_password_prompt: bool,
    /// Current password input
    pub password_input: String,

    /// Selected packages for batch operations
    pub selected_packages: HashMap<String, Package>,

    /// Flag indicating if confirmation prompt should be shown
    pub show_confirm_prompt: bool,
    /// Packages pending confirmation for install/remove
    pub packages_pending_confirmation: Vec<Package>,

    /// Flag indicating if console output should be shown
    pub show_console: bool,
    /// Console output buffer
    pub console_buffer: Vec<String>,
    /// Sender for command stdin
    pub command_stdin_tx: Option<UnboundedSender<String>>,

    /// Application configuration
    pub config: AppConfig,

    /// Flag indicating if package details view should be shown
    pub show_package_details: bool,

    /// Localization manager
    pub localizer: crate::i18n::Localizer,

    /// Flag indicating if dependency visualization should be shown
    pub show_dependency_visualization: bool,
}

impl App {
    /// Creates a new instance of the application state
    ///
    /// Initializes all fields with default values and sets up the initial
    /// configuration with default settings.
    pub fn new() -> Self {
        Self {
            search_input: String::new(),
            input_mode: InputMode::Normal,
            results: Vec::new(),
            should_quit: false,
            is_loading: false,
            action_tx: None,
            selected_index: None,
            pending_command: None,
            error_message: None,
            available_updates: None,

            // Start with password prompt
            show_password_prompt: true,
            password_input: String::new(),

            // Batch Selection
            selected_packages: HashMap::new(),

            // Confirmation
            show_confirm_prompt: false,
            packages_pending_confirmation: Vec::new(),

            show_console: false,
            console_buffer: Vec::new(),
            command_stdin_tx: None,

            // Configuration
            config: crate::config::AppConfig::default(),

            // Package details view
            show_package_details: false,

            // Localization
            localizer: crate::i18n::Localizer::new(),

            // Dependency visualization
            show_dependency_visualization: false,
        }
    }

    /// Sets the action sender for communicating with the background task
    ///
    /// # Arguments
    /// * `tx` - The unbounded sender for sending actions
    pub fn set_sender(&mut self, tx: UnboundedSender<Action>) {
        self.action_tx = Some(tx);
    }

    /// Toggles the selection state of the currently selected package
    ///
    /// If the package is currently selected, it will be deselected.
    /// If the package is not selected, it will be added to the selection.
    pub fn toggle_selection(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(pkg) = self.results.get(idx) {
                if self.selected_packages.contains_key(&pkg.name) {
                    self.selected_packages.remove(&pkg.name);
                } else {
                    self.selected_packages.insert(pkg.name.clone(), pkg.clone());
                }
            }
        }
    }

    /// Moves the selection to the next package in the results list
    ///
    /// If the current selection is at the end of the list, it wraps around to the beginning.
    pub fn next(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.selected_index {
            Some(i) => {
                if i >= self.results.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);
    }

    /// Moves the selection to the previous package in the results list
    ///
    /// If the current selection is at the beginning of the list, it wraps around to the end.
    pub fn previous(&mut self) {
        if self.results.is_empty() {
            return;
        }
        let i = match self.selected_index {
            Some(i) => {
                if i == 0 {
                    self.results.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.selected_index = Some(i);
    }

    /// Gets the currently selected package if available
    pub fn get_selected_package(&self) -> Option<&Package> {
        if let Some(index) = self.selected_index {
            self.results.get(index)
        } else {
            None
        }
    }

    /// Shows the package details view for the currently selected package
    pub fn show_package_details(&mut self) {
        if self.selected_index.is_some() {
            self.show_package_details = true;
        }
    }

    /// Hides the package details view
    pub fn hide_package_details(&mut self) {
        self.show_package_details = false;
    }

    /// Shows the dependency visualization for the currently selected package
    pub fn show_dependency_visualization(&mut self) {
        if self.selected_index.is_some() {
            self.show_dependency_visualization = true;
        }
    }

    /// Hides the dependency visualization
    pub fn hide_dependency_visualization(&mut self) {
        self.show_dependency_visualization = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Package, PackageSource};

    #[test]
    fn test_package_creation() {
        let pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "A test package".to_string(),
            source: PackageSource::Pacman,
            is_installed: false,
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: vec![],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec![],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
        };

        assert_eq!(pkg.name, "test-package");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.description, "A test package");
        assert_eq!(pkg.source, PackageSource::Pacman);
        assert!(!pkg.is_installed);
    }

    #[test]
    fn test_app_initialization() {
        let app = App::new();

        assert_eq!(app.search_input, "");
        assert!(matches!(app.input_mode, InputMode::Normal));
        assert_eq!(app.results.len(), 0);
        assert!(!app.should_quit);
        assert!(!app.is_loading);
        assert!(app.selected_index.is_none());
        assert!(app.pending_command.is_none());
        assert!(app.error_message.is_none());
        assert!(app.available_updates.is_none());
        assert!(app.show_password_prompt);
        assert_eq!(app.password_input, "");
        assert_eq!(app.selected_packages.len(), 0);
        assert!(!app.show_confirm_prompt);
        assert_eq!(app.packages_pending_confirmation.len(), 0);
        assert!(!app.show_console);
        assert_eq!(app.console_buffer.len(), 0);
        assert!(app.command_stdin_tx.is_none());
    }

    #[test]
    fn test_toggle_selection() {
        let mut app = App::new();

        // Add a test package to results
        let test_pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "A test package".to_string(),
            source: PackageSource::Pacman,
            is_installed: false,
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: vec![],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec![],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
        };

        app.results.push(test_pkg.clone());
        app.selected_index = Some(0);

        // Initially not selected
        assert_eq!(app.selected_packages.len(), 0);

        // Toggle selection
        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 1);
        assert!(app.selected_packages.contains_key("test-package"));

        // Toggle again to deselect
        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 0);
        assert!(!app.selected_packages.contains_key("test-package"));
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();

        // Add some test packages
        for i in 0..3 {
            app.results.push(Package {
                name: format!("package-{}", i),
                version: "1.0.0".to_string(),
                description: format!("Test package {}", i),
                source: PackageSource::Pacman,
                is_installed: false,
                installed_size: None,
                download_size: None,
                groups: vec![],
                licenses: vec![],
                maintainers: vec![],
                keywords: vec![],
                url: None,
                depends_on: vec![],
                required_by: vec![],
                opt_depends: vec![],
                conflicts: vec![],
                replaces: vec![],
                provides: vec![],
            });
        }

        // Initially no selection
        assert!(app.selected_index.is_none());

        // Move to next (should wrap to 0)
        app.next();
        assert_eq!(app.selected_index, Some(0));

        // Move to next
        app.next();
        assert_eq!(app.selected_index, Some(1));

        // Move to next (should wrap to 0 when reaching end)
        app.next();
        assert_eq!(app.selected_index, Some(2));

        app.next();
        assert_eq!(app.selected_index, Some(0)); // Wrap around

        // Move to previous
        app.previous();
        assert_eq!(app.selected_index, Some(2));

        app.previous();
        assert_eq!(app.selected_index, Some(1));

        app.previous();
        assert_eq!(app.selected_index, Some(0));

        app.previous();
        assert_eq!(app.selected_index, Some(2)); // Wrap around
    }
}
